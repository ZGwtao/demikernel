// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

pub mod async_queue;
pub mod async_value;
pub mod hashttlcache;
pub mod id_map;
pub mod pin_slab;

#[cfg(not(target_os = "none"))]
pub use std::collections::{HashMap, HashSet};
#[cfg(target_os = "none")]
pub use rust_alloc::collections::{BinaryHeap, LinkedList, VecDeque, vec_deque};
#[cfg(target_os = "none")]
pub use hashbrown::{HashMap, HashSet};
#[cfg(target_os = "none")]
pub mod hash_map { pub use hashbrown::hash_map::*; }
