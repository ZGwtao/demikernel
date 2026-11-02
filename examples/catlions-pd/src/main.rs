#![no_std]
#![no_main]

extern crate alloc;

mod probe;
mod config;

use alloc::{string::String, vec::Vec};
use core::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};
use demikernel::{LibOS, QDesc, QToken, catlions::runtime::{Application, EventLoop}, runtime::types::demi_opcode_t};
use log::{LevelFilter, Log, Metadata, Record};
use sel4_microkit::{Handler, debug_println, protection_domain};

struct MicrokitLogger;
static LOGGER: MicrokitLogger = MicrokitLogger;

impl Log for MicrokitLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        debug_println!("{}: {}", record.level(), record.args());
    }
    fn flush(&self) {}
}

const CLIENT_TEST: bool = cfg!(catlions_connect_test);
const CLIENT_REFUSED: bool = cfg!(catlions_connect_refused);
const PROBE_APP: bool = cfg!(catlions_probe_app);
const CLIENT_MESSAGE: &[u8] = b"catlions active open";

#[protection_domain(heap_size = 0x1000000)]
fn init() -> impl Handler {
    demikernel::catlions::runtime::init();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let config = demikernel::demikernel::config::Config::new(String::new())
        .unwrap()
        .with_local_ipv4_addr(config::guest_ip());
    let mut libos = LibOS::new_with_config(demikernel::LibOSName::Catlions, None, config).unwrap();
    if PROBE_APP {
        return EventLoop::new(App::Probe(probe::Probe::new(libos)));
    }
    let qd = libos.socket(2, 2, 0).unwrap();
    libos
        .bind(
            qd,
            SocketAddr::V4(SocketAddrV4::new(config::guest_ip(), config::udp_port())),
        )
        .unwrap();
    let qt = libos.pop(qd, None).unwrap();
    log::info!("catlions: original inetstack UDP echo ready");
    let tcp_listener = libos.socket(2, 1, 0).unwrap();
    libos
        .bind(
            tcp_listener,
            SocketAddr::V4(SocketAddrV4::new(config::guest_ip(), config::tcp_port())),
        )
        .unwrap();
    libos.listen(tcp_listener, 8).unwrap();
    let tcp_accept = libos.accept(tcp_listener).unwrap();
    log::info!("catlions: original inetstack TCP echo ready");
    let outgoing = if CLIENT_TEST {
        Some(Outgoing::connect(&mut libos, 0))
    } else {
        None
    };
    let echo = Echo {
        libos,
        qd,
        qt,
        tcp_listener,
        tcp_accept,
        tcp_clients: Vec::new(),
        outgoing,
    };
    EventLoop::new(App::Echo(echo))
}

enum App {
    Echo(Echo),
    Probe(probe::Probe),
}

impl Application for App {
    fn poll(&mut self) {
        match self {
            Self::Echo(app) => app.poll(),
            Self::Probe(app) => app.poll(),
        }
    }
}

#[derive(Clone, Copy)]
enum TcpState {
    Pop,
    Push,
    Close,
}

struct TcpClient {
    qd: QDesc,
    token: QToken,
    state: TcpState,
}

#[derive(Clone, Copy)]
enum OutgoingState {
    Connect,
    Push,
    Pop,
    Close,
}

struct Outgoing {
    qd: QDesc,
    token: QToken,
    state: OutgoingState,
    round: u8,
    failed: bool,
}

impl Outgoing {
    fn connect(libos: &mut LibOS, round: u8) -> Self {
        let qd = libos.socket(2, 1, 0).unwrap();
        let port = if CLIENT_REFUSED { config::host_refused_port() } else { config::host_tcp_port() };
        let remote = SocketAddr::V4(SocketAddrV4::new(config::host_ip(), port));
        let token = libos.connect(qd, remote).unwrap();
        log::info!("catlions: active TCP connect round {}", round + 1);
        Self { qd, token, state: OutgoingState::Connect, round, failed: false }
    }
}

struct Echo {
    libos: LibOS,
    qd: QDesc,
    qt: QToken,
    tcp_listener: QDesc,
    tcp_accept: QToken,
    tcp_clients: Vec<TcpClient>,
    outgoing: Option<Outgoing>,
}

