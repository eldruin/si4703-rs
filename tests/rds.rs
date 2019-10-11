extern crate embedded_hal_mock as hal;
extern crate si4703;
use hal::i2c::Transaction as I2cTrans;
use si4703::RdsMode;

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

write_test!(en_rds_std, BF::RDS, 16, 3, enable_rds, RdsMode::Standard);

#[test]
fn can_enable_rds_verbose() {
    let transactions = [
        I2cTrans::read(DEV_ADDR, [0; 32].to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            vec![
                (BF::RDSM >> 8) as u8,
                BF::RDSM as u8,
                0,
                0,
                (BF::RDS >> 8) as u8,
                BF::RDS as u8,
            ],
        ),
    ];
    let mut dev = new_si4703(&transactions);
    dev.enable_rds(RdsMode::Verbose).unwrap();
    destroy(dev);
}
