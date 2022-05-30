// Shamefully taken from https://github.com/EdgewaterDevelopment/rust-ssd1680

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
