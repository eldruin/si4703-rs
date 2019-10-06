use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use si470x::{ic, Si470x};

pub const DEV_ADDR: u8 = 0x10;

pub struct BitFlags;
impl BitFlags {
    pub const DE: u16 = 1 << 11;
}

#[allow(unused)]
pub fn new_si4703(transactions: &[I2cTrans]) -> Si470x<I2cMock, ic::Si4703> {
    Si470x::new_si4703(I2cMock::new(&transactions))
}

pub fn destroy<IC>(dev: Si470x<I2cMock, IC>) {
    dev.destroy().done();
}
