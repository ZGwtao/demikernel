use core::net::Ipv4Addr;

pub fn guest_ip() -> Ipv4Addr { env!("CATLIONS_GUEST_IP").parse().unwrap() }
pub fn udp_port() -> u16 { env!("CATLIONS_UDP_PORT").parse().unwrap() }
pub fn tcp_port() -> u16 { env!("CATLIONS_TCP_PORT").parse().unwrap() }
pub fn probe_port() -> u16 { env!("CATLIONS_PROBE_PORT").parse().unwrap() }
pub fn host_ip() -> Ipv4Addr { env!("CATLIONS_HOST_IP").parse().unwrap() }
pub fn host_tcp_port() -> u16 { env!("CATLIONS_HOST_TCP_PORT").parse().unwrap() }
pub fn host_refused_port() -> u16 { env!("CATLIONS_HOST_REFUSED_PORT").parse().unwrap() }
