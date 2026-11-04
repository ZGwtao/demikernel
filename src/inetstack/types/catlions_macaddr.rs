// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::runtime::fail::Fail;
use core::{fmt, str::FromStr};
use libc::EINVAL;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.try_into().unwrap())
    }

    pub fn octets(&self) -> [u8; 6] {
        self.0
    }

    pub fn broadcast() -> Self {
        Self([0xff; 6])
    }

    pub fn nil() -> Self {
        Self([0; 6])
    }

    pub fn is_nil(self) -> bool {
        self.0 == [0; 6]
    }

    pub fn is_broadcast(self) -> bool {
        self.0 == [0xff; 6]
    }

    pub fn is_multicast(self) -> bool {
        self.0[0] & 1 == 1
    }

    pub fn is_unicast(self) -> bool {
        self.0[0] & 1 == 0
    }

    pub fn to_canonical(self) -> String {
        format!(
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse_canonical_str(value: &str) -> Result<Self, Fail> {
        let invalid = || Fail::new(EINVAL, "failed to parse MAC Address");
        let mut bytes = [0; 6];
        if value.len() == 12 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            for (index, byte) in bytes.iter_mut().enumerate() {
                *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| invalid())?;
            }
            return Ok(Self(bytes));
        }
        let mut groups = value.split([':', '-', '.']);
        for byte in &mut bytes {
            let group = groups.next().ok_or_else(invalid)?;
            if group.is_empty() || group.len() > 2 {
                return Err(invalid());
            }
            *byte = u8::from_str_radix(group, 16).map_err(|_| invalid())?;
        }
        if groups.next().is_some() {
            return Err(invalid());
        }
        Ok(Self(bytes))
    }

    pub fn to_array(self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_canonical())
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MacAddress({})", self.to_canonical())
    }
}

impl FromStr for MacAddress {
    type Err = Fail;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_canonical_str(value)
    }
}
