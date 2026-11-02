#![no_std]

use core::{
    ops::{Add, Sub},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use sel4_microkit::{Channel, MessageInfo, debug_println, get_mr, set_mr};

#[unsafe(no_mangle)]
extern "C" fn demi_sddf_notify(channel: u32) {
    Channel::new(channel as usize).notify();
}

#[unsafe(no_mangle)]
extern "C" fn demi_sddf_fault_drop(kind: u32) {
    debug_println!("catlions: injected TCP drop {}", kind);
}

unsafe extern "C" {
    fn demi_sddf_net_init() -> i32;
    fn demi_sddf_net_arm_rx() -> i32;
    fn demi_sddf_net_mac() -> *const u8;
    fn demi_sddf_net_send(data: *const u8, len: usize) -> i32;
    fn demi_sddf_net_recv_lease(data: *mut *const u8, offset: *mut u64) -> i32;
    fn demi_sddf_net_release(offset: u64);
    fn demi_sddf_net_tx_acquire(data: *mut *mut u8, offset: *mut u64) -> i32;
    fn demi_sddf_net_tx_release(offset: u64);
    fn demi_sddf_net_tx_submit(offset: u64, len: usize);
    fn demi_sddf_net_clear_drop();
    fn demi_sddf_timer_init() -> i32;
    fn demi_sddf_timer_channel() -> u32;
}

pub fn net_init() -> i32 { unsafe { demi_sddf_net_init() } }
pub fn net_arm_rx() -> bool { unsafe { demi_sddf_net_arm_rx() != 0 } }
pub fn net_mac() -> [u8; 6] {
    let data = unsafe { core::slice::from_raw_parts(demi_sddf_net_mac(), 6) };
    data.try_into().unwrap()
}
pub fn net_send(data: &[u8]) -> i32 { unsafe { demi_sddf_net_send(data.as_ptr(), data.len()) } }
pub fn net_recv_lease() -> Result<Option<(NonNull<u8>, u16, u64)>, ()> {
    let mut data = core::ptr::null();
    let mut offset = 0;
    let len = unsafe { demi_sddf_net_recv_lease(&mut data, &mut offset) };
    if len < 0 { return Err(()); }
    if len == 0 { return Ok(None); }
    Ok(Some((NonNull::new(data as *mut u8).unwrap(), len as u16, offset)))
}
pub unsafe extern "C" fn net_release(offset: u64) { unsafe { demi_sddf_net_release(offset) } }
pub fn tx_acquire() -> Option<(NonNull<u8>, u64)> {
    let mut data = core::ptr::null_mut();
    let mut offset = 0;
    (unsafe { demi_sddf_net_tx_acquire(&mut data, &mut offset) } == 0)
        .then(|| (NonNull::new(data).unwrap(), offset))
}
pub unsafe extern "C" fn tx_release(offset: u64) { unsafe { demi_sddf_net_tx_release(offset) } }
pub fn tx_submit(offset: u64, len: usize) { unsafe { demi_sddf_net_tx_submit(offset, len) } }
pub fn clear_drop() { unsafe { demi_sddf_net_clear_drop() } }
pub fn timer_init() -> i32 { unsafe { demi_sddf_timer_init() } }
pub fn timer_channel() -> usize { (unsafe { demi_sddf_timer_channel() }) as usize }

static CHANNEL: AtomicUsize = AtomicUsize::new(usize::MAX);

pub fn init(channel: usize) {
    CHANNEL.store(channel, Ordering::Relaxed);
}

fn channel() -> Channel {
    Channel::new(CHANNEL.load(Ordering::Relaxed))
}

pub fn arm_after(delay: Duration) {
    set_mr(0, delay.as_nanos() as u64);
    channel().pp_call(MessageInfo::new(1, 1));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(u64);

impl Instant {
    pub fn now() -> Self {
        channel().pp_call(MessageInfo::new(0, 0));
        Self(get_mr(0))
    }
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
    pub fn duration_since(self, earlier: Self) -> Duration {
        Duration::from_nanos(self.0 - earlier.0)
    }
    pub fn elapsed(self) -> Duration {
        Self::now() - self
    }
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub(earlier.0).map(Duration::from_nanos)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        Self(self.0 + rhs.as_nanos() as u64)
    }
}
impl Sub<Duration> for Instant {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        Self(self.0 - rhs.as_nanos() as u64)
    }
}
impl Sub<Self> for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        self.duration_since(rhs)
    }
}
