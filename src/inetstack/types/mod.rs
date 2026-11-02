// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[cfg(not(target_os = "none"))]
mod macaddr;
#[cfg(target_os = "none")]
use crate::std_sel4::macaddr;
mod portnum;

//======================================================================================================================
// Exports
//======================================================================================================================

pub use self::{macaddr::MacAddress, portnum::Port16};
