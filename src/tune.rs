use super::{BitFlags, Error, ErrorWithPin, OperationState, Register, Si4703, TuneChannel};
use hal::blocking::i2c;
use hal::digital::v2::InputPin;

impl<I2C, E, IC> Si4703<I2C, IC>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    /// Tune to a certain frequency
    ///
    /// It is not recommended to call this again this while the tuning
    /// is not finished. It should be waited on the STC interrupt pin.
    pub fn tune(&mut self, channel: TuneChannel) -> nb::Result<(), Error<E>> {
        let set_initial_value = |regs: &mut [u16; 16]| {
            let raw = get_raw_tune_channel(regs[Register::SYSCONFIG2], channel)?;
            regs[Register::CHANNEL] = BitFlags::TUNE | raw;
            Ok(Register::CHANNEL)
        };
        let mut state = self.tuning_state;
        let result = self.tune_seek(
            Register::CHANNEL,
            BitFlags::TUNE,
            &mut state,
            &set_initial_value,
        );
        self.tuning_state = state;
        result
    }

    /// Tune using GPIO2 as STC interrupt pin (recommended)
    ///
    /// This will configure GPIO2 as STC interrupt pin and enable
    /// STC interrupts if appropriate.
    pub fn tune_with_stc_int_pin<PinE, P: InputPin<Error = PinE>>(
        &mut self,
        channel: TuneChannel,
        stc_int_pin: &P,
    ) -> nb::Result<(), ErrorWithPin<E, PinE>> {
        if self.tuning_state == OperationState::Busy
            && stc_int_pin
                .is_high()
                .map_err(ErrorWithPin::Pin)
                .map_err(nb::Error::Other)?
        {
            Err(nb::Error::WouldBlock)
        } else {
            let set_initial_value = |regs: &mut [u16; 16]| {
                let raw = get_raw_tune_channel(regs[Register::SYSCONFIG2], channel)?;
                regs[Register::CHANNEL] = BitFlags::TUNE | raw;
                let previous_sysconfig1 = regs[Register::SYSCONFIG1];
                regs[Register::SYSCONFIG1] &= 0xFFF3;
                regs[Register::SYSCONFIG1] |= 1 << 2;
                regs[Register::SYSCONFIG1] |= BitFlags::STCIEN;
                if previous_sysconfig1 != regs[Register::SYSCONFIG1] {
                    Ok(Register::SYSCONFIG1)
                } else {
                    Ok(Register::CHANNEL)
                }
            };
            let mut state = self.tuning_state;
            let result = self.tune_seek(
                Register::CHANNEL,
                BitFlags::TUNE,
                &mut state,
                &set_initial_value,
            );
            self.tuning_state = state;
            result.map_err(|e| match e {
                nb::Error::Other(e) => nb::Error::Other(e.into()),
                nb::Error::WouldBlock => nb::Error::WouldBlock,
            })
        }
    }
}

fn get_raw_tune_channel<E>(sysconfig2: u16, channel: TuneChannel) -> Result<u16, Error<E>> {
    match channel {
        TuneChannel::Raw(raw) if raw >= (1 << 10) => Err(Error::InvalidInputData),
        TuneChannel::Raw(raw) => Ok(raw),
        TuneChannel::Mhz(mhz) => {
            let (band_min, band_max) = match (sysconfig2 & (3 << 6)) >> 6 {
                0 => (87.5, 108.0),
                1 => (76.0, 108.0),
                _ => (76.0, 90.0),
            };
            if mhz < band_min || mhz > band_max {
                return Err(Error::InvalidInputData);
            }
            let spacing_mhz = match (sysconfig2 & (3 << 4)) >> 4 {
                0 => 0.2,
                1 => 0.1,
                _ => 0.05,
            };
            Ok(libm::floorf((mhz - band_min) / spacing_mhz) as u16)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! get_raw_tune_channel_eq {
        ($sysconfig2:expr, $channel:expr, $expected:expr) => {
            assert_eq!(
                $expected,
                get_raw_tune_channel::<()>($sysconfig2, $channel).unwrap()
            );
        };
    }

    #[test]
    fn get_raw_tune_channel_raw_correct() {
        get_raw_tune_channel_eq!(0, TuneChannel::Raw(0), 0);
        get_raw_tune_channel_eq!(0, TuneChannel::Raw(0x3FF), 0x3FF);
    }

    macro_rules! get_raw_tune_channel_error {
        ($sysconfig2:expr, $channel:expr) => {
            match get_raw_tune_channel::<()>($sysconfig2, $channel) {
                Err(Error::InvalidInputData) => (),
                _ => panic!("Error not returned."),
            }
        };
    }

    #[test]
    fn get_raw_tune_channel_raw_too_big() {
        get_raw_tune_channel_error!(0, TuneChannel::Raw(0b100_0000_0000));
    }

    #[test]
    fn get_raw_tune_channel_mhz_incorrect_europe_band() {
        get_raw_tune_channel_error!(0, TuneChannel::Mhz(87.4));
        get_raw_tune_channel_error!(0, TuneChannel::Mhz(108.1));
    }

    #[test]
    fn get_raw_tune_channel_mhz_incorrect_japan_wide_band() {
        get_raw_tune_channel_error!(1 << 6, TuneChannel::Mhz(75.9));
        get_raw_tune_channel_error!(1 << 6, TuneChannel::Mhz(108.1));
    }
    #[test]
    fn get_raw_tune_channel_mhz_incorrect_japan_band() {
        get_raw_tune_channel_error!(2 << 6, TuneChannel::Mhz(75.9));
        get_raw_tune_channel_error!(2 << 6, TuneChannel::Mhz(90.1));
    }

    #[test]
    fn get_raw_tune_channel_mhz_europe_band() {
        get_raw_tune_channel_eq!(0, TuneChannel::Mhz(87.5), 0);
        get_raw_tune_channel_eq!(0, TuneChannel::Mhz(88.0), 2);
        get_raw_tune_channel_eq!(0, TuneChannel::Mhz(108.0), 102);
    }

    #[test]
    fn get_raw_tune_channel_mhz_japan_wide_band() {
        get_raw_tune_channel_eq!(1 << 6, TuneChannel::Mhz(76.0), 0);
        get_raw_tune_channel_eq!(1 << 6, TuneChannel::Mhz(88.0), 60);
        get_raw_tune_channel_eq!(1 << 6, TuneChannel::Mhz(108.0), 160);
    }

    #[test]
    fn get_raw_tune_channel_mhz_japan_band() {
        get_raw_tune_channel_eq!(2 << 6, TuneChannel::Mhz(76.0), 0);
        get_raw_tune_channel_eq!(2 << 6, TuneChannel::Mhz(88.0), 60);
        get_raw_tune_channel_eq!(2 << 6, TuneChannel::Mhz(90.0), 70);
    }

    #[test]
    fn get_raw_tune_channel_mhz_spacing_is_considered() {
        get_raw_tune_channel_eq!(0, TuneChannel::Mhz(88.0), 2);
        get_raw_tune_channel_eq!(1 << 4, TuneChannel::Mhz(88.0), 5);
        get_raw_tune_channel_eq!(2 << 4, TuneChannel::Mhz(88.0), 10);
    }
}
