#include <microkit.h>
#include <sddf/timer/config.h>
#include <sddf/network/config.h>
#include <sddf/network/constants.h>
#include <sddf/network/queue.h>
#include <string.h>

extern void demi_sddf_notify(unsigned channel);
extern void demi_sddf_fault_drop(unsigned kind);

void _assert_fail(const char *expression, const char *file, unsigned line,
                  const char *function)
{
    (void)expression;
    (void)file;
    (void)line;
    (void)function;
    __builtin_trap();
}

__attribute__((section(".net_client_config"), used))
net_client_config_t demi_sddf_net_config;

const uint8_t *demi_sddf_net_mac(void)
{
    return demi_sddf_net_config.mac_addr.addr;
}

static net_queue_handle_t rx;
static net_queue_handle_t tx;
static uint64_t tx_recycled[4096];
static size_t tx_recycled_len;

#if CATLIONS_DROP_TCP
static int dropped_tcp;
static int drop_enabled = 1;

static int drop_tcp_packet(const uint8_t *data, size_t len)
{
    if (!drop_enabled || (dropped_tcp && CATLIONS_DROP_TCP != 3) || len < 54 ||
        data[12] != 0x08 || data[13] != 0x00 || data[23] != 6)
        return 0;
    size_t ip_header_len = (data[14] & 0x0f) * 4;
    if (ip_header_len < 20 || len < 14 + ip_header_len + 20)
        return 0;
    const uint8_t *tcp = data + 14 + ip_header_len;
    size_t tcp_header_len = (tcp[12] >> 4) * 4;
    size_t ip_total_len = ((size_t)data[16] << 8) | data[17];
    int match = CATLIONS_DROP_TCP != 2 ? ((tcp[13] & 0x12) == 0x12) :
        (ip_total_len > ip_header_len + tcp_header_len);
    if (match) {
        if (!dropped_tcp)
            demi_sddf_fault_drop(CATLIONS_DROP_TCP);
        dropped_tcp++;
    }
    return match;
}
#endif

void demi_sddf_net_clear_drop(void)
{
#if CATLIONS_DROP_TCP
    drop_enabled = 0;
#endif
}

int demi_sddf_net_init(void)
{
    if (!net_config_check_magic(&demi_sddf_net_config))
        return -1;
    net_queue_init(&rx, demi_sddf_net_config.rx.free_queue.vaddr,
                   demi_sddf_net_config.rx.active_queue.vaddr,
                   demi_sddf_net_config.rx.num_buffers);
    net_queue_init(&tx, demi_sddf_net_config.tx.free_queue.vaddr,
                   demi_sddf_net_config.tx.active_queue.vaddr,
                   demi_sddf_net_config.tx.num_buffers);
    if (tx.capacity > 4096)
        return -1;
    net_buffers_init(&tx, 0);
    return 0;
}

void demi_sddf_net_release(uint64_t offset)
{
    net_buff_desc_t desc = { .io_or_offset = offset, .len = 0, .oid = 0 };
    net_enqueue_free(&rx, desc);
    if (net_require_signal_free(&rx)) {
        net_cancel_signal_free(&rx);
        demi_sddf_notify(demi_sddf_net_config.rx.id);
    }
}

int demi_sddf_net_recv_lease(const uint8_t **data, uint64_t *offset)
{
    net_buff_desc_t desc;
    if (net_dequeue_active(&rx, &desc) != 0)
        return 0;
    if (desc.len == 0) {
        demi_sddf_net_release(desc.io_or_offset);
        return 0;
    }
    if (desc.len > NET_BUFFER_SIZE ||
        desc.io_or_offset > demi_sddf_net_config.rx_data.size ||
        desc.len > demi_sddf_net_config.rx_data.size - desc.io_or_offset) {
        demi_sddf_net_release(desc.io_or_offset);
        return -1;
    }
    *data = (uint8_t *)demi_sddf_net_config.rx_data.vaddr + desc.io_or_offset;
    *offset = desc.io_or_offset;
    return desc.len;
}

void demi_sddf_net_tx_release(uint64_t offset)
{
    if (tx_recycled_len == 4096)
        __builtin_trap();
    tx_recycled[tx_recycled_len++] = offset;
}

int demi_sddf_net_tx_acquire(uint8_t **data, uint64_t *offset)
{
    net_buff_desc_t desc;
    if (tx_recycled_len != 0) {
        desc = (net_buff_desc_t) { .io_or_offset = tx_recycled[--tx_recycled_len], .len = 0, .oid = 0 };
    } else if (net_dequeue_free(&tx, &desc) != 0) {
        return -1;
    }
    if (desc.io_or_offset > demi_sddf_net_config.tx_data.size ||
        NET_BUFFER_SIZE > demi_sddf_net_config.tx_data.size - desc.io_or_offset) {
        demi_sddf_net_tx_release(desc.io_or_offset);
        return -1;
    }
    *data = (uint8_t *)demi_sddf_net_config.tx_data.vaddr + desc.io_or_offset;
    *offset = desc.io_or_offset;
    return 0;
}

static void demi_sddf_net_tx_enqueue(uint64_t offset, size_t len)
{
    uint8_t *data = (uint8_t *)demi_sddf_net_config.tx_data.vaddr + offset;
#if CATLIONS_DROP_TCP
    if (drop_tcp_packet(data, len)) {
        demi_sddf_net_tx_release(offset);
        return;
    }
#endif
    net_buff_desc_t desc = { .io_or_offset = offset, .len = len, .oid = 0 };
    net_enqueue_active(&tx, desc);
    if (net_require_signal_active(&tx)) {
        net_cancel_signal_active(&tx);
        demi_sddf_notify(demi_sddf_net_config.tx.id);
    }
}

void demi_sddf_net_tx_submit(uint64_t offset, size_t len)
{
    demi_sddf_net_tx_enqueue(offset, len);
}

int demi_sddf_net_send(const void *data, size_t len)
{
#if CATLIONS_DROP_TCP
    if (drop_tcp_packet(data, len))
        return 0;
#endif
    uint8_t *tx_data;
    uint64_t offset;
    if (len > NET_BUFFER_SIZE || demi_sddf_net_tx_acquire(&tx_data, &offset) != 0)
        return -1;
    memcpy(tx_data, data, len);
    demi_sddf_net_tx_enqueue(offset, len);
    return 0;
}

int demi_sddf_net_arm_rx(void)
{
    net_request_signal_active(&rx);
    if (!net_queue_empty_active(&rx)) {
        net_cancel_signal_active(&rx);
        return 1;
    }
    return 0;
}

#undef memcmp
int memcmp(const void *left, const void *right, size_t len)
{
    const unsigned char *a = left, *b = right;
    for (size_t i = 0; i < len; i++) {
        if (a[i] != b[i])
            return (int)a[i] - (int)b[i];
    }
    return 0;
}

#undef strlen
size_t strlen(const char *text)
{
    size_t len = 0;
    while (text[len] != '\0')
        len++;
    return len;
}

__attribute__((section(".timer_client_config"), used))
timer_client_config_t demi_sddf_timer_config;

unsigned demi_sddf_timer_channel(void)
{
    return demi_sddf_timer_config.driver_id;
}

int demi_sddf_timer_init(void)
{
    return timer_config_check_magic(&demi_sddf_timer_config) ? 0 : -1;
}

