// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[cfg(feature = "catlions-libos")]
pub mod catlions;

pub mod bindings;
#[cfg(not(feature = "catlions-libos"))]
pub mod config;
#[cfg(feature = "catlions-libos")]
pub use self::catlions::config;
pub mod libos;
