#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use core::{
    fmt,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

struct Storage {
    data: Vec<u8>,
    external: Option<External>,
}

struct External {
    ptr: NonNull<u8>,
    capacity: usize,
    token: u64,
    release: unsafe extern "C" fn(u64),
    tx: bool,
}

impl Storage {
    fn as_slice(&self) -> &[u8] {
        match &self.external {
            Some(external) => unsafe {
                core::slice::from_raw_parts(external.ptr.as_ptr(), external.capacity)
            },
            None => &self.data,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        match &self.external {
            Some(external) => unsafe {
                core::slice::from_raw_parts_mut(external.ptr.as_ptr(), external.capacity)
            },
            None => &mut self.data,
        }
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self {
            data: self.as_slice().to_vec(),
            external: None,
        }
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        if let Some(external) = &self.external {
            unsafe { (external.release)(external.token) };
        }
    }
}

/// Demikernel buffer backed by heap memory or a borrowed SDDF RX/TX descriptor.
/// Views share storage until one is mutably accessed.
pub struct DemiBuffer {
    storage: Rc<Storage>,
    offset: u16,
    len: u16,
    direct: bool,
}

impl fmt::Debug for DemiBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DemiBuffer")
            .field("len", &self.len)
            .field("headroom", &self.offset)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    TooLarge,
    OutOfBounds,
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("buffer too large"),
            Self::OutOfBounds => f.write_str("buffer operation outside data"),
        }
    }
}

impl core::error::Error for BufferError {}

impl DemiBuffer {
    pub fn new(capacity: u16) -> Self {
        Self::new_with_headroom(capacity, 0)
    }

    pub fn new_with_headroom(capacity: u16, headroom: u16) -> Self {
        let total = capacity.checked_add(headroom).expect("buffer too large");
        #[cfg(target_os = "none")]
        if headroom > 0 && total <= 2048 {
            if let Some((ptr, token)) = sddf::tx_acquire() {
                return Self {
                    storage: Rc::new(Storage {
                        data: Vec::new(),
                        external: Some(External {
                            ptr,
                            capacity: 2048,
                            token,
                            release: sddf::tx_release,
                            tx: true,
                        }),
                    }),
                    offset: headroom,
                    len: capacity,
                    direct: true,
                };
            }
        }
        Self {
            storage: Rc::new(Storage {
                data: vec![0; total as usize],
                external: None,
            }),
            offset: headroom,
            len: capacity,
            direct: true,
        }
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, BufferError> {
        Self::from_slice_with_headroom(data, 0)
    }

    /// # Safety
    /// The memory must remain mapped until `release` is called. The caller gives
    /// this buffer exclusive ownership of the external descriptor.
    pub unsafe fn from_external(
        ptr: NonNull<u8>,
        len: u16,
        token: u64,
        release: unsafe extern "C" fn(u64),
    ) -> Self {
        Self {
            storage: Rc::new(Storage {
                data: Vec::new(),
                external: Some(External {
                    ptr,
                    capacity: len as usize,
                    token,
                    release,
                    tx: false,
                }),
            }),
            offset: 0,
            len,
            direct: true,
        }
    }

    pub fn from_slice_with_headroom(data: &[u8], headroom: usize) -> Result<Self, BufferError> {
        let total = data
            .len()
            .checked_add(headroom)
            .ok_or(BufferError::TooLarge)?;
        if total >= u16::MAX as usize {
            return Err(BufferError::TooLarge);
        }
        let mut buffer = Self::new_with_headroom(data.len() as u16, headroom as u16);
        buffer.copy_from_slice(data);
        Ok(buffer)
    }

    pub fn headroom(&self) -> u16 {
        self.offset
    }

    pub fn is_heap_allocated(&self) -> bool {
        self.storage.external.is_none()
    }

    pub fn is_direct(&self) -> bool {
        self.direct
    }

    pub fn into_direct(mut self) -> Self {
        if !self.direct {
            Rc::make_mut(&mut self.storage);
            self.direct = true;
        }
        self
    }

    pub fn into_raw(self) -> NonNull<u8> {
        NonNull::new(Box::into_raw(Box::new(self))).unwrap().cast()
    }

    /// Transfers a uniquely held TX-backed packet to its SDDF descriptor.
    /// Shared packets remain with Demikernel so TCP may retransmit them.
    pub fn into_tx_descriptor(self) -> Result<(u64, usize), Self> {
        if Rc::strong_count(&self.storage) != 1
            || !self
                .storage
                .external
                .as_ref()
                .is_some_and(|external| external.tx)
        {
            return Err(self);
        }
        let Self {
            storage,
            offset,
            len,
            ..
        } = self;
        let mut storage = Rc::try_unwrap(storage).ok().unwrap();
        let external = storage.external.take().unwrap();
        unsafe {
            core::ptr::copy(
                external.ptr.as_ptr().add(offset as usize),
                external.ptr.as_ptr(),
                len as usize,
            );
        }
        Ok((external.token, len as usize))
    }

    /// # Safety
    /// `token` must come from `DemiBuffer::into_raw` and be consumed exactly once.
    pub unsafe fn from_raw(token: NonNull<u8>) -> Self {
        *unsafe { Box::from_raw(token.cast::<Self>().as_ptr()) }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn adjust(&mut self, count: usize) -> Result<(), BufferError> {
        if count > self.len() {
            return Err(BufferError::OutOfBounds);
        }
        self.offset += count as u16;
        self.len -= count as u16;
        Ok(())
    }

    pub fn trim(&mut self, count: usize) -> Result<(), BufferError> {
        if count > self.len() {
            return Err(BufferError::OutOfBounds);
        }
        self.len -= count as u16;
        Ok(())
    }

    pub fn prepend(&mut self, count: usize) -> Result<(), BufferError> {
        if count > self.headroom() as usize {
            return Err(BufferError::OutOfBounds);
        }
        self.offset -= count as u16;
        self.len += count as u16;
        Ok(())
    }

    pub fn split_back(&mut self, offset: usize) -> Result<Self, BufferError> {
        if offset > self.len() {
            return Err(BufferError::OutOfBounds);
        }
        let mut back = self.clone();
        back.adjust(offset)?;
        self.trim(self.len() - offset)?;
        Ok(back)
    }

    pub fn split_front(&mut self, offset: usize) -> Result<Self, BufferError> {
        if offset > self.len() {
            return Err(BufferError::OutOfBounds);
        }
        let mut front = self.clone();
        front.trim(self.len() - offset)?;
        self.adjust(offset)?;
        Ok(front)
    }
}

impl Clone for DemiBuffer {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            offset: self.offset,
            len: self.len,
            direct: self.is_empty(),
        }
    }
}

