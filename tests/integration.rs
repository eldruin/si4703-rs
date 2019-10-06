extern crate embedded_hal_mock as hal;
extern crate si470x;
use hal::i2c::Transaction as I2cTrans;
use si470x::{DeEmphasis, Error, SeekDirection, SeekMode};

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

const DATA: [u8; 32] = [
    0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13,
];

#[test]
fn can_enable_osc() {
    let transactions = [
        I2cTrans::read(DEV_ADDR, DATA.to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            vec![4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0x81, 0x0],
        ),
    ];
    let mut dev = new_si4703(&transactions);
    dev.enable_oscillator().unwrap();
    destroy(dev);
}

#[test]
fn can_enable() {
    let transactions = [I2cTrans::write(DEV_ADDR, vec![0, 1])];
    let mut dev = new_si4703(&transactions);
    dev.enable().unwrap();
    destroy(dev);
}

macro_rules! write_register_test {
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
}

macro_rules! write_powercfg_test {
    ($name:ident, $value:expr, $method:ident $(, $arg:expr)*) => {
        write_register_test!($name, $value, 9, 1, $method $(, $arg)*);
    };
}

write_powercfg_test!(can_unmute, 0x4000_u16, unmute);
write_powercfg_test!(can_mute, 0x0, mute);
write_powercfg_test!(
    config_seek_nowrap_down,
    0x0,
    configure_seek,
    SeekMode::NoWrap,
    SeekDirection::Down
);
write_powercfg_test!(
    config_seek_wrap_down,
    BF::SKMODE,
    configure_seek,
    SeekMode::Wrap,
    SeekDirection::Down
);
write_powercfg_test!(
    config_seek_nowrap_up,
    BF::SEEKUP,
    configure_seek,
    SeekMode::NoWrap,
    SeekDirection::Up
);
write_powercfg_test!(
    config_seek_wrap_up,
    BF::SKMODE | BF::SEEKUP,
    configure_seek,
    SeekMode::Wrap,
    SeekDirection::Up
);

write_register_test!(set_de_75, 0, 16, 3, set_deemphasis, DeEmphasis::Us75);
write_register_test!(set_de_50, BF::DE, 16, 3, set_deemphasis, DeEmphasis::Us50);

write_register_test!(set_vol_min, 0, 16, 4, set_volume, 0);
write_register_test!(set_vol_max, 0xF, 16, 4, set_volume, 0xF);
set_invalid_test!(cannot_set_too_high_vol, new_si4703, set_volume, 0x10);
