use crate::{
    demikernel::config::Config,
    inetstack::{consts::{MAX_BATCH_SIZE_NUM_PACKETS, MAX_HEADER_SIZE}, protocols::layer1::PhysicalLayer},
    runtime::{fail::Fail, memory::{DemiBuffer, DemiMemoryAllocator}},
};
use arrayvec::ArrayVec;
use sddf_rs::{net_buffer_size, net_init, net_send_copy, rx_acquire, tx_acquire, SBuf};

pub struct SddfRuntime;

impl SddfRuntime {
    pub fn new(_config: &Config) -> Result<Self, Fail> {
        if net_init() != 0 {
            return Err(Fail::new(libc::EIO, "sDDF network config is invalid"));
        }
        Ok(Self)
    }

    fn sddf_allocate_tx_buffer(&self, size: usize, headroom: usize) -> Result<DemiBuffer, Fail> {
        if headroom > net_buffer_size() || size > net_buffer_size() - headroom {
            return Err(Fail::new(libc::EINVAL, "payload and headroom exceed sDDF buffer"));
        }
        let tx = tx_acquire().ok_or_else(|| Fail::new(libc::EAGAIN, "sDDF TX queue is full"))?;
        let metadata = SBuf::tx(tx, size as u16, headroom as u16);
        Ok(unsafe { DemiBuffer::from_sddf(metadata) })
    }

    fn try_publish_sddf(packet: DemiBuffer) -> Result<(), DemiBuffer> {
        if !packet.is_sddf_allocated() || packet.headroom() != 0 {
            return Err(packet);
        }
        let len = packet.len();
        let metadata = packet.into_sbuf().unwrap();
        if !unsafe { &*metadata }.is_tx() {
            return Err(unsafe { DemiBuffer::from_sddf(metadata) });
        }
        if unsafe { &mut *metadata }.try_publish(len) {
            unsafe { SBuf::drop_raw(metadata) };
            Ok(())
        } else {
            Err(unsafe { DemiBuffer::from_sddf(metadata) })
        }
    }
}

impl DemiMemoryAllocator for SddfRuntime {
    fn max_buffer_size_bytes(&self) -> usize {
        net_buffer_size() - MAX_HEADER_SIZE
    }

    fn max_buffer_size_bytes_with_headroom(&self, headroom: usize) -> usize {
        net_buffer_size().saturating_sub(headroom)
    }

    fn allocate_demi_buffer(&self, size: usize) -> Result<DemiBuffer, Fail> {
        self.sddf_allocate_tx_buffer(size, MAX_HEADER_SIZE)
    }

    fn allocate_demi_buffer_with_headroom(&self, size: usize, headroom: usize) -> Result<DemiBuffer, Fail> {
        self.sddf_allocate_tx_buffer(size, headroom)
    }
}

impl PhysicalLayer for SddfRuntime {
    fn transmit(&mut self, packets: ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>) -> Result<(), Fail> {
        for packet in packets {
            let packet = match Self::try_publish_sddf(packet) {
                Ok(()) => continue,
                Err(packet) => packet,
            };
            if net_send_copy(&packet) != 0 {
                return Err(Fail::new(libc::EAGAIN, "sDDF TX queue is full"));
            }
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<ArrayVec<DemiBuffer, MAX_BATCH_SIZE_NUM_PACKETS>, Fail> {
        let mut packets = ArrayVec::new();
        for _ in 0..MAX_BATCH_SIZE_NUM_PACKETS {
            match rx_acquire() {
                Ok(Some(buffer)) => {
                    let metadata = SBuf::rx(buffer);
                    packets.push(unsafe { DemiBuffer::from_sddf(metadata) });
                },
                Ok(None) => break,
                Err(()) => return Err(Fail::new(libc::EIO, "invalid sDDF RX descriptor")),
            }
        }
        Ok(packets)
    }
}
