extern crate embedded_hal_mock as hal;
extern crate si470x;
use hal::i2c::Transaction as I2cTrans;
use si470x::{ChannelSpacing as Spacing, DeEmphasis, Error, Gpio2Config};

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

write_powercfg_test!(can_unmute, 0x4000_u16, unmute);
write_powercfg_test!(can_mute, 0x0, mute);

write_test!(set_de_75, 0, 16, 3, set_deemphasis, DeEmphasis::Us75);
write_test!(set_de_50, BF::DE, 16, 3, set_deemphasis, DeEmphasis::Us50);

write_test!(set_vol_min, 0, 16, 4, set_volume, 0);
write_test!(set_vol_max, 0xF, 16, 4, set_volume, 0xF);
set_invalid_test!(cannot_set_too_high_vol, new_si4703, set_volume, 0x10);

write_test!(spc_200, 0, 16, 4, set_channel_spacing, Spacing::Khz200);
write_test!(spc_100, 1 << 4, 16, 4, set_channel_spacing, Spacing::Khz100);
write_test!(spc_50, 2 << 4, 16, 4, set_channel_spacing, Spacing::Khz50);

const STC_RDS_INT: Gpio2Config = Gpio2Config::StcRdsInterrupt;
write_test!(gpio2_hi, 0, 16, 3, set_gpio2, Gpio2Config::HighImpedance);
write_test!(gpio2_int, 1 << 2, 16, 3, set_gpio2, STC_RDS_INT);
write_test!(gpio2_high, 3 << 2, 16, 3, set_gpio2, Gpio2Config::High);
write_test!(gpio2_low, 2 << 2, 16, 3, set_gpio2, Gpio2Config::Low);

write_test!(dis_stci, 0, 16, 3, disable_stc_interrupts);
write_test!(en_stci, BF::STCIEN, 16, 3, enable_stc_interrupts);
