// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Domain(i32);

impl Domain {
    pub const IPV4: Self = Self(2);
}

impl From<i32> for Domain {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Type(i32);

impl Type {
    pub const STREAM: Self = Self(1);
    pub const DGRAM: Self = Self(2);
}

impl From<i32> for Type {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Protocol(i32);

impl From<i32> for Protocol {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
