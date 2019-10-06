use super::Gpio2Config;
use core::marker::PhantomData;
use hal::blocking::delay::DelayMs;
use hal::blocking::i2c;
use hal::digital::v2::OutputPin;

const DEVICE_ADDRESS: u8 = 0x10;

struct Register;
impl Register {
    const POWERCFG: usize = 0x2;
    const SYSCONFIG1: usize = 0x4;
    const SYSCONFIG2: usize = 0x5;
    const TEST1: usize = 0x7;
}

struct BitFlags;
impl BitFlags {
    const DMUTE: u16 = 1 << 14;
    const DE: u16 = 1 << 11;
    const SKMODE: u16 = 1 << 10;
    const SEEKUP: u16 = 1 << 9;
    const ENABLE: u16 = 1;
    const STCIEN: u16 = 1 << 14;
}

/// Reset the device and select I2C communication
///
/// This includes a 2ms delay to allow the pins to settle and the device
/// to perform the reset.
pub fn reset<E, RSTP: OutputPin<Error = E>, SDAP: OutputPin<Error = E>, DELAY: DelayMs<u8>>(
    rst: &mut RSTP,
    sda: &mut SDAP,
    delay: &mut DELAY,
) -> Result<(), E> {
    sda.set_low()?;
    rst.set_low()?;
    delay.delay_ms(1);
    rst.set_high()?;
    delay.delay_ms(1);
    Ok(())
}

impl<I2C, E> Si470x<I2C, ic::Si4703>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    /// Create new instance of a Si4703 device
    pub fn new_si4703(i2c: I2C) -> Self {
        Si470x {
            i2c,
            is_seeking: false,
            _ic: PhantomData,
        }
    }
}

impl<I2C, IC> Si470x<I2C, IC> {
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E, IC> Si470x<I2C, IC>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    /// Enable the oscillator.
    ///
    /// This must be done at the beginning.
    /// After calling this, a minimum of 500ms must be waited in order
    /// for the oscillator to power up.
    pub fn enable_oscillator(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::TEST1] = 0x8100;
        self.write_registers(&regs[0..=Register::TEST1])
    }

    /// Enable the device.
    ///
    /// After calling this it must be waited for the device to power up.
    /// See: Powerup time in the datasheet.
    /// On the Si4703, this is a maximum of 110ms.
    pub fn enable(&mut self) -> Result<(), Error<E>> {
        self.write_powercfg(BitFlags::ENABLE)
    }

    /// Unmute (disable mute)
    pub fn unmute(&mut self) -> Result<(), Error<E>> {
        let powercfg = self.read_powercfg()?;
        self.write_powercfg(powercfg | BitFlags::DMUTE)
    }

    /// Mute (enable mute)
    pub fn mute(&mut self) -> Result<(), Error<E>> {
        let powercfg = self.read_powercfg()?;
        self.write_powercfg(powercfg & !BitFlags::DMUTE)
    }

    /// Configure seeking
    pub fn configure_seek(
        &mut self,
        mode: SeekMode,
        direction: SeekDirection,
    ) -> Result<(), Error<E>> {
        let powercfg = self.read_powercfg()?;
        let powercfg = match mode {
            SeekMode::Wrap => powercfg | BitFlags::SKMODE,
            SeekMode::NoWrap => powercfg & !BitFlags::SKMODE,
        };
        let powercfg = match direction {
            SeekDirection::Up => powercfg | BitFlags::SEEKUP,
            SeekDirection::Down => powercfg & !BitFlags::SEEKUP,
        };
        self.write_powercfg(powercfg)
    }

    /// Set de-emphasis
    pub fn set_deemphasis(&mut self, de: DeEmphasis) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        match de {
            DeEmphasis::Us75 => regs[Register::SYSCONFIG1] &= !BitFlags::DE,
            DeEmphasis::Us50 => regs[Register::SYSCONFIG1] |= BitFlags::DE,
        }
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Set the volume [0..15]
    ///
    /// For volume values greater than 15, `Error::InvalidInputData`
    /// will be returned.
    pub fn set_volume(&mut self, volume: u8) -> Result<(), Error<E>> {
        if volume > 15 {
            return Err(Error::InvalidInputData);
        }
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG2] &= 0xFFF0;
        regs[Register::SYSCONFIG2] |= u16::from(volume);
        self.write_registers(&regs[0..=Register::SYSCONFIG2])
    }

    /// Set GPIO2 function / status
    pub fn set_gpio2(&mut self, config: Gpio2Config) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match config {
            Gpio2Config::HighImpedance => 0,
            Gpio2Config::StcRdsInterrupt => 1,
            Gpio2Config::Low => 2,
            Gpio2Config::High => 3,
        };
        regs[Register::SYSCONFIG1] &= 0xFFF3;
        regs[Register::SYSCONFIG1] |= mask << 2;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    fn read_powercfg(&mut self) -> Result<u16, Error<E>> {
        const OFFSET: usize = 0xA;
        let mut data = [0; 32];
        self.i2c
            .read(DEVICE_ADDRESS, &mut data[..18])
            .map_err(Error::I2C)?;
        let registers = to_registers(data, OFFSET);
        Ok(registers[Register::POWERCFG])
    }

    fn read_registers(&mut self) -> Result<[u16; 16], Error<E>> {
        const OFFSET: usize = 0xA;
        let mut data = [0; 32];
        self.i2c
            .read(DEVICE_ADDRESS, &mut data)
            .map_err(Error::I2C)?;
        let registers = to_registers(data, OFFSET);
        Ok(registers)
    }

    fn write_powercfg(&mut self, value: u16) -> Result<(), Error<E>> {
        let data = [(value >> 8) as u8, value as u8];
        self.i2c.write(DEVICE_ADDRESS, &data).map_err(Error::I2C)
    }

    fn write_registers(&mut self, registers: &[u16]) -> Result<(), Error<E>> {
        const OFFSET: usize = 0x2;
        let data = from_registers(registers, OFFSET);
        self.i2c
            .write(DEVICE_ADDRESS, &data[..((registers.len() - OFFSET) * 2)])
            .map_err(Error::I2C)
    }
}

