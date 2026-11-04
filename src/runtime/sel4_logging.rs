// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LevelFilter::Info)
    .write(|message| sel4::debug_print!("{}", message))
    .build();

pub fn initialize() {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}
