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

const DISPLAY_WIDTH: u8 = 250;
const DISPLAY_HEIGHT: u8 = 122;
// round up to divisible by 8
const BUF_SIZE: usize = ((DISPLAY_HEIGHT as usize + 7) / 8) * DISPLAY_WIDTH as usize;

pub use crate::{
    display::DisplayRotation, error::Error, ssd1680::Ssd1680, ssd1680tricolor::Ssd1680TriColor,
    ssd1680tricolor::TriColor,
};

mod display;
mod interface;
mod ssd1680;
mod ssd1680tricolor;

mod command {

    /// SSD1680 Commands
    #[derive(Debug)]
    #[allow(dead_code)]
    pub enum Command {
        /// SW Reset
        Reset = 0x12,
        DataMode = 0x11,
        Border = 0x3C,
        GateVoltage = 0x03,
        SourceVoltage = 0x04,
        RamXCount = 0x4E,
        RamYCount = 0x4F,
        RamXPos = 0x44,
        RamYPos = 0x45,
        Control = 0x01,
        Vcom = 0x2C,
        DispCtrl1 = 0x21,
        DispCtrl2 = 0x22,
        MasterActivate = 0x20,
        WriteRAM1 = 0x24,
        WriteRAM2 = 0x26,
        Sleep = 0x10,
    }
}

mod error {
    use core::convert::Infallible;

    #[derive(Debug)]
    pub enum Error<E = ()> {
        /// Communication error
        Comm(E),
        /// Pin setting error
        Pin(Infallible),
    }
}
