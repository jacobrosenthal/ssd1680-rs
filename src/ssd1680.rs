use crate::{
    command::Command,
    display::{find_rotation, DisplayRotation},
    error::Error,
    interface::SpiInterface,
    BUF_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH,
};

use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    pixelcolor::BinaryColor,
    prelude::*,
};

pub struct Ssd1680<SPI, OPIN, IPIN>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    buffer: [u8; BUF_SIZE],
    display_rotation: DisplayRotation,
    interface: SpiInterface<SPI, OPIN, IPIN>,
}

impl<SPI, OPIN, E, IPIN> Ssd1680<SPI, OPIN, IPIN>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    pub fn new(spi: SPI, dc: OPIN, busy: IPIN, display_rotation: DisplayRotation) -> Self {
        Self {
            interface: SpiInterface::new(spi, dc, busy),
            display_rotation,
            buffer: [0xFF; BUF_SIZE], // inverted
        }
    }

    pub async fn init<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.interface.software_reset(delay).await
    }

    pub async fn flush<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.interface.power_up(delay).await?;

        self.interface.set_ram_address(1, 0).await?;

        self.interface
            .write_ram_frame_buffer(&self.buffer, Command::WriteRAM1)
            .await?;

        // update
        {
            self.interface.send_command(Command::DispCtrl2).await?;
            self.interface.send_data(&[0xF4]).await?;

            self.interface.send_command(Command::MasterActivate).await?;
            self.interface.busy_wait(delay)?;
            delay.delay_ms(1000);
        }

        self.interface.power_down(delay).await
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: BinaryColor) {
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
            BinaryColor::On => {
                self.buffer[index] &= !bit;
            }
            BinaryColor::Off => {
                self.buffer[index] |= bit;
            }
        }
    }
}

#[cfg(feature = "graphics")]
impl<SPI, OPIN, E, IPIN> DrawTarget for Ssd1680<SPI, OPIN, IPIN>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    type Color = BinaryColor;
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
impl<SPI, OPIN, E, IPIN> OriginDimensions for Ssd1680<SPI, OPIN, IPIN>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH.into(), DISPLAY_HEIGHT.into())
    }
}
