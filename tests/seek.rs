extern crate embedded_hal_mock as hal;
extern crate si470x;
#[macro_use]
extern crate nb;
use hal::i2c::Transaction as I2cTrans;
use hal::pin::{Mock as PinMock, State as PinState, Transaction as PinTrans};
use si470x::{Error, ErrorWithPin, SeekDirection, SeekMode};

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

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
fn can_fail_seeking() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[16] = (BF::SEEK >> 8) as u8;
    seeking_data[17] = BF::SEEK as u8;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = ((BF::STC | BF::SF_BL) >> 8) as u8;
    seeking_found_data[1] = (BF::STC | BF::SF_BL) as u8;
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
    assert_error!(block!(dev.seek()), Error::SeekFailed);
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

#[test]
fn can_fail_seeking_with_stc_int_pin() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[16] = (BF::SEEK >> 8) as u8;
    seeking_data[17] = BF::SEEK as u8;
    seeking_data[20] = (BF::STCIEN >> 8) as u8;
    seeking_data[21] = BF::STCIEN as u8 | 1 << 2;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = ((BF::STC | BF::SF_BL) >> 8) as u8;
    seeking_found_data[1] = (BF::STC | BF::SF_BL) as u8;
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
    assert_error!(
        block!(dev.seek_with_stc_int_pin(&mut pin)),
        ErrorWithPin::SeekFailed
    );
    destroy(dev);
    pin.done();
}
