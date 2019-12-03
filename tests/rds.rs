use embedded_hal_mock::i2c::Transaction as I2cTrans;
use si4703::{
    fill_with_rds_radio_text, get_rds_radio_text, RdsBlockData, RdsBlockErrors, RdsData, RdsMode,
    RdsRadioText, RdsRadioTextData,
};

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

read_test!(rds_not_sync, 0, 2, false, rds_synchronized);
read_test!(rds_sync, BF::RDSS, 2, true, rds_synchronized);

read_test!(rds_not_ready, 0, 2, false, rds_ready);
read_test!(rds_ready, BF::RDSR, 2, true, rds_ready);

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

// It is not possible to call default() for consts
const IRRELEVANT: RdsBlockData = RdsBlockData {
    data: 0,
    errors: RdsBlockErrors::None,
};

const DATA_AB: RdsData = RdsData {
    a: IRRELEVANT,
    b: RdsBlockData {
        data: 0x2800,
        errors: RdsBlockErrors::OneOrTwo,
    },
    c: IRRELEVANT,
    d: RdsBlockData {
        data: 0x4142,
        errors: RdsBlockErrors::OneOrTwo,
    },
};

const DATA_ABCD: RdsData = RdsData {
    a: IRRELEVANT,
    b: RdsBlockData {
        data: 0x2000,
        errors: RdsBlockErrors::OneOrTwo,
    },
    c: RdsBlockData {
        data: 0x4142,
        errors: RdsBlockErrors::ThreeToFive,
    },
    d: RdsBlockData {
        data: 0x4344,
        errors: RdsBlockErrors::OneOrTwo,
    },
};

const NULL_TEXT: RdsRadioTextData = RdsRadioTextData::Four('\0', '\0', '\0', '\0');

mod get_rds_radio_text {
    use super::*;

    #[test]
    fn unsupported_number_of_errors_in_block_b() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::ThreeToFive,
            },
            ..Default::default()
        };
        assert_eq!(None, get_rds_radio_text(&data));
    }

    #[test]
    fn different_type() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x3000,
                errors: RdsBlockErrors::OneOrTwo,
            },
            ..Default::default()
        };
        assert_eq!(None, get_rds_radio_text(&data));
    }

    #[test]
    fn erroneous_data_c_for_four() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2000,
                errors: RdsBlockErrors::OneOrTwo,
            },
            c: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::TooMany,
            },
            ..Default::default()
        };
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: None
            }),
            get_rds_radio_text(&data)
        );
    }

    #[test]
    fn erroneous_data_d_for_four() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2000,
                errors: RdsBlockErrors::OneOrTwo,
            },
            d: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::TooMany,
            },
            ..Default::default()
        };
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: None
            }),
            get_rds_radio_text(&data)
        );
    }

    #[test]
    fn read_four() {
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: Some((RdsRadioTextData::Four('A', 'B', 'C', 'D'), 0)),
            }),
            get_rds_radio_text(&DATA_ABCD)
        );
    }

    #[test]
    fn can_get_screen_clear() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2010,
                errors: RdsBlockErrors::OneOrTwo,
            },
            ..Default::default()
        };
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: true,
                text: Some((NULL_TEXT, 0)),
            }),
            get_rds_radio_text(&data)
        );
    }

    #[test]
    fn can_get_offset() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2009,
                errors: RdsBlockErrors::OneOrTwo,
            },
            ..Default::default()
        };
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: Some((NULL_TEXT, 9 * 4)),
            }),
            get_rds_radio_text(&data)
        );
    }

    #[test]
    fn read_two() {
        let mut data = DATA_AB;
        data.b.data += 9;
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: Some((RdsRadioTextData::Two('A', 'B'), 9 * 2)),
            }),
            get_rds_radio_text(&data)
        );
    }

    #[test]
    fn erroneous_data_d_for_two() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2800,
                errors: RdsBlockErrors::OneOrTwo,
            },
            d: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::TooMany,
            },
            ..Default::default()
        };
        assert_eq!(
            Some(RdsRadioText {
                screen_clear: false,
                text: None
            }),
            get_rds_radio_text(&data)
        );
    }
}

mod fill_radio_text {
    use super::*;
    const EMPTY: [char; 64] = ['_'; 64];

    macro_rules! array_eq {
        ($value:expr, $expected:expr) => {
            assert_eq!($expected[..32], $value[..32]);
            assert_eq!($expected[32..], $value[32..]);
        };
    }

    #[test]
    fn empty_data() {
        let data = RdsData::default();
        let mut text = EMPTY;
        assert_eq!(false, fill_with_rds_radio_text(&mut text, &data));
        array_eq!(EMPTY, text);
    }

    #[test]
    fn erroneous_data() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2000,
                errors: RdsBlockErrors::OneOrTwo,
            },
            d: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::TooMany,
            },
            ..Default::default()
        };
        let mut text = EMPTY;
        assert_eq!(false, fill_with_rds_radio_text(&mut text, &data));
        array_eq!(EMPTY, text);
    }

    #[test]
    fn can_get_screen_clear_on_erroneous_data() {
        let data = RdsData {
            b: RdsBlockData {
                data: 0x2010,
                errors: RdsBlockErrors::OneOrTwo,
            },
            d: RdsBlockData {
                data: 0,
                errors: RdsBlockErrors::TooMany,
            },
            ..Default::default()
        };
        let mut text = EMPTY;
        assert!(fill_with_rds_radio_text(&mut text, &data));
        array_eq!(EMPTY, text);
    }

    #[test]
    fn can_read_two() {
        let mut text = EMPTY;
        assert_eq!(false, fill_with_rds_radio_text(&mut text, &DATA_AB));
        let mut expected = EMPTY;
        expected[0..2].copy_from_slice(&['A', 'B']);
        array_eq!(expected, text);
    }

    #[test]
    fn can_read_four() {
        let mut text = EMPTY;
        assert_eq!(false, fill_with_rds_radio_text(&mut text, &DATA_ABCD));
        let mut expected = EMPTY;
        expected[0..4].copy_from_slice(&['A', 'B', 'C', 'D']);
        array_eq!(expected, text);
    }

    #[test]
    fn can_get_screen_clear() {
        let mut text = EMPTY;
        let mut data = DATA_AB;
        data.b.data += 0x10;
        assert!(fill_with_rds_radio_text(&mut text, &data));
    }
}
