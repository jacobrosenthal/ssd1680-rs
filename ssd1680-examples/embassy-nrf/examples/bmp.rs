//! The rust-toolchain will pull in the correct nightly and target so all you
//! need to run is
//!
//! Feather xenon
//! https://docs.particle.io/datasheets/discontinued/xenon-datasheet/
//! https://docs.particle.io/assets/images/xenon/xenon-pinout-v1.0.pdf
//! https://docs.particle.io/assets/images/xenon/xenon-block-diagram.png
//!
//! antenna selection
//! p025 = 0, p0.24 = 1 pcb antenna
//! p025 = 1, p0.24 = 0 external u.fl
//!
//! p0.13 red rgb
//! p0.14 green rgb
//! p0.15 blue rgb
//! p0.11 button
//! p1.12 blue led
//!
//! p9.27 scl
//! p0.26 sda
//!
//! ssd1680
//! p1.10 dc
//! p1.08 ecs
//! p1.02 srcs
//! p1.01 sd cs
//!
//! p0.31 ss
//! p1.15 sck
//! p1.13 mosi
//!
//! cargo run --release --example bmp
//!
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::future::pending;
use embassy::interrupt::InterruptExt;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::gpio::{self, AnyPin, Level, NoPin, Output, OutputDrive, Pin};
use embassy_nrf::{interrupt, spim};
use embedded_graphics::prelude::*;
use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use embedded_hal::digital::v2::OutputPin;
use ssd1680::{DisplayRotation, Ssd1680};
use tinybmp::Bmp;

// we make a lazily created static
static EXECUTOR: Forever<embassy::executor::Executor> = Forever::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    // once we hit runtime we create and fill that executor finally
    let executor = EXECUTOR.put(embassy::executor::Executor::new());

    // provides the peripherals from the async first pac if you selected it
    let dp = embassy_nrf::init(Default::default());

    let blue = gpio::Output::new(
        // degrade just a typesystem hack to forget which pin it is so we can
        // call it Anypin and make our function calls more generic
        dp.P1_12.degrade(),
        gpio::Level::High,
        gpio::OutputDrive::Standard,
    );

    // spawn tasks
    executor.run(|spawner| {
        let _ = spawner.spawn(blinky_task(blue));
        // let _ = spawner.spawn(display_task());
    })
}

#[embassy::task]
async fn blinky_task(mut led: gpio::Output<'static, AnyPin>) {
    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy::task]
pub async fn display_task() {
    // Too lazy to pass all the pins and peripherals we need.
    // Safety: Fragile but safe as long as pins and peripherals arent used
    // anywhere else
    let mut dp = unsafe { <embassy_nrf::Peripherals as embassy::util::Steal>::steal() };

    let mut spim_irq = interrupt::take!(SPIM3);
    spim_irq.set_priority(interrupt::Priority::P4);

    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M16;
    let spim = spim::Spim::new(
        &mut dp.SPI3,
        &mut spim_irq,
        &mut dp.P1_15,
        NoPin,
        &mut dp.P1_13,
        spim_config,
    );

    // p0.31 ss
    // p1.08 ecs
    let dc = Output::new(&mut dp.P1_10, Level::High, OutputDrive::Standard);
    let mut display = Ssd1680::new(spim, dc, DisplayRotation::Rotate0);

    // let mut rst = Output::new(&mut dp.P1_08, Level::High, OutputDrive::Standard);
    // Timer::after(Duration::from_millis(1)).await;
    // rst.set_low().ok();
    // Timer::after(Duration::from_millis(1)).await;
    // rst.set_high().ok();

    display.init().unwrap();

    let (w, h) = display.dimensions();

    let bmp = Bmp::from_slice(include_bytes!("../../../assets/rust-pride.bmp"))
        .expect("Failed to load BMP image");

    let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::zero());

    // Position image in the center of the display
    let moved = im.translate(Point::new(
        (w as u32 - bmp.size().width) as i32 / 2,
        (h as u32 - bmp.size().height) as i32 / 2,
    ));

    moved.draw(&mut display).unwrap();

    display.flush_async().await.unwrap();
    // display.flush().unwrap();

    // Block forever so the above drivers don't get dropped
    pending::<()>().await;
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
