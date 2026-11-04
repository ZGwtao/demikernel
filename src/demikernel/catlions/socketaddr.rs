// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::{pal::Socklen, runtime::fail::Fail};
use core::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    ptr,
};
use libc::sockaddr;

pub const SOCKADDR_IN_LEN: usize = 16;
const AF_INET: u16 = 2;

pub fn sockaddr_to_socketaddr(addr: *const sockaddr, len: Socklen) -> Result<SocketAddr, Fail> {
    if addr.is_null() || len as usize != SOCKADDR_IN_LEN {
        return Err(Fail::new(libc::EINVAL, "bad socket name length"));
    }

    let mut bytes = [0u8; SOCKADDR_IN_LEN];
    unsafe { ptr::copy_nonoverlapping(addr.cast::<u8>(), bytes.as_mut_ptr(), bytes.len()) };

    if u16::from_ne_bytes([bytes[0], bytes[1]]) != AF_INET {
        return Err(Fail::new(libc::ENOTSUP, "communication domain not supported"));
    }

    let ip = Ipv4Addr::new(bytes[4], bytes[5], bytes[6], bytes[7]);
    let port = u16::from_be_bytes([bytes[2], bytes[3]]);

    Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
}

pub fn socketaddrv4_to_bytes(addr: SocketAddrV4) -> [u8; SOCKADDR_IN_LEN] {
    let mut bytes = [0u8; SOCKADDR_IN_LEN];
    bytes[..2].copy_from_slice(&AF_INET.to_ne_bytes());
    bytes[2..4].copy_from_slice(&addr.port().to_be_bytes());
    bytes[4..8].copy_from_slice(&addr.ip().octets());
    bytes
}
