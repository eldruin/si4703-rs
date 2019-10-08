use super::{
    ic, BitFlags, ChannelSpacing, DeEmphasis, Error, ErrorWithPin, Gpio2Config, Register,
    SeekDirection, SeekMode, SeekingState, Si470x,
};
use core::marker::PhantomData;
use hal::blocking::delay::DelayMs;
use hal::blocking::i2c;
use hal::digital::v2::InputPin;
use hal::digital::v2::OutputPin;

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
            seeking_state: SeekingState::Idle,
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

    /// Set channel spacing
    pub fn set_channel_spacing(&mut self, spacing: ChannelSpacing) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match spacing {
            ChannelSpacing::Khz200 => 0,
            ChannelSpacing::Khz100 => 1,
            ChannelSpacing::Khz50 => 2,
        };
        regs[Register::SYSCONFIG2] &= !(0b11 << 4);
        regs[Register::SYSCONFIG2] |= mask << 4;
        self.write_registers(&regs[..=Register::SYSCONFIG2])
    }

    /// Enable generating STC interrupts.
    ///
    /// For this to be useful, gpio2 pin must be configured to be
    /// used as STC/RDS interrupt.
    pub fn enable_stc_interrupts(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] |= BitFlags::STCIEN;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Disable generating STC interrupts.
    pub fn disable_stc_interrupts(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] &= !BitFlags::STCIEN;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
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

    /// Seek
    ///
    /// It is not recommended to call this again this while the seeking
    /// is not finished. It should be waited on the STC interrupt pin.
    pub fn seek(&mut self) -> nb::Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let seek = (regs[Register::POWERCFG] & BitFlags::SEEK) != 0;
        let stc = (regs[Register::STATUSRSSI] & BitFlags::STC) != 0;
        let failed = (regs[Register::STATUSRSSI] & BitFlags::SF_BL) != 0;

        match (self.seeking_state, seek, stc) {
            (SeekingState::Idle, false, false) => {
                regs[Register::POWERCFG] |= BitFlags::SEEK;
                self.write_powercfg(regs[Register::POWERCFG])
                    .map_err(nb::Error::Other)?;
                self.seeking_state = SeekingState::Seeking;
                Err(nb::Error::WouldBlock)
            }
            (SeekingState::Seeking, true, true) => {
                regs[Register::POWERCFG] &= !(BitFlags::SEEK);
                self.write_powercfg(regs[Register::POWERCFG])
                    .map_err(nb::Error::Other)?;
                self.seeking_state = SeekingState::WaitingForStcToClear(!failed);
                Err(nb::Error::WouldBlock)
            }
            (SeekingState::WaitingForStcToClear(success), false, false) => {
                self.seeking_state = SeekingState::Idle;
                if success {
                    Ok(())
                } else {
                    Err(nb::Error::Other(Error::SeekFailed))
                }
            }
            (_, _, _) => Err(nb::Error::WouldBlock),
        }
    }

    /// Seek using GPIO2 as STC interrupt pin (recommended)
    ///
    /// This will configure GPIO2 as STC interrupt pin and enable
    /// STC interrupts if appropriate.
    pub fn seek_with_stc_int_pin<PinE, P: InputPin<Error = PinE>>(
        &mut self,
        stc_int_pin: &mut P,
    ) -> nb::Result<(), ErrorWithPin<E, PinE>> {
        if self.seeking_state == SeekingState::Seeking
            && stc_int_pin
                .is_high()
                .map_err(ErrorWithPin::Pin)
                .map_err(nb::Error::Other)?
        {
            Err(nb::Error::WouldBlock)
        } else {
            let mut regs = self.read_registers_bare_err().map_err(ErrorWithPin::I2C)?;
            let seek = (regs[Register::POWERCFG] & BitFlags::SEEK) != 0;
            let stc = (regs[Register::STATUSRSSI] & BitFlags::STC) != 0;
            let failed = (regs[Register::STATUSRSSI] & BitFlags::SF_BL) != 0;

            match (self.seeking_state, seek, stc) {
                (SeekingState::Idle, false, false) => {
                    regs[Register::POWERCFG] |= BitFlags::SEEK;
                    let previous_sysconfig1 = regs[Register::SYSCONFIG1];
                    regs[Register::SYSCONFIG1] &= 0xFFF3;
                    regs[Register::SYSCONFIG1] |= 1 << 2;
                    regs[Register::SYSCONFIG1] |= BitFlags::STCIEN;
                    if previous_sysconfig1 != regs[Register::SYSCONFIG1] {
                        self.write_registers_bare_err(&regs[..=Register::SYSCONFIG1])
                            .map_err(ErrorWithPin::I2C)
                            .map_err(nb::Error::Other)?;
                    } else {
                        self.write_powercfg_bare_err(regs[Register::POWERCFG])
                            .map_err(ErrorWithPin::I2C)
                            .map_err(nb::Error::Other)?;
                    }
                    self.seeking_state = SeekingState::Seeking;
                    Err(nb::Error::WouldBlock)
                }
                (SeekingState::Seeking, true, true) => {
                    regs[Register::POWERCFG] &= !(BitFlags::SEEK);
                    self.write_powercfg_bare_err(regs[Register::POWERCFG])
                        .map_err(ErrorWithPin::I2C)
                        .map_err(nb::Error::Other)?;
                    self.seeking_state = SeekingState::WaitingForStcToClear(!failed);
                    Err(nb::Error::WouldBlock)
                }
                (SeekingState::WaitingForStcToClear(success), false, false) => {
                    self.seeking_state = SeekingState::Idle;
                    if success {
                        Ok(())
                    } else {
                        Err(nb::Error::Other(ErrorWithPin::SeekFailed))
                    }
                }
                (_, _, _) => Err(nb::Error::WouldBlock),
            }
        }
    }
}
