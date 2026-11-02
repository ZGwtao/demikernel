use rust_alloc::string::String;
use core::{fmt, str::FromStr};

use crate::runtime::fail::Fail;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self { Self(bytes) }
    pub fn from_bytes(bytes: &[u8]) -> Self { Self(bytes.try_into().unwrap()) }
    pub const fn octets(&self) -> [u8; 6] { self.0 }
    pub const fn broadcast() -> Self { Self([0xff; 6]) }
    pub const fn nil() -> Self { Self([0; 6]) }
    pub fn is_nil(self) -> bool { self.0 == [0; 6] }
    pub fn is_broadcast(self) -> bool { self.0 == [0xff; 6] }
    pub fn is_multicast(self) -> bool { self.0[0] & 1 != 0 }
    pub fn is_unicast(self) -> bool { !self.is_multicast() && !self.is_nil() }
    pub fn to_canonical(self) -> String { rust_alloc::format!("{self}") }
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
    pub const fn to_array(self) -> [u8; 6] { self.0 }
    pub fn parse_canonical_str(text: &str) -> Result<Self, Fail> {
        let mut bytes = [0; 6];
        let mut parts = text.split([':', '-']);
        for byte in &mut bytes {
            *byte = u8::from_str_radix(parts.next().ok_or(Fail::new(libc::EINVAL, "invalid MAC address"))?, 16)
                .map_err(|_| Fail::new(libc::EINVAL, "invalid MAC address"))?;
        }
        if parts.next().is_some() { return Err(Fail::new(libc::EINVAL, "invalid MAC address")); }
        Ok(Self(bytes))
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0;
        write!(f, "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}", b[0], b[1], b[2], b[3], b[4], b[5])
    }
}
impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "MacAddress({self})") }
}
impl FromStr for MacAddress {
    type Err = Fail;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::parse_canonical_str(s) }
}
