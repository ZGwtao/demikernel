// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

pub mod libc;

pub type AddressFamily = u16;
pub type Socklen = u32;
pub type SockAddrIn = libc::sockaddr_in;
pub type SockAddrIn6 = libc::sockaddr_in6;
pub type SockAddrStorage = libc::sockaddr_storage;
pub type Linger = libc::linger;
pub type KeepAlive = bool;

pub const AF_INET: AddressFamily = 2;
pub const AF_INET6: AddressFamily = 10;
pub const SOMAXCONN: i32 = 128;
pub const SOL_SOCKET: i32 = 1;
pub const SO_LINGER: i32 = 13;

pub fn socketaddrv4_to_sockaddr(addr: &core::net::SocketAddrV4) -> libc::sockaddr {
    let mut data = [0; 14];
    data[..2].copy_from_slice(&addr.port().to_be_bytes());
    data[2..6].copy_from_slice(&addr.ip().octets());
    libc::sockaddr {
        sa_family: AF_INET,
        sa_data: data,
    }
}
