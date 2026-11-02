use libc::sockaddr;

pub const CPU_DATA_CACHE_LINE_SIZE_IN_BYTES: usize = 64;
pub const AF_INET: libc::sa_family_t = libc::AF_INET as u16;
pub const AF_INET6: libc::sa_family_t = libc::AF_INET6 as u16;
pub const SOMAXCONN: i32 = libc::SOMAXCONN;
pub const SOL_SOCKET: i32 = libc::SOL_SOCKET;
pub const SO_LINGER: i32 = libc::SO_LINGER;
pub type SockAddrIn = libc::sockaddr_in;
pub type SockAddrIn6 = libc::sockaddr_in6;
pub type Socklen = libc::socklen_t;
pub type SockAddrStorage = libc::sockaddr_storage;
pub type AddressFamily = libc::sa_family_t;
pub type Linger = libc::linger;
pub type KeepAlive = bool;

pub fn socketaddrv4_to_sockaddr(addr: &core::net::SocketAddrV4) -> sockaddr {
    let mut data = [0; 14];
    data[..2].copy_from_slice(&addr.port().to_be_bytes());
    data[2..6].copy_from_slice(&addr.ip().octets());
    sockaddr { sa_family: AF_INET, sa_data: data }
}
