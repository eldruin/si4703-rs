extern crate embedded_hal_mock as hal;
extern crate si470x;
#[macro_use]
extern crate nb;
use hal::i2c::Transaction as I2cTrans;
use hal::pin::{Mock as PinMock, State as PinState, Transaction as PinTrans};
use si470x::{DeEmphasis, Error, Gpio2Config, SeekDirection, SeekMode};

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

macro_rules! write_powercfg_test {
    ($name:ident, $value:expr, $method:ident $(, $arg:expr)*) => {
        write_test!($name, $value, 9, 1, $method $(, $arg)*);
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

write_test!(set_de_75, 0, 16, 3, set_deemphasis, DeEmphasis::Us75);
write_test!(set_de_50, BF::DE, 16, 3, set_deemphasis, DeEmphasis::Us50);

write_test!(set_vol_min, 0, 16, 4, set_volume, 0);
write_test!(set_vol_max, 0xF, 16, 4, set_volume, 0xF);
set_invalid_test!(cannot_set_too_high_vol, new_si4703, set_volume, 0x10);

const STC_RDS_INT: Gpio2Config = Gpio2Config::StcRdsInterrupt;
write_test!(gpio2_hi, 0, 16, 3, set_gpio2, Gpio2Config::HighImpedance);
write_test!(gpio2_int, 1 << 2, 16, 3, set_gpio2, STC_RDS_INT);
write_test!(gpio2_high, 3 << 2, 16, 3, set_gpio2, Gpio2Config::High);
write_test!(gpio2_low, 2 << 2, 16, 3, set_gpio2, Gpio2Config::Low);

write_test!(dis_stci, 0, 16, 3, disable_stc_interrupts);
write_test!(en_stci, BF::STCIEN, 16, 3, enable_stc_interrupts);

#[test]
fn can_seek() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[16] = (BF::SEEK >> 8) as u8;
    seeking_data[17] = BF::SEEK as u8;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = (BF::STC >> 8) as u8;
    seeking_found_data[1] = BF::STC as u8;
    seeking_found_data[16] = (BF::SEEK >> 8) as u8;
    seeking_found_data[17] = BF::SEEK as u8;
    let transactions = [
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
        I2cTrans::write(DEV_ADDR, vec![(BF::SEEK >> 8) as u8, BF::SEEK as u8]),
        I2cTrans::read(DEV_ADDR, seeking_data.to_vec()),
        I2cTrans::read(DEV_ADDR, seeking_found_data.to_vec()),
        I2cTrans::write(DEV_ADDR, vec![0, 0]),
        I2cTrans::read(DEV_ADDR, found_data.to_vec()),
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
    ];
    let mut dev = new_si4703(&transactions);
    block!(dev.seek()).unwrap();
    destroy(dev);
}

#[test]
fn can_seek_with_stc_int_pin() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[16] = (BF::SEEK >> 8) as u8;
    seeking_data[17] = BF::SEEK as u8;
    seeking_data[20] = (BF::STCIEN >> 8) as u8;
    seeking_data[21] = BF::STCIEN as u8 | 1 << 2;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = (BF::STC >> 8) as u8;
    seeking_found_data[1] = BF::STC as u8;
    seeking_found_data[16] = (BF::SEEK >> 8) as u8;
    seeking_found_data[17] = BF::SEEK as u8;
    let transactions = [
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            vec![
                (BF::SEEK >> 8) as u8,
                BF::SEEK as u8,
                0,
                0,
                (BF::STCIEN >> 8) as u8,
                BF::STCIEN as u8 | 1 << 2,
            ],
        ),
        I2cTrans::read(DEV_ADDR, seeking_data.to_vec()),
        // this time STC bit is (incorrectly) not set
        I2cTrans::read(DEV_ADDR, seeking_found_data.to_vec()),
        I2cTrans::write(DEV_ADDR, vec![0, 0]),
        I2cTrans::read(DEV_ADDR, found_data.to_vec()),
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
    ];
    let pin_trans = [
        PinTrans::get(PinState::High),
        PinTrans::get(PinState::Low), // this time STC bit is (incorrectly) not set
        PinTrans::get(PinState::Low),
    ];
    let mut pin = PinMock::new(&pin_trans);
    let mut dev = new_si4703(&transactions);
    block!(dev.seek_with_stc_int_pin(&mut pin)).unwrap();
    destroy(dev);
    pin.done()
}
