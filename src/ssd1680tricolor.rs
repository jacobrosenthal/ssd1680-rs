use crate::{
    command::Command,
    display::{find_rotation, DisplayRotation},
    error::Error,
    interface::SpiInterface,
    BUF_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

use core::convert::Infallible;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    prelude::*,
};

pub struct Ssd1680TriColor<SPI, OPIN, P>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    buffer: [u8; BUF_SIZE],
    buffer2: [u8; BUF_SIZE],
    display_rotation: DisplayRotation,
    interface: SpiInterface<SPI, OPIN, P>,
}

impl<SPI, OPIN, E, P> Ssd1680TriColor<SPI, OPIN, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    pub fn new(spi: SPI, dc: OPIN, busy: P, display_rotation: DisplayRotation) -> Self {
        Self {
            interface: SpiInterface::new(spi, dc, busy),
            display_rotation,
            buffer: [0xFF; BUF_SIZE], // inverted
            buffer2: [0x00; BUF_SIZE],
        }
    }

    pub async fn init<D, OPIN2>(&mut self, delay: &mut D, reset: &mut OPIN2) -> Result<(), Error<E>>
    where
        OPIN2: OutputPin<Error = Infallible>,
        D: DelayUs,
    {
        reset.set_low().ok();
        delay.delay_ms(10).await.ok();
        reset.set_high().ok();

        self.interface.software_reset().await
    }

    pub async fn flush<D, OPIN2>(
        &mut self,
        delay: &mut D,
        reset: &mut OPIN2,
    ) -> Result<(), Error<E>>
    where
        D: DelayUs,
        OPIN2: OutputPin<Error = Infallible>,
    {
        reset.set_low().ok();
        delay.delay_ms(10).await.ok();
        reset.set_high().ok();

        self.interface.power_up().await?;

        self.flush_display().await?;

        self.flush_update().await
    }

    pub async fn flush_display(&mut self) -> Result<(), Error<E>> {
        self.interface.set_ram_address(1, 0).await?;

        self.interface
            .write_ram_frame_buffer(&self.buffer, Command::WriteRAM1)
            .await?;

        self.interface.busy_wait().await?;

        self.interface.set_ram_address(1, 0).await?;

        self.interface
            .write_ram_frame_buffer(&self.buffer2, Command::WriteRAM2)
            .await?;

        self.interface.busy_wait().await
    }

    pub async fn flush_update(&mut self) -> Result<(), Error<E>> {
        self.interface.send_command(Command::DispCtrl2).await?;
        self.interface.send_data(&[0xF4]).await?;

        self.interface.send_command(Command::MasterActivate).await?;
        self.interface.busy_wait().await
    }

    pub async fn power_down<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.interface.power_down().await?;
        delay.delay_ms(100).await.ok();
        Ok(())
    }

    pub async fn power_up<D, OPIN2>(
        &mut self,
        delay: &mut D,
        reset: &mut OPIN2,
    ) -> Result<(), Error<E>>
    where
        D: DelayUs,
        OPIN2: OutputPin<Error = Infallible>,
    {
        reset.set_low().ok();
        delay.delay_ms(10).await.ok();
        reset.set_high().ok();

        self.interface.power_up().await
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: TriColor) {
        let height = ((DISPLAY_HEIGHT as usize + 7) / 8) as u32;

        let (nx, ny) = find_rotation(
            x,
            y,
            DISPLAY_HEIGHT.into(),
            DISPLAY_WIDTH.into(),
            self.display_rotation,
        );

        let (index, bit) = (
            nx / 8 + height * (DISPLAY_WIDTH as u32 - 1 - ny),
            0x80 >> (nx % 8),
        );

        let index = index as usize;
        if index >= self.buffer.len() {
            return;
        }

        match color {
            TriColor::Black => {
                self.buffer[index] &= !bit;
                self.buffer2[index] &= !bit;
            }
            TriColor::Chromatic => {
                self.buffer[index] |= bit;
                self.buffer2[index] |= bit;
            }
            TriColor::White => {
                self.buffer[index] |= bit;
                self.buffer2[index] &= !bit;
            }
        }
    }
}

#[cfg(feature = "graphics")]
impl<SPI, OPIN, E, P> DrawTarget for Ssd1680TriColor<SPI, OPIN, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| self.set_pixel(pos.x as u32, pos.y as u32, color));

        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<SPI, OPIN, E, P> OriginDimensions for Ssd1680TriColor<SPI, OPIN, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH.into(), DISPLAY_HEIGHT.into())
    }
}

/// Only for the Black/White/Color-Displays
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TriColor {
    /// Black color
    Black,
    /// White color
    White,
    /// Chromatic color
    Chromatic,
}

impl TriColor {
    /// Get the color encoding of the color for one bit
    pub fn get_bit_value(self) -> u8 {
        match self {
            TriColor::White => 1u8,
            TriColor::Black | TriColor::Chromatic => 0u8,
        }
    }

    /// Gets a full byte of black or white pixels
    pub fn get_byte_value(self) -> u8 {
        match self {
            TriColor::White => 0xff,
            TriColor::Black | TriColor::Chromatic => 0x00,
        }
    }
}

#[cfg(feature = "graphics")]
impl PixelColor for TriColor {
    type Raw = ();
}
