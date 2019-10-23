use super::{
    ic, Band, BitFlags, ChannelSpacing, DeEmphasis, Error, ErrorWithPin, Gpio1Config, Gpio2Config,
    Gpio3Config, OutputMode, Register, SeekDirection, SeekMode, SeekingState, Si4703,
    SoftmuteAttenuation, SoftmuteRate, StereoToMonoBlendLevel, Volume,
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

impl<I2C, E> Si4703<I2C, ic::Si4702>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    /// Create new instance of a Si4702 device
    pub fn new_si4702(i2c: I2C) -> Self {
        Si4703 {
            i2c,
            seeking_state: SeekingState::Idle,
            _ic: PhantomData,
        }
    }
}

impl<I2C, E> Si4703<I2C, ic::Si4703>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    /// Create new instance of a Si4703 device
    pub fn new(i2c: I2C) -> Self {
        Si4703 {
            i2c,
            seeking_state: SeekingState::Idle,
            _ic: PhantomData,
        }
    }
}

impl<I2C, IC> Si4703<I2C, IC> {
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E, IC> Si4703<I2C, IC>
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

    /// Enable softmute
    pub fn enable_softmute(
        &mut self,
        rate: SoftmuteRate,
        attenuation: SoftmuteAttenuation,
    ) -> Result<(), Error<E>> {
        let rate_mask = match rate {
            SoftmuteRate::Fastest => 0,
            SoftmuteRate::Fast => 1,
            SoftmuteRate::Slow => 2,
            SoftmuteRate::Slowest => 3,
        };
        let attenuation_mask = match attenuation {
            SoftmuteAttenuation::Db16 => 0,
            SoftmuteAttenuation::Db14 => 1,
            SoftmuteAttenuation::Db12 => 2,
            SoftmuteAttenuation::Db10 => 3,
        };

        let mut regs = self.read_registers()?;
        regs[Register::POWERCFG] &= !BitFlags::DSMUTE;
        regs[Register::SYSCONFIG3] &= 0x0FFF;
        regs[Register::SYSCONFIG3] |= (rate_mask << 14) | (attenuation_mask << 12);
        self.write_registers(&regs[0..=Register::SYSCONFIG3])
    }

    /// Disable softmute
    pub fn disable_softmute(&mut self) -> Result<(), Error<E>> {
        let powercfg = self.read_powercfg()?;
        self.write_powercfg(powercfg | BitFlags::DSMUTE)
    }

    /// Set output mode: Stereo/Mono
    pub fn set_output_mode(&mut self, mode: OutputMode) -> Result<(), Error<E>> {
        let powercfg = self.read_powercfg()?;
        let powercfg = match mode {
            OutputMode::Stereo => powercfg & !BitFlags::MONO,
            OutputMode::Mono => powercfg | BitFlags::MONO,
        };
        self.write_powercfg(powercfg)
    }

    fn get_powercfg_for_seek_config(
        powercfg: u16,
        mode: SeekMode,
        direction: SeekDirection,
    ) -> u16 {
        let powercfg = match mode {
            SeekMode::Wrap => powercfg | BitFlags::SKMODE,
            SeekMode::NoWrap => powercfg & !BitFlags::SKMODE,
        };
        let powercfg = match direction {
            SeekDirection::Up => powercfg | BitFlags::SEEKUP,
            SeekDirection::Down => powercfg & !BitFlags::SEEKUP,
        };
        powercfg
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

    /// Set the volume
    ///
    /// This will automatically activate or deactivate the extended volume
    /// range as appropriate.
    pub fn set_volume(&mut self, volume: Volume) -> Result<(), Error<E>> {
        let (volume_mask, volext) = match volume {
            Volume::Mute => (0_u16, false),
            Volume::Dbfsm58 => (1, true),
            Volume::Dbfsm56 => (2, true),
            Volume::Dbfsm54 => (3, true),
            Volume::Dbfsm52 => (4, true),
            Volume::Dbfsm50 => (5, true),
            Volume::Dbfsm48 => (6, true),
            Volume::Dbfsm46 => (7, true),
            Volume::Dbfsm44 => (8, true),
            Volume::Dbfsm42 => (9, true),
            Volume::Dbfsm40 => (10, true),
            Volume::Dbfsm38 => (11, true),
            Volume::Dbfsm36 => (12, true),
            Volume::Dbfsm34 => (13, true),
            Volume::Dbfsm32 => (14, true),
            Volume::Dbfsm30 => (15, true),
            Volume::Dbfsm28 => (1, false),
            Volume::Dbfsm26 => (2, false),
            Volume::Dbfsm24 => (3, false),
            Volume::Dbfsm22 => (4, false),
            Volume::Dbfsm20 => (5, false),
            Volume::Dbfsm18 => (6, false),
            Volume::Dbfsm16 => (7, false),
            Volume::Dbfsm14 => (8, false),
            Volume::Dbfsm12 => (9, false),
            Volume::Dbfsm10 => (10, false),
            Volume::Dbfsm8 => (11, false),
            Volume::Dbfsm6 => (12, false),
            Volume::Dbfsm4 => (13, false),
            Volume::Dbfsm2 => (14, false),
            Volume::Dbfs0 => (15, false),
        };
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG2] &= 0xFFF0;
        regs[Register::SYSCONFIG2] |= volume_mask;
        if volume_mask == 0 {
            self.write_registers(&regs[0..=Register::SYSCONFIG2])
        } else {
            if volext {
                regs[Register::SYSCONFIG3] |= BitFlags::VOLEXT
            } else {
                regs[Register::SYSCONFIG3] &= !BitFlags::VOLEXT
            }
            self.write_registers(&regs[0..=Register::SYSCONFIG3])
        }
    }

