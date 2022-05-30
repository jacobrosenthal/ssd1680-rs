#![feature(async_closure)]
#![no_std]
// #![deny(missing_debug_implementations)]
// #![deny(missing_docs)]
// #![deny(warnings)]
// #![deny(missing_copy_implementations)]
// #![deny(trivial_casts)]
// #![deny(trivial_numeric_casts)]
// #![deny(unsafe_code)]
// #![deny(unstable_features)]
// #![deny(unused_import_braces)]
// #![deny(unused_qualifications)]

extern crate embedded_hal as hal;

const DISPLAY_WIDTH: u8 = 250;
const DISPLAY_HEIGHT: u8 = 122;

mod command;
mod display;
mod displayrotation;
mod error;
#[doc(hidden)]
pub mod test_helpers;

pub use crate::{display::Ssd1680, displayrotation::DisplayRotation, error::Error};
