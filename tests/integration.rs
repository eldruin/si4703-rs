extern crate embedded_hal_mock as hal;
extern crate si470x;
use hal::i2c::Transaction as I2cTrans;

mod common;
use self::common::{destroy, new_si4703, DEV_ADDR};

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
