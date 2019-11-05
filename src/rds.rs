use crate::{marker, BitFlags, Error, RdsMode, Register, Si4703};
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

    /// Get RDS whether a new RDS group is ready.
    pub fn rds_ready(&mut self) -> Result<bool, Error<E>> {
        let status = self.read_status()?;
        Ok((status & BitFlags::RDSR) != 0)
    }

    /// Get RDS synchronization status (only available in RDS verbose mode).
    pub fn rds_synchronized(&mut self) -> Result<bool, Error<E>> {
        let status = self.read_status()?;
        Ok((status & BitFlags::RDSS) != 0)
    }
}
