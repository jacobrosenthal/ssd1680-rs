//! This example draws a small square one pixel at a time in the top left corner of the display
//!
//! You will probably want to use the [`embedded_graphics`](https://crates.io/crates/embedded-graphics) crate to do more complex drawing.
//!
//! This example is for the STM32F103 "Blue Pill" board using a 4 wire interface to the display on
//! SPI1.
//!
//! Wiring connections are as follows
//!
//! ```
//! GND -> GND
//! 3V3 -> VCC
//! PA5 -> SCL
//! PA7 -> SDA
//! PB0 -> RST
//! PB1 -> D/C
//! ```
//!
//! Run on a Blue Pill with `cargo run --release --example pixelsquare`.

#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use panic_semihosting as _;
use ssd1680::{DisplayRotation::Rotate0, Ssd1680};
use stm32f1xx_hal::{
    delay::Delay,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    stm32,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // SPI1
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let dc = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        8.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut display = Ssd1680::new(spi, dc, Rotate0);

    display.reset(&mut rst, &mut delay).unwrap();
    display.init().unwrap();
    display.flush().unwrap();

    let white = 0xffff;
    let red = 0xf800;
    let green = 0x07e0;
    let blue = 0x001f;

    // Top side
    display.set_pixel(0, 0, white);
    display.set_pixel(1, 0, white);
    display.set_pixel(2, 0, white);
    display.set_pixel(3, 0, white);

    // Right side
    display.set_pixel(3, 0, red);
    display.set_pixel(3, 1, red);
    display.set_pixel(3, 2, red);
    display.set_pixel(3, 3, red);

    // Bottom side
    display.set_pixel(0, 3, green);
    display.set_pixel(1, 3, green);
    display.set_pixel(2, 3, green);
    display.set_pixel(3, 3, green);

    // Left side
    display.set_pixel(0, 0, blue);
    display.set_pixel(0, 1, blue);
    display.set_pixel(0, 2, blue);
    display.set_pixel(0, 3, blue);

    display.flush().unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
