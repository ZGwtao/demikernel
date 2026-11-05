// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

#[cfg(not(feature = "catlions-libos"))]
use {crate::runtime::fail::Fail, std::env};

//======================================================================================================================
// Structures
//======================================================================================================================

#[derive(Clone, Copy)]
pub enum LibOSName {
    Catpowder,
    Catnap,
    Catnip,
    Catlions,
}

//======================================================================================================================
// Associated Functions
//======================================================================================================================

#[cfg(not(feature = "catlions-libos"))]
impl LibOSName {
    pub fn from_env() -> Result<Self, Fail> {
        match env::var("LIBOS") {
            Ok(name) => Ok(name.into()),
            Err(_) => Err(Fail::new(libc::EINVAL, "missing value for LIBOS environment variable")),
        }
    }
}

//======================================================================================================================
// Trait Implementations
//======================================================================================================================

impl From<String> for LibOSName {
    fn from(str: String) -> Self {
        match str.to_lowercase().as_str() {
            "catpowder" => LibOSName::Catpowder,
            "catnap" => LibOSName::Catnap,
            "catnip" => LibOSName::Catnip,
            "catlions" => LibOSName::Catlions,
            _ => panic!("unknown libos"),
        }
    }
}
