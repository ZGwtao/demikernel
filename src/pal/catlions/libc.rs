// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

pub use core::ffi::{c_int, c_void};

#[allow(non_camel_case_types)]
pub type size_t = usize;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct in_addr {
    pub s_addr: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr_in {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: in_addr,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr_in6 {
    pub sin6_family: u16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: [u8; 16],
    pub sin6_scope_id: u32,
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct sockaddr_storage(pub [u8; 128]);

const _: () = assert!(core::mem::size_of::<sockaddr>() == 16);
const _: () = assert!(core::mem::size_of::<sockaddr_in>() == 16);
const _: () = assert!(core::mem::size_of::<sockaddr_in6>() == 28);
const _: () = assert!(core::mem::size_of::<sockaddr_storage>() == 128);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct linger {
    pub l_onoff: c_int,
    pub l_linger: c_int,
}

pub const EADDRINUSE: c_int = 98;
pub const EADDRNOTAVAIL: c_int = 99;
pub const EAGAIN: c_int = 11;
pub const EALREADY: c_int = 114;
pub const EBADF: c_int = 9;
pub const EBADMSG: c_int = 74;
pub const EBUSY: c_int = 16;
pub const ECONNABORTED: c_int = 103;
pub const ECONNREFUSED: c_int = 111;
pub const ECONNRESET: c_int = 104;
pub const EDESTADDRREQ: c_int = 89;
pub const EFAULT: c_int = 14;
pub const EINPROGRESS: c_int = 115;
pub const EINVAL: c_int = 22;
pub const EIO: c_int = 5;
pub const EISCONN: c_int = 106;
pub const ENOENT: c_int = 2;
pub const ENOPROTOOPT: c_int = 92;
pub const ENOSYS: c_int = 38;
pub const ENOTCONN: c_int = 107;
pub const ENOTSUP: c_int = 95;
pub const EOPNOTSUPP: c_int = ENOTSUP;
pub const ERANGE: c_int = 34;
pub const ETIMEDOUT: c_int = 110;
pub const EWOULDBLOCK: c_int = EAGAIN;
