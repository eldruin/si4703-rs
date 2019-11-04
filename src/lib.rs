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
//! ## Usage (see also examples folder)
//!
//! To use this driver, import this crate and an `embedded_hal` implementation,
//! then instantiate the appropriate device.
//! In the following examples an instance of the device Si4703 will be created
//! as an example. An instance of the Si4702 device can be created with:
//! `Si4703::new_si4702(...)`.
//!
//! Please find additional examples using hardware in this repository: [driver-examples].
//!
//! [driver-examples]: https://github.com/eldruin/driver-examples
//!
//! ### Seek a channel, listen for 5 seconds, then seek again
//!
//! ```no_run
//! use embedded_hal::blocking::delay::DelayMs;
//! use linux_embedded_hal::{Delay, I2cdev, Pin};
//! use si4703::{
//!     reset_and_select_i2c_method1, ChannelSpacing, DeEmphasis, ErrorWithPin, SeekDirection,
//!     SeekMode, Si4703, Volume,
//! };
//!
//! #fn main() {
//! let mut delay = Delay {};
//! {
//!     // Reset and communication protocol selection must be done beforehand
//!     let mut sda = Pin::new(2);
//!     let mut rst = Pin::new(17);
//!     reset_and_select_i2c_method1(&mut rst, &mut sda, &mut delay).unwrap();
//! }
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let mut radio = Si4703::new(dev);
//! radio.enable_oscillator().unwrap();
//! // Wait for the oscillator to stabilize
//! delay.delay_ms(500_u16);
//! radio.enable().unwrap();
//! // Wait for powerup
//! delay.delay_ms(110_u16);
//!
//! radio.set_volume(Volume::Dbfsm28).unwrap();
//! radio.set_deemphasis(DeEmphasis::Us50).unwrap();
//! radio.set_channel_spacing(ChannelSpacing::Khz100).unwrap();
//! radio.unmute().unwrap();
//!
//! // use STC interrupt pin method
//! let stc_int = Pin::new(27);
//! loop {
//!     match radio.seek_with_stc_int_pin(SeekMode::Wrap, SeekDirection::Up, &stc_int) {
//!         Err(nb::Error::WouldBlock) => {
//!             let channel = radio.get_channel().unwrap_or(-1.0);
//!             println!("Trying channel at {:1} MHz", channel);
//!         }
//!         Err(nb::Error::Other(ErrorWithPin::SeekFailed)) => {
//!             println!("Seek Failed");
//!         }
//!         Err(e) => {
//!             println!("Error: {:?}", e);
//!         }
//!         Ok(_) => {
//!             let channel = radio.get_channel().unwrap_or(-1.0);
//!             println!("Found channel at {:1} MHz", channel);
//!             delay.delay_ms(5000_u16); // listen for 5 seconds, then seek again
//!         }
//!     }
//!     delay.delay_ms(50_u8);
//! }
//! #}
//! ```
//!

#![deny(unsafe_code, missing_docs)]
#![no_std]

mod device_impl;
mod rds;
mod register_access;
mod reset;
mod seek;
use crate::register_access::{BitFlags, Register};
pub use crate::reset::{
    reset_and_select_i2c_method1, reset_and_select_i2c_method1_with_gpio3,
    reset_and_select_i2c_method2,
};
mod tune;
mod types;
use crate::types::OperationState;
pub use crate::types::{
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
