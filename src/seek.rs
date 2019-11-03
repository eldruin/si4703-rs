use super::{
    BitFlags, Error, ErrorWithPin, OperationState, Register, SeekDirection, SeekFmImpulseThreshold,
    SeekMode, SeekSnrThreshold, Si4703,
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
        match direction {
            SeekDirection::Up => powercfg | BitFlags::SEEKUP,
            SeekDirection::Down => powercfg & !BitFlags::SEEKUP,
        }
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
        let set_initial_value = |regs: &mut [u16; 16]| {
            let powercfg = regs[Register::POWERCFG] | BitFlags::SEEK;
            regs[Register::POWERCFG] =
                Self::get_powercfg_for_seek_config(powercfg, mode, direction);
            Ok(Register::POWERCFG)
        };
        let mut state = self.seeking_state;
        let result = self.tune_seek(
            Register::POWERCFG,
            BitFlags::SEEK,
            &mut state,
            &set_initial_value,
        );
        self.seeking_state = state;
        result
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
        if self.seeking_state == OperationState::Busy
            && stc_int_pin
                .is_high()
                .map_err(ErrorWithPin::Pin)
                .map_err(nb::Error::Other)?
        {
            Err(nb::Error::WouldBlock)
        } else {
            let set_initial_value = |regs: &mut [u16; 16]| {
                let powercfg = regs[Register::POWERCFG] | BitFlags::SEEK;
                regs[Register::POWERCFG] =
                    Self::get_powercfg_for_seek_config(powercfg, mode, direction);
                let previous_sysconfig1 = regs[Register::SYSCONFIG1];
                regs[Register::SYSCONFIG1] &= 0xFFF3;
                regs[Register::SYSCONFIG1] |= 1 << 2;
                regs[Register::SYSCONFIG1] |= BitFlags::STCIEN;
                if previous_sysconfig1 != regs[Register::SYSCONFIG1] {
                    Ok(Register::SYSCONFIG1)
                } else {
                    Ok(Register::POWERCFG)
                }
            };
            let mut state = self.seeking_state;
            let result = self.tune_seek(
                Register::POWERCFG,
                BitFlags::SEEK,
                &mut state,
                &set_initial_value,
            );
            self.seeking_state = state;
            result.map_err(|e| match e {
                nb::Error::Other(e) => nb::Error::Other(e.into()),
                nb::Error::WouldBlock => nb::Error::WouldBlock,
            })
        }
    }

    pub(crate) fn tune_seek(
        &mut self,
        register: usize,
        bitflag: u16,
        state: &mut OperationState,
        set_start_value: &dyn Fn(&mut [u16; 16]) -> Result<usize, Error<E>>,
    ) -> nb::Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        let flag = (regs[register] & bitflag) != 0;
        let stc = (regs[Register::STATUSRSSI] & BitFlags::STC) != 0;
        let failed = (regs[Register::STATUSRSSI] & BitFlags::SF_BL) != 0;
        let afcrl = (regs[Register::STATUSRSSI] & BitFlags::AFCRL) != 0;

        match (*state, flag, stc) {
            (OperationState::Idle, false, false) => {
                let register = set_start_value(&mut regs)?;
                self.write_registers(&regs[..=register])?;
                *state = OperationState::Busy;
                Err(nb::Error::WouldBlock)
            }
            (OperationState::Busy, true, true) => {
                regs[register] &= !bitflag;
                self.write_registers(&regs[..=register])?;
                *state = OperationState::WaitingForStcToClear(!failed && !afcrl);
                Err(nb::Error::WouldBlock)
            }
            (OperationState::WaitingForStcToClear(success), false, false) => {
                *state = OperationState::Idle;
                if success {
                    Ok(())
                } else {
                    Err(nb::Error::Other(Error::SeekFailed))
                }
            }
            (_, _, _) => Err(nb::Error::WouldBlock),
        }
    }
}