impl Echo {
    fn drain_outgoing(&mut self) {
        let Some(client) = &mut self.outgoing else { return };
        let result = match self.libos.wait(client.token, Some(Duration::ZERO)) {
            Ok(result) => result,
            Err(error) if error.errno == 110 => return,
            Err(_) => panic!("Demikernel outgoing wait failed"),
        };
        match client.state {
            OutgoingState::Connect if result.qr_opcode == demi_opcode_t::DEMI_OPC_CONNECT => {
                let sga = self.libos.sgaalloc(CLIENT_MESSAGE.len()).unwrap();
                unsafe {
                    core::ptr::copy_nonoverlapping(CLIENT_MESSAGE.as_ptr(), sga.segments[0].data_buf_ptr as *mut u8, CLIENT_MESSAGE.len());
                }
                client.token = self.libos.push(client.qd, &sga).unwrap();
                self.libos.sgafree(sga).unwrap();
                client.state = OutgoingState::Push;
            }
            OutgoingState::Connect if result.qr_opcode == demi_opcode_t::DEMI_OPC_FAILED => {
                log::info!("catlions: active TCP connect failed with {}", result.qr_ret);
                client.failed = true;
                client.token = self.libos.async_close(client.qd).unwrap();
                client.state = OutgoingState::Close;
            }
            OutgoingState::Push if result.qr_opcode == demi_opcode_t::DEMI_OPC_PUSH => {
                client.token = self.libos.pop(client.qd, None).unwrap();
                client.state = OutgoingState::Pop;
            }
            OutgoingState::Pop if result.qr_opcode == demi_opcode_t::DEMI_OPC_POP => {
                let sga = unsafe { result.qr_value.sga };
                let segment = sga.segments[0];
                let bytes = unsafe { core::slice::from_raw_parts(segment.data_buf_ptr as *const u8, segment.data_len_bytes as usize) };
                assert_eq!(bytes, CLIENT_MESSAGE);
                self.libos.sgafree(sga).unwrap();
                log::info!("catlions: active TCP echo round {} passed", client.round + 1);
                client.token = self.libos.async_close(client.qd).unwrap();
                client.state = OutgoingState::Close;
            }
            OutgoingState::Close if result.qr_opcode == demi_opcode_t::DEMI_OPC_CLOSE => {
                if client.failed {
                    log::info!("catlions: failed active TCP connection closed");
                    self.outgoing = None;
                    return;
                }
                let next = client.round + 1;
                self.outgoing = if next < 2 { Some(Outgoing::connect(&mut self.libos, next)) } else { None };
                if next == 2 { log::info!("catlions: active TCP connect/reconnect passed"); }
            }
            _ => panic!("Demikernel outgoing operation failed: {:?} {}", result.qr_opcode, result.qr_ret),
        }
    }
    fn drain(&mut self) {
        loop {
            match self.libos.wait(self.qt, Some(Duration::ZERO)) {
                Ok(result) => match result.qr_opcode {
                    demi_opcode_t::DEMI_OPC_POP => {
                        let sga = unsafe { result.qr_value.sga };
                        let sender = sga.sockaddr_src;
                        let port = u16::from_be_bytes([sender.sa_data[0], sender.sa_data[1]]);
                        let ip = Ipv4Addr::new(
                            sender.sa_data[2],
                            sender.sa_data[3],
                            sender.sa_data[4],
                            sender.sa_data[5],
                        );
                        let remote = SocketAddr::V4(SocketAddrV4::new(ip, port));
                        let len = sga.segments[0].data_len_bytes as usize;
                        let reply = self.libos.sgaalloc(len).unwrap();
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                sga.segments[0].data_buf_ptr as *const u8,
                                reply.segments[0].data_buf_ptr as *mut u8,
                                len,
                            );
                        }
                        self.qt = self.libos.pushto(self.qd, &reply, remote).unwrap();
                        self.libos.sgafree(reply).unwrap();
                        self.libos.sgafree(sga).unwrap();
                    }
                    demi_opcode_t::DEMI_OPC_PUSH => {
                        self.qt = self.libos.pop(self.qd, None).unwrap();
                    }
                    _ => panic!("Demikernel operation failed"),
                },
                Err(error) if error.errno == 110 => break,
                Err(_) => panic!("Demikernel wait failed"),
            }
        }
    }

    fn drain_tcp(&mut self) {
        loop {
            let mut tokens = Vec::with_capacity(self.tcp_clients.len() + 1);
            tokens.push(self.tcp_accept);
            tokens.extend(self.tcp_clients.iter().map(|client| client.token));
            match self.libos.wait_any(&tokens, Some(Duration::ZERO)) {
                Ok((0, result)) if result.qr_opcode == demi_opcode_t::DEMI_OPC_ACCEPT => {
                    let qd = unsafe { result.qr_value.ares.qd.into() };
                    let token = self.libos.pop(qd, None).unwrap();
                    self.tcp_clients.push(TcpClient {
                        qd,
                        token,
                        state: TcpState::Pop,
                    });
                    self.tcp_accept = self.libos.accept(self.tcp_listener).unwrap();
                    log::info!("catlions: TCP client accepted");
                }
                Ok((0, result))
                    if result.qr_opcode == demi_opcode_t::DEMI_OPC_FAILED
                        && result.qr_ret == 110 =>
                {
                    log::warn!("catlions: TCP accept failed; retrying");
                    sddf::clear_drop();
                    self.tcp_accept = self.libos.accept(self.tcp_listener).unwrap();
                }
                Ok((index, result)) if index > 0 => {
                    let client = &mut self.tcp_clients[index - 1];
                    match client.state {
                        TcpState::Pop if result.qr_opcode == demi_opcode_t::DEMI_OPC_POP => {
                            let sga = unsafe { result.qr_value.sga };
                            if sga.segments[0].data_len_bytes == 0 {
                                self.libos.sgafree(sga).unwrap();
                                client.token = self.libos.async_close(client.qd).unwrap();
                                client.state = TcpState::Close;
                            } else {
                                client.token = self.libos.push(client.qd, &sga).unwrap();
                                self.libos.sgafree(sga).unwrap();
                                client.state = TcpState::Push;
                            }
                        }
                        TcpState::Push if result.qr_opcode == demi_opcode_t::DEMI_OPC_PUSH => {
                            client.token = self.libos.pop(client.qd, None).unwrap();
                            client.state = TcpState::Pop;
                        }
                        TcpState::Close if result.qr_opcode == demi_opcode_t::DEMI_OPC_CLOSE => {
                            self.tcp_clients.swap_remove(index - 1);
                        }
                        TcpState::Pop if result.qr_opcode == demi_opcode_t::DEMI_OPC_FAILED => {
                            client.token = self.libos.async_close(client.qd).unwrap();
                            client.state = TcpState::Close;
                        }
                        _ => panic!("Demikernel TCP operation failed"),
                    }
                }
                Ok(_) => panic!("Demikernel TCP accept failed"),
                Err(error) if error.errno == 110 => break,
                Err(_) => panic!("Demikernel TCP wait failed"),
            }
        }
    }

}

impl Application for Echo {
    fn poll(&mut self) {
        self.drain();
        self.drain_tcp();
        self.drain_outgoing();
    }
}
