use crate::{
    marker, BitFlags, Error, RdsBlockData, RdsBlockErrors, RdsData, RdsMode, RdsRadioText,
    RdsRadioTextData, Register, Si4703,
};
use embedded_hal::blocking::i2c;

impl<I2C, E, IC> Si4703<I2C, IC>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
    IC: marker::WithRds,
{
    /// Enable RDS.
    pub fn enable_rds(&mut self, mode: RdsMode) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] |= BitFlags::RDS;
        match mode {
            RdsMode::Standard => regs[Register::POWERCFG] &= !BitFlags::RDSM,
            RdsMode::Verbose => regs[Register::POWERCFG] |= BitFlags::RDSM,
        }
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Disable RDS.
    pub fn disable_rds(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] &= !BitFlags::RDS;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Enable RDS interrupts.
    pub fn enable_rds_interrupts(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] |= BitFlags::RDSIEN;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Disable RDS interrupts.
    pub fn disable_rds_interrupts(&mut self) -> Result<(), Error<E>> {
        let mut regs = self.read_registers()?;
        regs[Register::SYSCONFIG1] &= !BitFlags::RDSIEN;
        self.write_registers(&regs[0..=Register::SYSCONFIG1])
    }

    /// Get whether a new RDS group is ready.
    pub fn rds_ready(&mut self) -> Result<bool, Error<E>> {
        let status = self.read_status()?;
        Ok((status & BitFlags::RDSR) != 0)
    }

    /// Get RDS synchronization status (only available in RDS verbose mode).
    pub fn rds_synchronized(&mut self) -> Result<bool, Error<E>> {
        let status = self.read_status()?;
        Ok((status & BitFlags::RDSS) != 0)
    }

    /// Get RDS data.
    pub fn rds_data(&mut self) -> Result<RdsData, Error<E>> {
        let regs = self.read_rds()?;
        let status = regs[Register::STATUSRSSI];
        let blera = get_block_errors(status, BitFlags::BLERA1, BitFlags::BLERA0);
        let readchan = regs[Register::READCHAN];
        let blerb = get_block_errors(readchan, BitFlags::BLERB1, BitFlags::BLERB0);
        let blerc = get_block_errors(readchan, BitFlags::BLERC1, BitFlags::BLERC0);
        let blerd = get_block_errors(readchan, BitFlags::BLERD1, BitFlags::BLERD0);
        Ok(RdsData {
            a: RdsBlockData {
                data: regs[Register::RDSA],
                errors: blera,
            },
            b: RdsBlockData {
                data: regs[Register::RDSB],
                errors: blerb,
            },
            c: RdsBlockData {
                data: regs[Register::RDSC],
                errors: blerc,
            },
            d: RdsBlockData {
                data: regs[Register::RDSD],
                errors: blerd,
            },
        })
    }
}

fn get_block_errors(data: u16, bitmask1: u16, bitmask0: u16) -> RdsBlockErrors {
    match ((data & bitmask1) != 0, (data & bitmask0) != 0) {
        (false, false) => RdsBlockErrors::None,
        (false, true) => RdsBlockErrors::OneOrTwo,
        (true, false) => RdsBlockErrors::ThreeToFive,
        (true, true) => RdsBlockErrors::TooMany,
    }
}

/// Fill char array with radio text with the RDS data.
///
/// This needs to be called repeatedly with each new data package.
/// Will return true when a screen clear was requested.
pub fn fill_with_rds_radio_text(text: &mut [char; 64], data: &RdsData) -> bool {
    if let Some(data) = get_rds_radio_text(data) {
        if let Some((text_data, offset)) = data.text {
            match text_data {
                RdsRadioTextData::Two(a, b) => {
                    text[offset] = a;
                    text[offset + 1] = b;
                }
                RdsRadioTextData::Four(a, b, c, d) => {
                    text[offset] = a;
                    text[offset + 1] = b;
                    text[offset + 2] = c;
                    text[offset + 3] = d;
                }
            }
        }
        data.screen_clear
    } else {
        false
    }
}

/// Get radio text from RDS data.
pub fn get_rds_radio_text(data: &RdsData) -> Option<RdsRadioText> {
    // See Radio Text here: https://en.wikipedia.org/wiki/Radio_Data_System
    const RADIO_TEXT_GTYPE: u16 = 0x2 << 12;
    if data.b.errors == RdsBlockErrors::None || data.b.errors == RdsBlockErrors::OneOrTwo {
        let gtype = data.b.data & (0xF << 12);
        if gtype == RADIO_TEXT_GTYPE {
            let should_clear = (data.b.data & (1 << 4)) != 0;
            let segment_offset = (data.b.data & 0xF) as usize;
            let b0 = data.b.data & (1 << 11);
            if b0 == 0 {
                if data.c.errors != RdsBlockErrors::TooMany
                    && data.d.errors != RdsBlockErrors::TooMany
                {
                    return Some(RdsRadioText {
                        screen_clear: should_clear,
                        text: Some((
                            RdsRadioTextData::Four(
                                ((data.c.data & 0xFF00) >> 8) as u8 as char,
                                (data.c.data & 0xFF) as u8 as char,
                                ((data.d.data & 0xFF00) >> 8) as u8 as char,
                                (data.d.data & 0xFF) as u8 as char,
                            ),
                            segment_offset * 4,
                        )),
                    });
                }
            } else if data.d.errors != RdsBlockErrors::TooMany {
                return Some(RdsRadioText {
                    screen_clear: should_clear,
                    text: Some((
                        RdsRadioTextData::Two(
                            ((data.d.data & 0xFF00) >> 8) as u8 as char,
                            (data.d.data & 0xFF) as u8 as char,
                        ),
                        segment_offset * 2,
                    )),
                });
            }

            return Some(RdsRadioText {
                screen_clear: should_clear,
                text: None,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_block_errors_none() {
        assert_eq!(RdsBlockErrors::None, get_block_errors(0, 2, 1));
    }

    #[test]
    fn get_block_errors_one() {
        assert_eq!(RdsBlockErrors::OneOrTwo, get_block_errors(1, 2, 1));
    }

    #[test]
    fn get_block_errors_three() {
        assert_eq!(RdsBlockErrors::ThreeToFive, get_block_errors(2, 2, 1));
    }

    #[test]
    fn get_block_errors_too_many() {
        assert_eq!(RdsBlockErrors::TooMany, get_block_errors(3, 2, 1));
    }
}
