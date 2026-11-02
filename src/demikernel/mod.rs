// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[cfg(not(feature = "catlions-libos"))]
pub mod bindings;
#[cfg(not(target_os = "none"))]
pub mod config;
#[cfg(target_os = "none")]
pub use crate::catlions::config;
pub mod libos;
