use embedded_hal_mock::{
    i2c::Transaction as I2cTrans,
    pin::{Mock as PinMock, State as PinState, Transaction as PinTrans},
};
use nb::block;
use si4703::TuneChannel;

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

#[test]
fn can_tune() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[18] = (BF::TUNE >> 8) as u8;
    seeking_data[19] = BF::TUNE as u8 | 2;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = (BF::STC >> 8) as u8;
    seeking_found_data[1] = BF::STC as u8;
    seeking_found_data[18] = (BF::TUNE >> 8) as u8;
    seeking_found_data[19] = BF::TUNE as u8 | 2;
    let transactions = [
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            vec![0, 0, (BF::TUNE >> 8) as u8, BF::TUNE as u8 | 2],
        ),
        I2cTrans::read(DEV_ADDR, seeking_data.to_vec()),
        I2cTrans::read(DEV_ADDR, seeking_found_data.to_vec()),
        I2cTrans::write(DEV_ADDR, vec![0, 0, 0, 2]),
        I2cTrans::read(DEV_ADDR, found_data.to_vec()),
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
    ];
    let mut dev = new_si4703(&transactions);
    block!(dev.tune(TuneChannel::Raw(2))).unwrap();
    destroy(dev);
}

#[test]
fn can_tune_with_stc_int_pin() {
    let mut found_data = [0; 32];
    found_data[0] = (BF::STC >> 8) as u8;
    found_data[1] = BF::STC as u8;
    let mut seeking_data = [0; 32];
    seeking_data[18] = (BF::TUNE >> 8) as u8;
    seeking_data[19] = BF::TUNE as u8 | 2;
    seeking_data[20] = (BF::STCIEN >> 8) as u8;
    seeking_data[21] = BF::STCIEN as u8 | 1 << 2;
    let mut seeking_found_data = [0; 32];
    seeking_found_data[0] = (BF::STC >> 8) as u8;
    seeking_found_data[1] = BF::STC as u8;
    seeking_found_data[18] = (BF::TUNE >> 8) as u8;
    seeking_found_data[19] = BF::TUNE as u8 | 2;
    let transactions = [
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            vec![
                0,
                0,
                (BF::TUNE >> 8) as u8,
                BF::TUNE as u8 | 2,
                (BF::STCIEN >> 8) as u8,
                BF::STCIEN as u8 | 1 << 2,
            ],
        ),
        I2cTrans::read(DEV_ADDR, seeking_data.to_vec()),
        // this time STC bit is (incorrectly) not set
        I2cTrans::read(DEV_ADDR, seeking_found_data.to_vec()),
        I2cTrans::write(DEV_ADDR, vec![0, 0, 0, 2]),
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
    block!(dev.tune_with_stc_int_pin(TuneChannel::Raw(2), &pin)).unwrap();
    destroy(dev);
    pin.done()
}
