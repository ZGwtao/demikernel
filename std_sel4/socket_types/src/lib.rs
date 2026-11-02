#![no_std]

macro_rules! socket_value {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(i32);
        impl From<i32> for $name {
            fn from(value: i32) -> Self { Self(value) }
        }
        impl From<$name> for i32 {
            fn from(value: $name) -> Self { value.0 }
        }
    };
}

socket_value!(Domain);
socket_value!(Type);
socket_value!(Protocol);

impl Domain {
    pub const IPV4: Self = Self(2);
    pub const IPV6: Self = Self(10);
}
impl Type {
    pub const STREAM: Self = Self(1);
    pub const DGRAM: Self = Self(2);
}
impl Protocol {
    pub const TCP: Self = Self(6);
    pub const UDP: Self = Self(17);
}
