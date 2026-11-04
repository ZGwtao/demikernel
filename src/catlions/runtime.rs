use crate::{
    demikernel::config::Config,
    inetstack::{consts::{MAX_BATCH_SIZE_NUM_PACKETS, MAX_HEADER_SIZE}, protocols::layer1::PhysicalLayer},
    runtime::{fail::Fail, memory::{DemiBuffer, DemiMemoryAllocator}},
};
use arrayvec::ArrayVec;

pub struct SddfRuntime;

impl SddfRuntime {
    pub fn new(_config: &Config) -> Result<Self, Fail> {
        if sddf_rs::net_init() != 0 {
            return Err(Fail::new(libc::EIO, "sDDF network config is invalid"));
        }
        Ok(Self)
    }
}

impl DemiMemoryAllocator for SddfRuntime {
    fn max_buffer_size_bytes(&self) -> usize {
        sddf_rs::net_buffer_size() - MAX_HEADER_SIZE
    }

    // Copied from catpower/linux/mod.rs, like it be like this...
    fn allocate_demi_buffer(&self, size: usize) -> Result<DemiBuffer, Fail> {
        if size > self.max_buffer_size_bytes() {
            return Err(Fail::new(libc::EINVAL, "packet exceeds sDDF buffer"));
        }
        Ok(DemiBuffer::new_with_headroom(size as u16, MAX_HEADER_SIZE as u16))
    }
}

impl PhysicalLayer for SddfRuntime {
    fn transmit(&mut self, packets: ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>) -> Result<(), Fail> {
        for packet in packets {
            if sddf_rs::net_send(&packet) != 0 {
                return Err(Fail::new(libc::EAGAIN, "sDDF TX queue is full"));
            }
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>, Fail> {
        let mut packets = ArrayVec::new();
        for _ in 0..MAX_BATCH_SIZE_NUM_PACKETS {
            let buffer_size = sddf_rs::net_buffer_size();
            let mut packet = DemiBuffer::new(buffer_size as u16);
            // @gt Copy the pkts from the RX buffers to DemiBuffers
            let len = sddf_rs::net_recv(&mut packet);
            if len < 0 {
                return Err(Fail::new(libc::EIO, "invalid sDDF RX descriptor"));
            }
            if len == 0 {
                break;
            }
            packet.trim(buffer_size - len as usize)?;
            packets.push(packet);
        }
        Ok(packets)
    }
}
