use crate::{command::Command, error::Error, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

pub struct SpiInterface<SPI, OPIN, IPIN>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    spi: SPI,
    dc: OPIN,
    busy: IPIN,
}

impl<SPI, OPIN, E, IPIN> SpiInterface<SPI, OPIN, IPIN>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    IPIN: InputPin<Error = Infallible>,
{
    pub fn new(spi: SPI, dc: OPIN, busy: IPIN) -> Self {
        Self { spi, dc, busy }
    }

    pub async fn software_reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.send_command(Command::Reset).await?;
        self.busy_wait(delay)
    }

    pub async fn write_ram_frame_buffer(
        &mut self,
        buffer: &[u8],
        command: Command,
    ) -> Result<(), Error<E>> {
        self.send_command(command).await?;
        self.dc.set_high().ok();
        self.spi.write(&buffer[..]).await.map_err(Error::Comm)
    }

    pub async fn set_ram_address(&mut self, x: u8, y: u8) -> Result<(), Error<E>> {
        self.send_command(Command::RamXCount).await?;
        self.send_data(&[x]).await?;

        self.send_command(Command::RamYCount).await?;
        self.send_data(&[y, ((y + 7) / 8)]).await
    }

    pub async fn send_command(&mut self, command: Command) -> Result<(), Error<E>> {
        self.dc.set_low().ok();
        self.spi.write(&[command as u8]).await.map_err(Error::Comm)
    }

    pub async fn send_data(&mut self, buffer: &[u8]) -> Result<(), Error<E>> {
        self.dc.set_high().ok();
        self.spi.write(buffer).await.map_err(Error::Comm)
    }

    pub fn hardware_reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.busy_wait(delay)
    }

    pub fn busy_wait<D>(&mut self, _delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        // no amount of delay is working instead of soldered busy pin.. need to scope this
        // delay.delay_ms(500);
        while self.busy.is_high().map_err(|_| Error::Pin(()))? {}
        Ok(())
    }

    pub async fn power_up<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.hardware_reset(delay)?;
        delay.delay_ms(100);
        self.busy_wait(delay)?;

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

    pub async fn power_down<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.software_reset(delay).await
    }
}
