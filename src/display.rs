use crate::{
    command::Command, displayrotation::DisplayRotation, error::Error, DISPLAY_HEIGHT, DISPLAY_WIDTH,
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

// round up to divisible by 8
pub const BUF_SIZE: usize = ((DISPLAY_HEIGHT as usize + 7) / 8) * DISPLAY_WIDTH as usize;

pub struct Ssd1680<SPI, OPIN, IPIN>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    buffer: [u8; BUF_SIZE],
    display_rotation: DisplayRotation,
    spi: SPI,
    dc: OPIN,
    busy: IPIN,
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
            spi,
            dc,
            display_rotation,
            // inverted
            buffer: [0xFF; BUF_SIZE],
            busy,
        }
    }

    pub async fn init<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.software_reset(delay).await?;

        Ok(())
    }

    pub async fn software_reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.send_command(Command::Reset).await?;
        self.busy_wait(delay);

        Ok(())
    }

    pub async fn flush<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.power_up(delay).await?;

        self.set_ram_address(1, 0).await?;

        self.write_ram_frame_buffer().await?;

        // update
        {
            self.send_command(Command::DispCtrl2).await?;
            self.send_data(&[0xF4]).await?;

            self.send_command(Command::MasterActivate).await?;
            self.busy_wait(delay);
            delay.delay_ms(1000);
        }

        self.power_down(delay).await?;
        Ok(())
    }

    async fn write_ram_frame_buffer(&mut self) -> Result<(), Error<E>> {
        self.send_command(Command::WriteRAM1).await?;
        self.dc.set_high().ok();
        self.spi.write(&self.buffer).await.map_err(Error::Comm)?;
        Ok(())
    }

    async fn set_ram_address(&mut self, x: u8, y: u8) -> Result<(), Error<E>> {
        self.send_command(Command::RamXCount).await?;
        self.send_data(&[x]).await?;

        self.send_command(Command::RamYCount).await?;
        self.send_data(&[y, ((y + 7) / 8)]).await
    }

    async fn send_command(&mut self, command: Command) -> Result<(), Error<E>> {
        self.dc.set_low().ok();
        self.spi.write(&[command as u8]).await.map_err(Error::Comm)
    }

    async fn send_data(&mut self, buffer: &[u8]) -> Result<(), Error<E>> {
        self.dc.set_high().ok();
        self.spi.write(buffer).await.map_err(Error::Comm)?;
        Ok(())
    }

    fn hardware_reset<D>(&mut self, delay: &mut D)
    where
        D: DelayUs,
    {
        self.busy_wait(delay);
    }

    fn busy_wait<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        // no amount of delay is working instead of soldered busy pin.. need to scope this
        // delay.delay_ms(500);
        while self.busy.is_high().unwrap() {}
        Ok(())
    }

    async fn power_up<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.hardware_reset(delay);
        delay.delay_ms(100);
        self.busy_wait(delay);

        // command list
        {
            self.software_reset(delay).await?;
            delay.delay_ms(20);

            self.send_command(Command::DataMode).await?;
            self.send_data(&[0x03]).await?;

            self.send_command(Command::Border).await?;
            self.send_data(&[0x05]).await?;

            self.send_command(Command::Vcom).await?;
            self.send_data(&[0x36]).await?;

            self.send_command(Command::GateVoltage).await?;
            self.send_data(&[0x17]).await?;

            self.send_command(Command::SourceVoltage).await?;
            self.send_data(&[0x41, 0x00, 0x32]).await?;

            self.set_ram_address(1, 0).await?;
        }

        self.send_command(Command::RamXPos).await?;
        self.send_data(&[0x01, ((DISPLAY_HEIGHT + 7) / 8)]).await?;

        self.send_command(Command::RamYPos).await?;
        self.send_data(&[0, 0, DISPLAY_WIDTH - 1, ((DISPLAY_WIDTH + 7) / 8)])
            .await?;

        self.send_command(Command::Control).await?;
        self.send_data(&[DISPLAY_WIDTH - 1, ((DISPLAY_WIDTH + 7) / 8), 0])
            .await
    }

    async fn power_down<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.software_reset(delay).await?;

        Ok(())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: BinaryColor) {
        let height = ((DISPLAY_HEIGHT as usize + 7) / 8) as u32;

        let (index, bit) = (
            x / 8 + height * (DISPLAY_WIDTH as u32 - 1 - y),
            0x80 >> (x % 8),
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

fn rotation(x: u32, y: u32, height: u32, width: u32, rotation: DisplayRotation) -> (u32, u8) {
    match rotation {
        DisplayRotation::Rotate0 => (x / 8 + (height - 1 - y) / 8 * y, 0x80 >> (x % 8)),
        DisplayRotation::Rotate90 => (height / 8 + (height - 1 - y) / 8 * x, 0x01 << (y % 8)),
        DisplayRotation::Rotate180 => (
            ((height - 1 - y) / 8 * width) - (x / 8 + (height - 1 - y) / 8 * y),
            0x01 << (x % 8),
        ),
        DisplayRotation::Rotate270 => (y / 8 + (width - x) * (height - 1 - y) / 8, 0x80 >> (y % 8)),
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