fn to_registers(data: [u8; 32], offset: usize) -> [u16; 16] {
    let mut registers = [0; 16];
    for i in 0..registers.len() {
        registers[(i + offset) % registers.len()] =
            u16::from(data[2 * i]) << 8 | u16::from(data[2 * i + 1])
    }
    registers
}

fn from_registers(registers: &[u16], offset: usize) -> [u8; 32] {
    let mut data = [0; 32];
    for i in 0..registers.len() {
        let reg = registers[(i + offset) % registers.len()];
        data[2 * i] = (reg >> 8) as u8;
        data[2 * i + 1] = reg as u8
    }
    data
}

#[cfg(test)]
mod tests {
    extern crate embedded_hal_mock as hal;
    use super::*;
    const DATA: [u8; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14,
        0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    ];
    const REGS: [u16; 16] = [
        1, 0x203, 0x405, 0x607, 0x809, 0xA0B, 0xC0D, 0xE0F, 0x1011, 0x1213, 0x1415, 0x1617, 0x1819,
        0x1A1B, 0x1C1D, 0x1E1F,
    ];
    #[test]
    fn can_convert_to_registers() {
        let registers = to_registers(DATA, 0xA);
        const SHIFTED_REGS: [u16; 16] = [
            0xC0D, 0xE0F, 0x1011, 0x1213, 0x1415, 0x1617, 0x1819, 0x1A1B, 0x1C1D, 0x1E1F, 1, 0x203,
            0x405, 0x607, 0x809, 0xA0B,
        ];
        assert_eq!(registers, SHIFTED_REGS)
    }

    #[test]
    fn can_convert_from_registers() {
        let data = from_registers(&REGS, 0x2);
        const SHIFTED_DATA: [u8; 32] = [
            4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0, 1, 2, 3,
        ];
        assert_eq!(data, SHIFTED_DATA)
    }
}
