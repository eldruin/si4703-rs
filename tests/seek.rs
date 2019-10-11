extern crate embedded_hal_mock as hal;
extern crate si4703;
#[macro_use]
extern crate nb;
use hal::i2c::Transaction as I2cTrans;
use hal::pin::{Mock as PinMock, State as PinState, Transaction as PinTrans};
use si4703::{Error, ErrorWithPin, SeekDirection, SeekMode};

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

macro_rules! seek_test {
    ($name:ident, $mode:ident, $direction:ident, $powercfg:expr) => {
        #[test]
        fn $name() {
            let powercfg = $powercfg | BF::SEEK;
            let mut found_data = [0; 32];
            found_data[0] = (BF::STC >> 8) as u8;
            found_data[1] = BF::STC as u8;
            let mut seeking_data = [0; 32];
            seeking_data[16] = (powercfg >> 8) as u8;
            seeking_data[17] = powercfg as u8;
            let mut seeking_found_data = [0; 32];
            seeking_found_data[0] = (BF::STC >> 8) as u8;
            seeking_found_data[1] = BF::STC as u8;
            seeking_found_data[16] = (powercfg >> 8) as u8;
            seeking_found_data[17] = powercfg as u8;
            let transactions = [
                I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
                I2cTrans::write(DEV_ADDR, vec![(powercfg >> 8) as u8, powercfg as u8]),
                I2cTrans::read(DEV_ADDR, seeking_data.to_vec()),
                I2cTrans::read(DEV_ADDR, seeking_found_data.to_vec()),
                I2cTrans::write(
                    DEV_ADDR,
                    vec![
                        ((powercfg & !BF::SEEK) >> 8) as u8,
                        (powercfg & !BF::SEEK) as u8,
                    ],
                ),
                I2cTrans::read(DEV_ADDR, found_data.to_vec()),
                I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
            ];
            let mut dev = new_si4703(&transactions);
            block!(dev.seek(SeekMode::$mode, SeekDirection::$direction)).unwrap();
            destroy(dev);
        }
    };
}

seek_test!(seek_nowrap_down, NoWrap, Down, 0);
seek_test!(seek_wrap_down, Wrap, Down, BF::SKMODE);
seek_test!(seek_nowrap_up, NoWrap, Up, BF::SEEKUP);
seek_test!(seek_wrap_up, Wrap, Up, BF::SKMODE | BF::SEEKUP);

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
    block!(dev.seek(SeekMode::NoWrap, SeekDirection::Down)).unwrap();
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
    assert_error!(
        block!(dev.seek(SeekMode::NoWrap, SeekDirection::Down)),
        Error::SeekFailed
    );
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
    block!(dev.seek_with_stc_int_pin(SeekMode::NoWrap, SeekDirection::Down, &pin)).unwrap();
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
        block!(dev.seek_with_stc_int_pin(SeekMode::NoWrap, SeekDirection::Down, &pin)),
        ErrorWithPin::SeekFailed
    );
    destroy(dev);
    pin.done();
}
