pub use core::{any, cell, cmp, convert, error, ffi, fmt, future, iter, marker, mem, num, ops, option, pin, ptr, result, slice, str, task};
pub use rust_alloc::{boxed, rc, string, vec};

pub mod alloc {
    pub use core::alloc::*;
    pub use rust_alloc::alloc::{Allocator, Global, alloc, dealloc};
    pub use rust_alloc::{boxed, collections, rc, string, vec};
}

pub mod net { pub use core::net::*; }
pub mod time { pub use core::time::Duration; pub use sddf::Instant; }
pub mod hash { pub use core::hash::*; pub type RandomState = hashbrown::DefaultHashBuilder; }
pub mod process { pub fn id() -> u32 { 1 } }

pub mod io {
    #[derive(Debug)]
    pub struct Error;
    pub struct Cursor<T> { data: T, position: usize }
    impl<T> Cursor<T> {
        pub fn new(data: T) -> Self { Self { data, position: 0 } }
        pub fn position(&self) -> u64 { self.position as u64 }
    }
    pub trait Read { fn read_exact(&mut self, output: &mut [u8]) -> Result<(), Error>; }
    impl Read for Cursor<&[u8]> {
        fn read_exact(&mut self, output: &mut [u8]) -> Result<(), Error> {
            let end = self.position + output.len();
            let input = self.data.get(self.position..end).ok_or(Error)?;
            output.copy_from_slice(input);
            self.position = end;
            Ok(())
        }
    }
}

pub mod env { pub fn var(_: &str) -> Result<rust_alloc::string::String, ()> { Err(()) } }

pub mod pal;
pub mod macaddr;
