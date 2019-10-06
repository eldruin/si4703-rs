use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use si470x::{ic, Si470x};

pub const DEV_ADDR: u8 = 0x10;

pub struct BitFlags;
impl BitFlags {
    pub const SKMODE: u16 = 1 << 10;
    pub const SEEKUP: u16 = 1 << 9;
    pub const SEEK: u16 = 1 << 8;
    pub const DE: u16 = 1 << 11;
    pub const STCIEN: u16 = 1 << 14;
    pub const STC: u16 = 1 << 14;
}

#[allow(unused)]
pub fn new_si4703(transactions: &[I2cTrans]) -> Si470x<I2cMock, ic::Si4703> {
    Si470x::new_si4703(I2cMock::new(transactions))
}

pub fn destroy<IC>(dev: Si470x<I2cMock, IC>) {
    dev.destroy().done();
}

#[macro_export]
macro_rules! assert_invalid_input_data {
    ($result:expr) => {
        match $result {
            Err(Error::InvalidInputData) => (),
            _ => panic!("InvalidInputData error not returned."),
        }
    };
}

#[macro_export]
macro_rules! set_invalid_test {
    ($name:ident, $create_method:ident, $method:ident $(, $value:expr)*) => {
        #[test]
        fn $name() {
            let mut dev = $create_method(&[]);
            assert_invalid_input_data!(dev.$method($($value),*));
            destroy(dev);
        }
    };
}
