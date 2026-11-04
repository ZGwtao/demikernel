// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[cfg(not(feature = "catlions-libos"))]
mod macaddr;
#[cfg(feature = "catlions-libos")]
#[path = "catlions_macaddr.rs"]
mod macaddr;
mod portnum;

//======================================================================================================================
// Exports
//======================================================================================================================

pub use self::{macaddr::MacAddress, portnum::Port16};
