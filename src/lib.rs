//! This is a platform agnostic Rust driver for the Si4702 and Si4703 FM radio
//! turners (receivers) using the [`embedded-hal`] traits and I2C.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Enable/disable the device. See: [`enable()`].
//! - Mute/unmute. See: [`mute()`].
//! - Configure seek. See: [`configure_seek()`].
//! - Seek with/without STC interrupts. See: [`seek_with_stc_int_pin()`].
//! - Tune a frequency with/without STC interrupts. See: [`tune_with_stc_int_pin()`].
//! - Set volume. See: [`set_volume()`].
//! - Set band. See: [`set_band()`].
//! - Set channel spacing. See: [`set_channel_spacing()`].
//! - Set the GPIO1, GPIO2 and GPIO3 function/status. See: [`set_gpio1()`].
//! - Enable/disable softmute. See: [`enable_softmute()`].
//! - Enable/disable auto gain control. See: [`enable_auto_gain_control()`].
//! - Enable/disable oscillator. See: [`enable_oscillator()`].
//! - Enable/disable STC interrupts. See: [`enable_stc_interrupts()`].
//! - Enable/disable RDS. See: [`enable_rds()`].
//! - Enable/disable audio High-Z. See: [`enable_audio_high_z()`].
//! - Set de-emphasis. See: [`set_deemphasis()`].
//! - Set stereo to mono blend level. See: [`set_stereo_to_mono_blend_level()`].
//! - Set stereo/mono output mode. See: [`set_output_mode()`].
//! - Get channel. See: [`get_channel()`].
//! - Get device ID. See: [`get_device_id()`].
//! - Get chip ID. See: [`get_chip_id()`].
//! - Reset and select I2C communication using several methods. See: [`reset_and_select_i2c_method1()`].
//!
//! [`enable()`]: struct.Si4703.html#method.enable
//! [`mute()`]: struct.Si4703.html#method.mute
//! [`configure_seek()`]: struct.Si4703.html#method.configure_seek
//! [`seek_with_stc_int_pin()`]: struct.Si4703.html#method.seek_with_stc_int_pin
//! [`tune_with_stc_int_pin()`]: struct.Si4703.html#method.tune_with_stc_int_pin
//! [`set_volume()`]: struct.Si4703.html#method.set_volume
//! [`set_band()`]: struct.Si4703.html#method.set_band
//! [`set_channel_spacing()`]: struct.Si4703.html#method.set_channel_spacing
//! [`set_gpio1()`]: struct.Si4703.html#method.set_gpio1
//! [`enable_softmute()`]: struct.Si4703.html#method.enable_softmute
//! [`enable_auto_gain_control()`]: struct.Si4703.html#method.enable_auto_gain_control
//! [`enable_oscillator()`]: struct.Si4703.html#method.enable_oscillator
//! [`enable_stc_interrupts()`]: struct.Si4703.html#method.enable_stc_interrupts
//! [`enable_rds()`]: struct.Si4703.html#method.enable_rds
//! [`enable_audio_high_z()`]: struct.Si4703.html#method.enable_audio_high_z
//! [`set_deemphasis()`]: struct.Si4703.html#method.set_deemphasis
//! [`set_stereo_to_mono_blend_level()`]: struct.Si4703.html#method.set_stereo_to_mono_blend_level
//! [`set_output_mode()`]: struct.Si4703.html#method.set_output_mode
//! [`get_channel()`]: struct.Si4703.html#method.get_channel
//! [`get_device_id()`]: struct.Si4703.html#method.get_device_id
//! [`get_chip_id()`]: struct.Si4703.html#method.get_chip_id
//! [`reset_and_select_i2c_method1()`]: struct.Si4703.html#method.reset_and_select_i2c_method1
//!
//! ## The devices
//!
//! The Si4702/03-C19 extends Silicon Laboratories Si4700/Si4701 FM tuner
//! family, and further increases the ease and attractiveness of adding FM
//! radio reception to mobile devices through small size and board area,
//! minimum component count, flexible programmability, and superior, proven
//! performance.
//!
//! The device offers significant programmability, and caters to the
//! subjective nature of FM listeners and variable FM broadcast environments
//! world-wide through a simplified programming interface and
//! mature functionality.
//!
//! The Si4703-C incorporates a digital processor for the European Radio Data
//! System (RDS) and the US Radio Broadcast Data System (RBDS) including all
//! required symbol decoding, block synchronization, error detection, and
//! error correction functions.
//!
//! RDS enables data such as station identification and song name to be
//! displayed to the user. The Si4703-C offers a detailed RDS view and a
//! standard view, allowing adopters to selectively choose granularity of RDS
//! status, data, and block errors.
//!
//! Datasheets:
//! - [Si4702/Si4703](https://www.silabs.com/documents/public/data-sheets/Si4702-03-C19.pdf)
//!
//! Further documentation:
//! - [Si4700/01/02/03 Programmer's Guide](https://www.silabs.com/documents/public/application-notes/AN230.pdf)
//! - [Using RDS/RBDS with the Si4701/03](https://www.silabs.com/documents/public/application-notes/AN243.pdf)
//! - [Si47xx Programming Guide](https://www.silabs.com/documents/public/application-notes/AN332.pdf)
//!

#![deny(unsafe_code, missing_docs)]
#![no_std]

extern crate embedded_hal as hal;
extern crate libm;
extern crate nb;

mod device_impl;
mod rds;
mod register_access;
mod reset;
mod seek;
use register_access::{BitFlags, Register};
pub use reset::{
    reset_and_select_i2c_method1, reset_and_select_i2c_method1_with_gpio3,
    reset_and_select_i2c_method2,
};
mod tune;
mod types;
use types::OperationState;
pub use types::{
    ic, marker, Band, ChannelSpacing, DeEmphasis, Error, ErrorWithPin, Gpio1Config, Gpio2Config,
    Gpio3Config, OutputMode, RdsMode, SeekDirection, SeekFmImpulseThreshold, SeekMode,
    SeekSnrThreshold, Si4703, SoftmuteAttenuation, SoftmuteRate, StereoToMonoBlendLevel,
    TuneChannel, Volume,
};

impl marker::WithRds for ic::Si4703 {}

mod private {
    use super::ic;
    pub trait Sealed {}

    impl Sealed for ic::Si4702 {}
    impl Sealed for ic::Si4703 {}
}