impl Deref for DemiBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        let start = self.offset as usize;
        &self.storage.as_slice()[start..start + self.len()]
    }
}

impl DerefMut for DemiBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let start = self.offset as usize;
        let len = self.len();
        &mut Rc::make_mut(&mut self.storage).as_mut_slice()[start..start + len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    static RELEASES: AtomicUsize = AtomicUsize::new(0);
    static TX_RELEASES: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn release(_token: u64) {
        RELEASES.fetch_add(1, Ordering::Relaxed);
    }

    unsafe extern "C" fn release_tx(_token: u64) {
        TX_RELEASES.fetch_add(1, Ordering::Relaxed);
    }

    #[test]
    fn unique_tx_buffer_transfers_descriptor() {
        TX_RELEASES.store(0, Ordering::Relaxed);
        let mut bytes = [0u8; 64];
        bytes[8..13].copy_from_slice(b"hello");
        let packet = DemiBuffer {
            storage: Rc::new(Storage {
                data: Vec::new(),
                external: Some(External {
                    ptr: NonNull::new(bytes.as_mut_ptr()).unwrap(),
                    capacity: bytes.len(),
                    token: 12,
                    release: release_tx,
                    tx: true,
                }),
            }),
            offset: 8,
            len: 5,
            direct: true,
        };
        assert!(packet.clone().into_tx_descriptor().is_err());
        assert_eq!(packet.into_tx_descriptor().unwrap(), (12, 5));
        assert_eq!(&bytes[..5], b"hello");
        assert_eq!(TX_RELEASES.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn headroom_and_split() {
        let mut packet = DemiBuffer::from_slice_with_headroom(b"payload", 4).unwrap();
        packet.prepend(4).unwrap();
        packet[..4].copy_from_slice(b"head");
        let tail = packet.split_back(4).unwrap();
        assert_eq!(&*packet, b"head");
        assert_eq!(&*tail, b"payload");
        assert_eq!(tail.headroom(), 4);
    }

    #[test]
    fn clone_owns_mutation() {
        let packet = DemiBuffer::from_slice(b"hello").unwrap();
        let mut copy = packet.clone();
        copy[0] = b'j';
        assert_eq!(&*packet, b"hello");
        assert_eq!(&*copy, b"jello");
    }

    #[test]
    fn raw_round_trip() {
        let packet = DemiBuffer::from_slice(b"hello").unwrap();
        let token = packet.into_raw();
        let packet = unsafe { DemiBuffer::from_raw(token) };
        assert_eq!(&*packet, b"hello");
    }

    #[test]
    fn external_buffer_returns_after_last_view() {
        RELEASES.store(0, Ordering::Relaxed);
        let mut bytes = [1, 2, 3, 4];
        let packet = unsafe {
            DemiBuffer::from_external(NonNull::new(bytes.as_mut_ptr()).unwrap(), 4, 7, release)
        };
        let view = packet.clone();
        drop(packet);
        assert_eq!(RELEASES.load(Ordering::Relaxed), 0);
        assert_eq!(&*view, &bytes);
        drop(view);
        assert_eq!(RELEASES.load(Ordering::Relaxed), 1);
    }
}
