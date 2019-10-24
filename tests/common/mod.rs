use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use si4703::{ic, Si4703};

pub const DEV_ADDR: u8 = 0x10;

pub struct BitFlags;
#[allow(unused)]
impl BitFlags {
    pub const DSMUTE: u16 = 1 << 15;
    pub const DMUTE: u16 = 1 << 14;
    pub const MONO: u16 = 1 << 13;
    pub const SKMODE: u16 = 1 << 10;
    pub const SEEKUP: u16 = 1 << 9;
    pub const SEEK: u16 = 1 << 8;
    pub const DE: u16 = 1 << 11;
    pub const STCIEN: u16 = 1 << 14;
    pub const STC: u16 = 1 << 14;
    pub const SF_BL: u16 = 1 << 13;
    pub const RDS: u16 = 1 << 12;
    pub const AGCD: u16 = 1 << 10;
    pub const RDSM: u16 = 1 << 11;
    pub const VOLEXT: u16 = 1 << 8;
}

#[allow(unused)]
pub fn new_si4703(transactions: &[I2cTrans]) -> Si4703<I2cMock, ic::Si4703> {
    Si4703::new(I2cMock::new(transactions))
}

pub fn destroy<IC>(dev: Si4703<I2cMock, IC>) {
    dev.destroy().done();
}

#[macro_export]
macro_rules! assert_error {
    ($result:expr, $error:ident::$variant:ident) => {
        match $result {
            Err($error::$variant) => (),
            _ => panic!("Error not returned."),
        }
    };
}

#[macro_export]
macro_rules! set_invalid_test {
    ($name:ident, $create_method:ident, $error:ident::$variant:ident, $method:ident $(, $value:expr)*) => {
        #[test]
        fn $name() {
            let mut dev = $create_method(&[]);
            assert_error!(dev.$method($($value),*), $error::$variant);
            destroy(dev);
        }
    };
}

#[macro_export]
macro_rules! write_test {
    ($name:ident, $value:expr, $read_reg_count:expr, $write_reg_count:expr, $method:ident $(, $arg:expr)*) => {
        #[test]
        fn $name() {
            let mut write = [0; $write_reg_count*2];
            write[($write_reg_count-1)*2] = ($value >> 8) as u8;
            write[($write_reg_count-1)*2+1] = $value as u8;
            let transactions = [
                I2cTrans::read(DEV_ADDR, [0;$read_reg_count*2].to_vec()),
                I2cTrans::write(DEV_ADDR, write.to_vec())];
            let mut dev = new_si4703(&transactions);
            dev.$method($($arg),*).unwrap();
            destroy(dev);
        }
    };
    ($name:ident, $read_reg_count:expr, $first_write_reg_index:expr, $first_write_reg_value:expr,
     $second_write_reg_index:expr, $second_write_reg_value:expr, $method:ident $(, $arg:expr)*) => {
        #[test]
        fn $name() {
            let mut write = [0; ($second_write_reg_index+1)*2];
            write[$first_write_reg_index*2] = ($first_write_reg_value >> 8) as u8;
            write[$first_write_reg_index*2+1] = $first_write_reg_value as u8;
            write[$second_write_reg_index*2] = ($second_write_reg_value >> 8) as u8;
            write[$second_write_reg_index*2+1] = $second_write_reg_value as u8;
            let transactions = [
                I2cTrans::read(DEV_ADDR, [0;$read_reg_count*2].to_vec()),
                I2cTrans::write(DEV_ADDR, write.to_vec())];
            let mut dev = new_si4703(&transactions);
            dev.$method($($arg),*).unwrap();
            destroy(dev);
        }
    };
}

#[macro_export]
macro_rules! write_powercfg_test {
    ($name:ident, $value:expr, $method:ident $(, $arg:expr)*) => {
        write_test!($name, $value, 9, 1, $method $(, $arg)*);
    };
}
