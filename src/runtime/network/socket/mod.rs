// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Exports
//======================================================================================================================

pub mod option;
pub mod state;
#[cfg(feature = "catlions-libos")]
mod types;

#[cfg(feature = "catlions-libos")]
pub use types::{Domain, Protocol, Type};
#[cfg(not(feature = "catlions-libos"))]
pub use socket2::{Domain, Protocol, Type};

//======================================================================================================================
// Imports
//======================================================================================================================

use ::std::net::SocketAddrV4;

//======================================================================================================================
// Structures
//======================================================================================================================

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SocketId {
    Active(SocketAddrV4, SocketAddrV4),
    Passive(SocketAddrV4),
}
