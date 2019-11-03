use super::{
    BitFlags, Error, ErrorWithPin, Register, SeekDirection, SeekFmImpulseThreshold, SeekMode,
    SeekSnrThreshold, SeekingState, Si4703,
};
use hal::blocking::i2c;
use hal::digital::v2::InputPin;

impl<I2C, E, IC> Si4703<I2C, IC>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
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

    /// Configure seek RSSI, SNR and FM impulse detection thresholds
    pub fn configure_seek(
        &mut self,
        rssi_threshold: u8,
        snr_threshold: SeekSnrThreshold,
        fm_impulse_threshold: SeekFmImpulseThreshold,
    ) -> Result<(), Error<E>> {
        let snr_mask = match snr_threshold {
            SeekSnrThreshold::Disabled => 0,
            SeekSnrThreshold::Enabled(v) if v > 7 || v == 0 => return Err(Error::InvalidInputData),
            SeekSnrThreshold::Enabled(v) => v << 4,
        };
        let cnt_mask = match fm_impulse_threshold {
            SeekFmImpulseThreshold::Disabled => 0,
            SeekFmImpulseThreshold::Enabled(v) if v > 15 || v == 0 => {
                return Err(Error::InvalidInputData)
            }
            SeekFmImpulseThreshold::Enabled(v) => v,
        };
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG2] &= 0xFF00;
        regs[Register::SYSCONFIG2] |= u16::from(rssi_threshold) << 8;
        regs[Register::SYSCONFIG3] &= 0xFF00;
        regs[Register::SYSCONFIG3] |= u16::from(snr_mask | cnt_mask);
        self.write_registers(&regs[..=Register::SYSCONFIG3])
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
        let afcrl = (regs[Register::STATUSRSSI] & BitFlags::AFCRL) != 0;

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
                self.seeking_state = SeekingState::WaitingForStcToClear(!failed && !afcrl);
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
            let afcrl = (regs[Register::STATUSRSSI] & BitFlags::AFCRL) != 0;

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
                    self.seeking_state = SeekingState::WaitingForStcToClear(!failed && !afcrl);
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
