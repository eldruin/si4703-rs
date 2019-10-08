use super::{marker, BitFlags, Error, RdsMode, Register, Si470x};

use hal::blocking::i2c;

impl<I2C, E, IC> Si470x<I2C, IC>
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
}