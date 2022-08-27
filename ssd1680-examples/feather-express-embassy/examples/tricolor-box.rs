//! The rust-toolchain will pull in the correct nightly and target so all you
//! need to run is
//!
//! Feather nrf52840 express
//! https://www.adafruit.com/product/4062
//! https://learn.adafruit.com/introducing-the-adafruit-nrf52840-feather?view=all
//! https://learn.adafruit.com/assets/68545/
//!
//! Adafruit 2.13" HD Tri-Color eInk / ePaper Display FeatherWing - 250x122 RW Panel with SSD1680
//! https://www.adafruit.com/product/4814
//! https://learn.adafruit.com/adafruit-2-13-eink-display-breakouts-and-featherwings?view=all
//! As of April 27, 2020 we're selling a version with SSD1680 chipset, instead of the SSD1675 chipset
//! Busy and Rst pin not connected
//!
//! P1_02 button
//! P0_16 neopixel
//! P1_10 led blue
//! P1_15 led red
//!
//! thinkink
//! P0_14 sck
//! P0_13 mosi
//! P0_15 miso
//! skip 3
//! P0_30 rst MUST SOLDER
//!
//! P0_06 11 busy MUST SOLDER
//! P0_27 10 dc
//! P0_26 9 cs
//! P0_07 6 srcs
//! P1_08 5 sd cs
//! skip 2
//!
//! DEFMT_LOG=trace cargo run --release --example tricolor-box
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use core::future::pending;
use embassy_time::{Delay, Duration, Timer};
use embassy_util::Forever;
use embassy_nrf::gpio::{self, AnyPin, Pin};
use embassy_nrf::interrupt::{self, InterruptExt};
use embedded_graphics::{
    prelude::*,
    primitives::{ PrimitiveStyle, Rectangle},
};
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::spi::ExclusiveDevice;
use ssd1680::{DisplayRotation, Ssd1680TriColor, TriColor};
use embassy_nrf::spim;

// we make a lazily created static
static EXECUTOR: Forever<embassy_executor::Executor> = Forever::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    // well use these logging macros instead of println to tunnel our logs via the debug chip
    info!("Hello Wosrld!");

    // once we hit runtime we create and fill that executor finally
    let executor = EXECUTOR.put(embassy_executor::Executor::new());

    // provides the peripherals from the async first pac if you selected it
    let dp = embassy_nrf::init(embassy_config());

    let blue = gpio::Output::new(
        // degrade just a typesystem hack to forget which pin it is so we can
        // call it Anypin and make our function calls more generic
        dp.P1_10.degrade(),
        gpio::Level::High,
        gpio::OutputDrive::Standard,
    );

    // spawn tasks
    executor.run(|spawner| {
        let _ = spawner.spawn(blinky_task(blue));
        let _ = spawner.spawn(display_task());
    })
}

#[embassy_executor::task]
async fn blinky_task(mut led: gpio::Output<'static, AnyPin>) {
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
pub async fn display_task() {
    // Too lazy to pass all the pins and peripherals we need.
    // Safety: Fragile but safe as long as pins and peripherals arent used
    // anywhere else
    let mut dp = unsafe { embassy_nrf::Peripherals::steal() };

    let mut spim_irq = interrupt::take!(SPIM3);
    spim_irq.set_priority(interrupt::Priority::P4);

    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M4;
    let spim = spim::Spim::new_txonly(
        &mut dp.SPI3,
        &mut spim_irq,
        &mut dp.P0_14,
        &mut dp.P0_13,
        spim_config,
    );

    let cs = gpio::Output::new(
        dp.P0_26.degrade(),
        gpio::Level::Low,
        gpio::OutputDrive::Standard,
    );
    let spi_dev = ExclusiveDevice::new(spim, cs);

    let dc = gpio::Output::new(
        dp.P0_27.degrade(),
        gpio::Level::High,
        gpio::OutputDrive::Standard,
    );

    let mut reset = gpio::Output::new(
        dp.P0_30.degrade(),
        gpio::Level::High,
        gpio::OutputDrive::Standard,
    );

    let busy = gpio::Input::new(dp.P0_06.degrade(), gpio::Pull::Up);

    let mut ssd1680 = Ssd1680TriColor::new(spi_dev, dc, busy, DisplayRotation::Rotate0);
    ssd1680.init(&mut Delay, &mut reset).await.unwrap();

    Rectangle::new(Point::new(0, 0), Size::new(15, 15))
        .into_styled(PrimitiveStyle::with_fill(TriColor::Chromatic))
        .draw(&mut ssd1680)
        .unwrap();

    ssd1680.flush(&mut Delay, &mut reset).await.unwrap();

    Delay.delay_ms(2000).await.unwrap();

    Rectangle::new(Point::new(0, 0), Size::new(30, 30))
        .into_styled(PrimitiveStyle::with_fill(TriColor::Chromatic))
        .draw(&mut ssd1680)
        .unwrap();

    ssd1680.flush(&mut Delay, &mut reset).await.unwrap();

    pending::<()>().await;
}

// WARNING may overflow and wrap-around in long lived apps
defmt::timestamp! {"{=usize}", {
    use core::sync::atomic::{AtomicUsize, Ordering};

    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
}}

// 0 is Highest. Lower prio number can preempt higher prio number
// Softdevice has reserved priorities 0, 1 and 3
pub fn embassy_config() -> embassy_nrf::config::Config {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    config.time_interrupt_priority = interrupt::Priority::P2;
    // if we see button misses lower this
    config.gpiote_interrupt_priority = interrupt::Priority::P7;
    config
}
