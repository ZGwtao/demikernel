use demikernel::catlions::runtime::Application;
use core::{net::{Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};
use crate::config;
use demikernel::{LibOS, QDesc, QToken, runtime::types::demi_opcode_t};

pub struct Probe {
    libos: LibOS,
    qd: QDesc,
    token: QToken,
    sending: bool,
}

impl Probe {
    pub fn new(mut libos: LibOS) -> Self {
        let qd = libos.socket(2, 2, 0).unwrap();
        libos.bind(qd, SocketAddr::V4(SocketAddrV4::new(config::guest_ip(), config::probe_port()))).unwrap();
        let token = libos.pop(qd, None).unwrap();
        log::info!("catlions: standalone UDP probe ready");
        Self { libos, qd, token, sending: false }
    }
}

impl Application for Probe {
    fn poll(&mut self) {
        loop {
            let result = match self.libos.wait(self.token, Some(Duration::ZERO)) {
                Ok(result) => result,
                Err(error) if error.errno == 110 => return,
                Err(_) => panic!("probe wait failed"),
            };
            if self.sending {
                assert_eq!(result.qr_opcode, demi_opcode_t::DEMI_OPC_PUSH);
                self.token = self.libos.pop(self.qd, None).unwrap();
                self.sending = false;
                continue;
            }
            assert_eq!(result.qr_opcode, demi_opcode_t::DEMI_OPC_POP);
            let sga = unsafe { result.qr_value.sga };
            let sender = sga.sockaddr_src;
            let port = u16::from_be_bytes([sender.sa_data[0], sender.sa_data[1]]);
            let ip = Ipv4Addr::new(sender.sa_data[2], sender.sa_data[3], sender.sa_data[4], sender.sa_data[5]);
            let remote = SocketAddr::V4(SocketAddrV4::new(ip, port));
            let len = sga.segments[0].data_len_bytes as usize;
            let reply = self.libos.sgaalloc(len).unwrap();
            let input = unsafe { core::slice::from_raw_parts(sga.segments[0].data_buf_ptr as *const u8, len) };
            let output = unsafe { core::slice::from_raw_parts_mut(reply.segments[0].data_buf_ptr as *mut u8, len) };
            for (dst, src) in output.iter_mut().zip(input) { *dst = src.to_ascii_uppercase(); }
            self.token = self.libos.pushto(self.qd, &reply, remote).unwrap();
            self.libos.sgafree(reply).unwrap();
            self.libos.sgafree(sga).unwrap();
            self.sending = true;
            log::info!("catlions: standalone UDP probe answered");
        }
    }
}
