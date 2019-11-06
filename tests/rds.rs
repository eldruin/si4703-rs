use embedded_hal_mock::i2c::Transaction as I2cTrans;
use si4703::{RdsBlockData, RdsBlockErrors, RdsData, RdsMode};

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

write_test!(en_rds_int, BF::RDSIEN, 16, 3, enable_rds_interrupts);
write_test!(dis_rds_int, 0, 16, 3, disable_rds_interrupts);

read_test!(rds_not_sync, 0, 1, false, rds_synchronized);
read_test!(rds_sync, BF::RDSS, 1, true, rds_synchronized);

read_test!(rds_not_ready, 0, 1, false, rds_ready);
read_test!(rds_ready, BF::RDSR, 1, true, rds_ready);

#[test]
fn get_rds_data() {
    let rds_data = RdsData {
        a: RdsBlockData {
            data: 0x1234,
            errors: RdsBlockErrors::OneOrTwo,
        },
        b: RdsBlockData {
            data: 0x5678,
            errors: RdsBlockErrors::ThreeToFive,
        },
        c: RdsBlockData {
            data: 0x9ABC,
            errors: RdsBlockErrors::OneOrTwo,
        },
        d: RdsBlockData {
            data: 0xDEF0,
            errors: RdsBlockErrors::TooMany,
        },
    };
    let bler_bcd = BF::BLERB1 | BF::BLERC0 | BF::BLERD0 | BF::BLERD1;
    let mut data = [0; 12];
    data[0] = (BF::BLERA0 >> 8) as u8;
    data[1] = BF::BLERA0 as u8;
    data[2] = (bler_bcd >> 8) as u8;
    data[3] = bler_bcd as u8;
    data[4] = (rds_data.a.data >> 8) as u8;
    data[5] = rds_data.a.data as u8;
    data[6] = (rds_data.b.data >> 8) as u8;
    data[7] = rds_data.b.data as u8;
    data[8] = (rds_data.c.data >> 8) as u8;
    data[9] = rds_data.c.data as u8;
    data[10] = (rds_data.d.data >> 8) as u8;
    data[11] = rds_data.d.data as u8;
    let transactions = [I2cTrans::read(DEV_ADDR, data.to_vec())];
    let mut dev = new_si4703(&transactions);
    assert_eq!(rds_data, dev.rds_data().unwrap());
    destroy(dev);
}
