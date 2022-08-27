use crate::{command::Command, error::Error, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use core::convert::Infallible;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

pub struct SpiInterface<SPI, OPIN, P>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    spi: SPI,
    dc: OPIN,
    busy: P,
}

impl<SPI, OPIN, E, P> SpiInterface<SPI, OPIN, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
    OPIN: OutputPin<Error = Infallible>,
    P: Wait<Error = Infallible>,
{
    pub fn new(spi: SPI, dc: OPIN, busy: P) -> Self {
        Self { spi, dc, busy }
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

    pub async fn busy_wait(&mut self) -> Result<(), Error<E>> {
        self.busy.wait_for_low().await.ok();
        Ok(())
    }

    pub async fn power_up(&mut self) -> Result<(), Error<E>> {
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
        // // Sleeps and keeps access to RAM and controller
        // Normal = 0x00,
        // // Sleeps without access to RAM/controller but keeps RAM content
        // Mode1 = 0x01,
        // // Same as MODE_1 but RAM content is not kept
        // Mode2 = 0x11,
        // but thats 11 bits, or 3 right? todo  talk to epd waveshare people

        // 0x0
        // 0x1
        // 0x2
        // 0x3

        self.send_command(Command::Sleep).await?;
        self.send_data(&[0x01]).await?;
        Ok(())
    }
}
