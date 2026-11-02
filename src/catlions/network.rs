use crate::{
    demikernel::config::Config,
    inetstack::{consts::{MAX_BATCH_SIZE_NUM_PACKETS, MAX_HEADER_SIZE}, protocols::layer1::PhysicalLayer},
    runtime::{fail::Fail, memory::{DemiBuffer, DemiMemoryAllocator}, SharedObject},
};
use arrayvec::ArrayVec;

struct SddfNetwork;

#[derive(Clone)]
pub struct SharedSddfRuntime(SharedObject<SddfNetwork>);

impl SharedSddfRuntime {
    pub fn new(_config: &Config) -> Result<Self, Fail> {
        Ok(Self(SharedObject::new(SddfNetwork)))
    }
}

impl DemiMemoryAllocator for SharedSddfRuntime {
    fn max_buffer_size_bytes(&self) -> usize {
        2048 - MAX_HEADER_SIZE
    }

    fn allocate_demi_buffer(&self, size: usize) -> Result<DemiBuffer, Fail> {
        if size > self.max_buffer_size_bytes() {
            return Err(Fail::new(libc::EINVAL, "packet exceeds SDDF TX buffer"));
        }
        Ok(DemiBuffer::new_with_headroom(size as u16, MAX_HEADER_SIZE as u16))
    }
}

impl PhysicalLayer for SharedSddfRuntime {
    fn transmit(&mut self, packets: ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>) -> Result<(), Fail> {
        for packet in packets {
            match packet.into_tx_descriptor() {
                Ok((offset, len)) => sddf::tx_submit(offset, len),
                Err(packet) => {
                    let rc = sddf::net_send(&packet);
                    if rc < 0 { return Err(Fail::new(libc::EAGAIN, "SDDF TX queue full")); }
                },
            }
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>, Fail> {
        let mut packets = ArrayVec::new();
        for _ in 0..MAX_BATCH_SIZE_NUM_PACKETS {
            let Some((data, len, offset)) = sddf::net_recv_lease()
                .map_err(|_| Fail::new(libc::EIO, "invalid SDDF RX descriptor"))? else { break };
            let packet = unsafe {
                DemiBuffer::from_external(
                    data,
                    len,
                    offset,
                    sddf::net_release,
                )
            };
            packets.push(packet);
        }
        Ok(packets)
    }
}