    /// Set band
    pub fn set_band(&mut self, band: Band) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match band {
            Band::Mhz875_108 => 0,
            Band::Mhz76_108 => 1,
            Band::Mhz76_90 => 2,
        };
        regs[Register::SYSCONFIG2] &= !(0b11 << 6);
        regs[Register::SYSCONFIG2] |= mask << 6;
        self.write_registers(&regs[..=Register::SYSCONFIG2])
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

    /// Set the GPIO1
    pub fn set_gpio1(&mut self, config: Gpio1Config) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match config {
            Gpio1Config::HighImpedance => 0,
            Gpio1Config::Low => 2,
            Gpio1Config::High => 3,
        };
        regs[Register::SYSCONFIG1] &= 0xFFFC;
        regs[Register::SYSCONFIG1] |= mask;
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

    /// Set the GPIO3
    pub fn set_gpio3(&mut self, config: Gpio3Config) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match config {
            Gpio3Config::HighImpedance => 0,
            Gpio3Config::MonoStereoIndicator => 1,
            Gpio3Config::Low => 2,
            Gpio3Config::High => 3,
        };
        regs[Register::SYSCONFIG1] &= 0xFFCF;
        regs[Register::SYSCONFIG1] |= mask << 4;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Enable automatic gain control (default).
    pub fn enable_auto_gain_control(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] &= !BitFlags::AGCD;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Disable automatic gain control.
    pub fn disable_auto_gain_control(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] |= BitFlags::AGCD;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Set the stereo to mono blend level
    pub fn set_stereo_to_mono_blend_level(
        &mut self,
        level: StereoToMonoBlendLevel,
    ) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let mask = match level {
            StereoToMonoBlendLevel::Dbuv31_49 => 0,
            StereoToMonoBlendLevel::Dbuv37_55 => 1,
            StereoToMonoBlendLevel::Dbuv19_37 => 2,
            StereoToMonoBlendLevel::Dbuv25_43 => 3,
        };
        regs[Register::SYSCONFIG1] &= 0xFF3F;
        regs[Register::SYSCONFIG1] |= mask << 6;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Seek
    ///
    /// It is not recommended to call this again this while the seeking
    /// is not finished. It should be waited on the STC interrupt pin.
    pub fn seek(&mut self, mode: SeekMode, direction: SeekDirection) -> nb::Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let seek = (regs[Register::POWERCFG] & BitFlags::SEEK) != 0;
        let stc = (regs[Register::STATUSRSSI] & BitFlags::STC) != 0;
        let failed = (regs[Register::STATUSRSSI] & BitFlags::SF_BL) != 0;

        match (self.seeking_state, seek, stc) {
            (SeekingState::Idle, false, false) => {
                let powercfg = regs[Register::POWERCFG] | BitFlags::SEEK;
                let powercfg = Self::get_powercfg_for_seek_config(powercfg, mode, direction);
                self.write_powercfg(powercfg).map_err(nb::Error::Other)?;
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
        mode: SeekMode,
        direction: SeekDirection,
        stc_int_pin: &P,
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
                    let powercfg = regs[Register::POWERCFG] | BitFlags::SEEK;
                    regs[Register::POWERCFG] =
                        Self::get_powercfg_for_seek_config(powercfg, mode, direction);
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

    /// Read the channel
    pub fn get_channel(&mut self) -> Result<f32, Error<E>> {
        let regs = self.read_registers()?;
        let spacing = (regs[Register::SYSCONFIG2] & (0b11 << 4)) >> 4;
        let spacing = match spacing {
            0 => 0.2,
            1 => 0.1,
            _ => 0.05,
        };
        let base = regs[Register::SYSCONFIG2] & (0b11 << 6);
        let base = if base == 0 { 87.5 } else { 76.0 };
        let channel = f32::from(regs[Register::READCHAN] & 0x3FF);
        Ok(channel * spacing + base)
    }

    /// Get the device ID
    ///
    /// Returns the (part number, manufacturer ID) as a tuple
    pub fn get_device_id(&mut self) -> Result<(u8, u16), Error<E>> {
        let regs = self.read_registers()?;
        let device_id = regs[Register::DEVICE_ID];
        let pn = ((device_id & 0xF000) >> 12) as u8;
        let mfid = device_id & 0xFFF;
        Ok((pn, mfid))
    }

    /// Get the chip ID
    ///
    /// Returns the (revision, device, firmware) as a tuple
    pub fn get_chip_id(&mut self) -> Result<(u8, u8, u8), Error<E>> {
        let regs = self.read_registers()?;
        let chip_id = regs[Register::CHIP_ID];
        let rev = ((chip_id & 0xFC00) >> 10) as u8;
        let dev = ((chip_id & 0x3C0) >> 6) as u8;
        let firmware = (chip_id & 0x3F) as u8;
        Ok((rev, dev, firmware))
    }
}
