// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::{pal::KeepAlive, runtime::fail::Fail, MacAddress};
use core::{net::Ipv4Addr, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub local_ipv4_addr: Ipv4Addr,
    pub local_link_addr: MacAddress,
    pub tcp_keepalive: KeepAlive,
    pub linger: Option<Duration>,
    pub no_delay: bool,
    pub mtu: u16,
    pub mss: usize,
    pub tcp_checksum_offload: bool,
    pub udp_checksum_offload: bool,
    pub enable_jumbo_frames: bool,
}

impl Config {
    pub fn new(_config_path: String) -> Result<Self, Fail> {
        Err(Fail::new(libc::ENOSYS, "Catlions has no file-based configuration"))
    }

    pub fn local_ipv4_addr(&self) -> Result<Ipv4Addr, Fail> {
        if self.local_ipv4_addr.is_unspecified() || self.local_ipv4_addr.is_broadcast() {
            return Err(Fail::new(libc::EINVAL, "Invalid IPv4 address"));
        }
        Ok(self.local_ipv4_addr)
    }

    pub fn local_link_addr(&self) -> Result<MacAddress, Fail> {
        Ok(self.local_link_addr)
    }

    pub fn tcp_keepalive(&self) -> Result<KeepAlive, Fail> {
        Ok(self.tcp_keepalive)
    }

    pub fn linger(&self) -> Result<Option<Duration>, Fail> {
        Ok(self.linger)
    }

    pub fn no_delay(&self) -> Result<bool, Fail> {
        Ok(self.no_delay)
    }

    pub fn mtu(&self) -> Result<u16, Fail> {
        Ok(self.mtu)
    }

    pub fn mss(&self) -> Result<usize, Fail> {
        Ok(self.mss)
    }

    pub fn tcp_checksum_offload(&self) -> Result<bool, Fail> {
        Ok(self.tcp_checksum_offload)
    }

    pub fn udp_checksum_offload(&self) -> Result<bool, Fail> {
        Ok(self.udp_checksum_offload)
    }

    pub fn enable_jumbo_frames(&self) -> Result<bool, Fail> {
        Ok(self.enable_jumbo_frames)
    }
}
