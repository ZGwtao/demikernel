use rust_alloc::string::String;
use core::{net::Ipv4Addr, time::Duration};
use hashbrown::HashMap;

use crate::{inetstack::types::MacAddress, runtime::fail::Fail};

#[derive(Clone, Debug)]
pub struct Config {
    ip: Ipv4Addr,
}

impl Config {
    pub fn new(_path: String) -> Result<Self, Fail> {
        Ok(Self { ip: Ipv4Addr::UNSPECIFIED })
    }

    pub fn with_local_ipv4_addr(mut self, ip: Ipv4Addr) -> Self {
        self.ip = ip;
        self
    }

    pub fn local_ipv4_addr(&self) -> Result<Ipv4Addr, Fail> { Ok(self.ip) }
    pub fn local_link_addr(&self) -> Result<MacAddress, Fail> {
        Ok(MacAddress::new(sddf::net_mac()))
    }
    pub fn arp_table(&self) -> Result<Option<HashMap<Ipv4Addr, MacAddress>>, Fail> {
        Ok(Some(HashMap::new()))
    }
    pub fn arp_cache_ttl(&self) -> Result<Duration, Fail> { Ok(Duration::from_secs(15)) }
    pub fn arp_request_timeout(&self) -> Result<Duration, Fail> { Ok(Duration::from_secs(20)) }
    pub fn arp_request_retries(&self) -> Result<usize, Fail> { Ok(5) }
    pub fn mtu(&self) -> Result<u16, Fail> { Ok(1500) }
    pub fn mss(&self) -> Result<usize, Fail> { Ok(1460) }
    pub fn tcp_checksum_offload(&self) -> Result<bool, Fail> { Ok(false) }
    pub fn udp_checksum_offload(&self) -> Result<bool, Fail> { Ok(false) }
    pub fn enable_jumbo_frames(&self) -> Result<bool, Fail> { Ok(false) }
    pub fn no_delay(&self) -> Result<bool, Fail> { Ok(false) }
    pub fn linger(&self) -> Result<Option<Duration>, Fail> { Ok(None) }
    pub fn tcp_keepalive(&self) -> Result<bool, Fail> { Ok(false) }
}
