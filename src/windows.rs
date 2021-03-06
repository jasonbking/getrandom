// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Windows
extern crate winapi;

use self::winapi::shared::minwindef::ULONG;
use self::winapi::um::ntsecapi::RtlGenRandom;
use self::winapi::um::winnt::PVOID;
use std::io;
use super::Error;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let ret = unsafe {
        RtlGenRandom(dest.as_mut_ptr() as PVOID, dest.len() as ULONG)
    };
    if ret == 0 { return Err(io::Error::last_os_error().into()); }
    Ok(())
}
