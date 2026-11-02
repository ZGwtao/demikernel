#[cfg(target_os = "none")]
mod network;

#[cfg(target_os = "none")]
pub mod timer_storage;

pub fn random_seed() -> [u8; 32] {
    [0; 32]
}

#[cfg(target_os = "none")]
pub mod config;
#[cfg(target_os = "none")]
pub mod scheduler;
#[cfg(target_os = "none")]
pub mod runtime;

