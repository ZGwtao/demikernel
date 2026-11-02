use crate::runtime::timer::SharedTimer;
use core::cell::UnsafeCell;

pub struct SinglePdTimer(UnsafeCell<Option<SharedTimer>>);

unsafe impl Sync for SinglePdTimer {}

impl SinglePdTimer {
    pub fn with<R>(&self, f: impl FnOnce(&SharedTimer) -> R) -> R {
        let slot = unsafe { &mut *self.0.get() };
        f(slot.get_or_insert_with(SharedTimer::default))
    }
}

pub static THREAD_TIME: SinglePdTimer = SinglePdTimer(UnsafeCell::new(None));
