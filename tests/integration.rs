extern crate embedded_hal_mock as hal;
extern crate si4703;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use si4703::{
    Band, ChannelSpacing as Spacing, DeEmphasis, Gpio1Config, Gpio2Config, Gpio3Config, OutputMode,
    Si4703, StereoToMonoBlendLevel as Blend, Volume,
};

mod common;
use self::common::{destroy, new_si4703, BitFlags as BF, DEV_ADDR};

const DATA: [u8; 32] = [
    0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13,
];

#[test]
fn can_create_and_destroy_si4702() {
    let dev = Si4703::new_si4702(I2cMock::new(&[]));
    destroy(dev);
}

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

write_powercfg_test!(can_unmute, BF::DMUTE, unmute);
write_powercfg_test!(can_mute, 0x0, mute);

write_powercfg_test!(can_dis_smute, BF::DSMUTE, disable_softmute);
write_powercfg_test!(can_en_smute, 0x0, enable_softmute);

write_powercfg_test!(set_stereo, 0, set_output_mode, OutputMode::Stereo);
write_powercfg_test!(set_mono, BF::MONO, set_output_mode, OutputMode::Mono);

#[macro_export]
macro_rules! write_syscfg1_test {
    ($name:ident, $value:expr, $method:ident $(, $arg:expr)*) => {
        write_test!($name, $value, 16, 3, $method $(, $arg)*);
    };
}

write_syscfg1_test!(set_de_75, 0, set_deemphasis, DeEmphasis::Us75);
write_syscfg1_test!(set_de_50, BF::DE, set_deemphasis, DeEmphasis::Us50);

write_syscfg1_test!(enable_agc, 0, enable_auto_gain_control);
write_syscfg1_test!(disable_agc, BF::AGCD, disable_auto_gain_control);

write_syscfg1_test!(
    set_blend_default,
    0,
    set_stereo_to_mono_blend_level,
    Blend::Dbuv31_49
);

write_syscfg1_test!(
    set_blend_minus12db,
    2 << 6,
    set_stereo_to_mono_blend_level,
    Blend::Dbuv19_37
);

write_syscfg1_test!(
    set_blend_minus6db,
    3 << 6,
    set_stereo_to_mono_blend_level,
    Blend::Dbuv25_43
);

write_syscfg1_test!(
    set_blend_plus6b,
    1 << 6,
    set_stereo_to_mono_blend_level,
    Blend::Dbuv37_55
);

#[macro_export]
macro_rules! set_vol_test {
    ($name:ident, $sysconfig2:expr, $sysconfig3:expr, $volume:ident) => {
        write_test!(
            $name,
            16,
            3,
            $sysconfig2,
            4,
            $sysconfig3,
            set_volume,
            Volume::$volume
        );
    };
}

set_vol_test!(set_vol_m58, 1, BF::VOLEXT, Dbfsm58);
set_vol_test!(set_vol_m56, 2, BF::VOLEXT, Dbfsm56);
set_vol_test!(set_vol_m54, 3, BF::VOLEXT, Dbfsm54);
set_vol_test!(set_vol_m52, 4, BF::VOLEXT, Dbfsm52);
set_vol_test!(set_vol_m50, 5, BF::VOLEXT, Dbfsm50);
set_vol_test!(set_vol_m48, 6, BF::VOLEXT, Dbfsm48);
set_vol_test!(set_vol_m46, 7, BF::VOLEXT, Dbfsm46);
set_vol_test!(set_vol_m44, 8, BF::VOLEXT, Dbfsm44);
set_vol_test!(set_vol_m42, 9, BF::VOLEXT, Dbfsm42);
set_vol_test!(set_vol_m40, 10, BF::VOLEXT, Dbfsm40);
set_vol_test!(set_vol_m38, 11, BF::VOLEXT, Dbfsm38);
set_vol_test!(set_vol_m36, 12, BF::VOLEXT, Dbfsm36);
set_vol_test!(set_vol_m34, 13, BF::VOLEXT, Dbfsm34);
set_vol_test!(set_vol_m32, 14, BF::VOLEXT, Dbfsm32);
set_vol_test!(set_vol_m30, 15, BF::VOLEXT, Dbfsm30);
set_vol_test!(set_vol_m28, 1, 0, Dbfsm28);
set_vol_test!(set_vol_m26, 2, 0, Dbfsm26);
set_vol_test!(set_vol_m24, 3, 0, Dbfsm24);
set_vol_test!(set_vol_m22, 4, 0, Dbfsm22);
set_vol_test!(set_vol_m20, 5, 0, Dbfsm20);
set_vol_test!(set_vol_m18, 6, 0, Dbfsm18);
set_vol_test!(set_vol_m16, 7, 0, Dbfsm16);
set_vol_test!(set_vol_m14, 8, 0, Dbfsm14);
set_vol_test!(set_vol_m12, 9, 0, Dbfsm12);
set_vol_test!(set_vol_m10, 10, 0, Dbfsm10);
set_vol_test!(set_vol_m8, 11, 0, Dbfsm8);
set_vol_test!(set_vol_m6, 12, 0, Dbfsm6);
set_vol_test!(set_vol_m4, 13, 0, Dbfsm4);
set_vol_test!(set_vol_m2, 14, 0, Dbfsm2);
set_vol_test!(set_vol_0, 15, 0, Dbfs0);
write_test!(can_set_mute_vol, 0, 16, 4, set_volume, Volume::Mute);

