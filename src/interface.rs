use crate::{command::Command, error::Error, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use core::convert::Infallible;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::delay::DelayUs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

pub struct SpiInterface<SPI, OPIN, OPIN2, P>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    OPIN2: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    spi: SPI,
    dc: OPIN,
    busy: P,
    reset: OPIN2,
}

impl<SPI, OPIN, OPIN2, E, P> SpiInterface<SPI, OPIN, OPIN2, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    OPIN2: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    pub fn new(spi: SPI, dc: OPIN, reset: OPIN2, busy: P) -> Self {
        Self {
            spi,
            dc,
            busy,
            reset,
        }
    }

    pub async fn software_reset(&mut self) -> Result<(), Error<E>> {
        self.send_command(Command::Reset).await?;
        self.busy_wait().await
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

    pub async fn hardware_reset<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.reset.set_low().ok();
        delay.delay_ms(10).await.ok();
        self.reset.set_high().ok();
        Ok(())
    }

    pub async fn busy_wait(&mut self) -> Result<(), Error<E>> {
        self.busy.wait_for_low().await.ok();
        Ok(())
    }

    pub async fn power_up<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs,
    {
        self.hardware_reset(delay).await?;
        self.software_reset().await?;

        // command list
        {
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

    pub async fn power_down(&mut self) -> Result<(), Error<E>> {
        self.send_command(Command::Sleep).await?;
        self.send_data(&[0x01]).await
    }
}