write_test!(band_87_5_108, 0, 16, 4, set_band, Band::Mhz875_108);
write_test!(band_76_108, 1 << 6, 16, 4, set_band, Band::Mhz76_108);
write_test!(band_76_90, 2 << 6, 16, 4, set_band, Band::Mhz76_90);

write_test!(spc_200, 0, 16, 4, set_channel_spacing, Spacing::Khz200);
write_test!(spc_100, 1 << 4, 16, 4, set_channel_spacing, Spacing::Khz100);
write_test!(spc_50, 2 << 4, 16, 4, set_channel_spacing, Spacing::Khz50);

write_test!(gpio1_hi, 0, 16, 3, set_gpio1, Gpio1Config::HighImpedance);
write_test!(gpio1_high, 3, 16, 3, set_gpio1, Gpio1Config::High);
write_test!(gpio1_low, 2, 16, 3, set_gpio1, Gpio1Config::Low);

const STC_RDS_INT: Gpio2Config = Gpio2Config::StcRdsInterrupt;
write_test!(gpio2_hi, 0, 16, 3, set_gpio2, Gpio2Config::HighImpedance);
write_test!(gpio2_int, 1 << 2, 16, 3, set_gpio2, STC_RDS_INT);
write_test!(gpio2_high, 3 << 2, 16, 3, set_gpio2, Gpio2Config::High);
write_test!(gpio2_low, 2 << 2, 16, 3, set_gpio2, Gpio2Config::Low);

const STEREO_IND: Gpio3Config = Gpio3Config::MonoStereoIndicator;
write_test!(gpio3_hi, 0, 16, 3, set_gpio3, Gpio3Config::HighImpedance);
write_test!(gpio3_stereo, 1 << 4, 16, 3, set_gpio3, STEREO_IND);
write_test!(gpio3_high, 3 << 4, 16, 3, set_gpio3, Gpio3Config::High);
write_test!(gpio3_low, 2 << 4, 16, 3, set_gpio3, Gpio3Config::Low);

write_test!(dis_stci, 0, 16, 3, disable_stc_interrupts);
write_test!(en_stci, BF::STCIEN, 16, 3, enable_stc_interrupts);

macro_rules! get_channel_test {
    ($name:ident, $sysconfig2:expr, $readchan:expr, $value:expr) => {
        #[test]
        fn $name() {
            let mut data = [0; 32];
            data[2] = ($readchan >> 8) as u8;
            data[3] = $readchan as u8;
            data[11 * 2] = ($sysconfig2 >> 8) as u8;
            data[11 * 2 + 1] = $sysconfig2 as u8;
            let transactions = [I2cTrans::read(DEV_ADDR, data.to_vec())];
            let mut dev = new_si4703(&transactions);
            let channel = dev.get_channel().unwrap();
            assert!(($value - 0.2) < channel);
            assert!(($value + 0.2) > channel);
            destroy(dev);
        }
    };
}

get_channel_test!(get_channel_87_base, 0, 0, 87.5);
get_channel_test!(get_channel_87_base_readchan, 0, 100_u16, 87.5 + 100.0 * 0.2);
get_channel_test!(get_channel_76_base, 1 << 6, 100_u16, 76.0 + 100.0 * 0.2);
get_channel_test!(get_channel_0_1_sp, 1 << 4, 100_u16, 87.5 + 100.0 * 0.1);
get_channel_test!(get_channel_0_05_sp, 2 << 4, 100_u16, 87.5 + 100.0 * 0.05);
get_channel_test!(get_chan_comb, 1 << 6 | 2 << 4, 100_u16, 76.0 + 100.0 * 0.05);

#[test]
fn can_read_device_id() {
    let mut data = [0; 32];
    data[6 * 2] = 0x12;
    data[6 * 2 + 1] = 0x42;
    let transactions = [I2cTrans::read(DEV_ADDR, data.to_vec())];
    let mut dev = new_si4703(&transactions);
    let (pn, mfid) = dev.get_device_id().unwrap();
    assert_eq!(pn, 1);
    assert_eq!(mfid, 0x242);
    destroy(dev);
}

#[test]
fn can_read_chip_id() {
    let mut data = [0; 32];
    data[7 * 2] = 0x85;
    data[7 * 2 + 1] = 0xA5;
    let transactions = [I2cTrans::read(DEV_ADDR, data.to_vec())];
    let mut dev = new_si4703(&transactions);
    let (rev, device, firmware) = dev.get_chip_id().unwrap();
    assert_eq!(rev, 33);
    assert_eq!(device, 6);
    assert_eq!(firmware, 37);
    destroy(dev);
}
