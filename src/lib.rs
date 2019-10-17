//! This is a platform agnostic Rust driver for the Si4702 and Si4703 FM radio
//! turners (receivers) using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! <!--TODO
//! This driver allows you to:
//! -->
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

use core::marker::PhantomData;
extern crate embedded_hal as hal;
extern crate nb;

mod device_impl;
mod rds;
mod register_access;
pub use device_impl::reset;
use register_access::{BitFlags, Register};

/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
    /// Seek operation failed / Band limit reached
    SeekFailed,
}

/// Errors for operations involving I2C communication as well
/// as interaction with pins
#[derive(Debug)]
pub enum ErrorWithPin<CommE, PinE> {
    /// I²C bus communication error
    I2C(CommE),
    /// Error while communicating with pin
    Pin(PinE),
    /// Seek operation failed / Band limit reached
    SeekFailed,
}

/// IC markers
#[doc(hidden)]
pub mod ic {
    /// Used for Si4702 devices
    pub struct Si4702(());
    /// Used for Si4703 devices
    pub struct Si4703(());
}

/// markers
#[doc(hidden)]
pub mod marker {
    use super::private;
    pub trait WithRds: private::Sealed {}
}

impl marker::WithRds for ic::Si4703 {}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SeekingState {
    Idle,
    Seeking,
    WaitingForStcToClear(bool),
}

/// Si4703 device driver
#[derive(Debug)]
pub struct Si4703<I2C, IC> {
    i2c: I2C,
    seeking_state: SeekingState,
    _ic: PhantomData<IC>,
}

/// Seek mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeekMode {
    /// Wrap at the end of the band (default)
    Wrap,
    /// Stop at the end of the band
    NoWrap,
}

/// Seek direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeekDirection {
    /// Down (default)
    Down,
    /// Up
    Up,
}

/// De-emphasis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeEmphasis {
    /// 75 us (used in USA) (default)
    Us75,
    /// 50 us (used in Europe, Australia and Japan)
    Us50,
}

/// GPIO1 configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gpio1Config {
    /// High impedance (default)
    HighImpedance,
    /// High
    High,
    /// Low
    Low,
}

/// GPIO2 configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gpio2Config {
    /// High impedance (default)
    HighImpedance,
    /// STC/RDS interrupt (logic high until interrupt occurs)
    StcRdsInterrupt,
    /// High
    High,
    /// Low
    Low,
}

/// GPIO3 configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gpio3Config {
    /// High impedance (default)
    HighImpedance,
    /// Mono/Stereo indicator (logic low for mono, high for stereo)
    MonoStereoIndicator,
    /// High
    High,
    /// Low
    Low,
}

/// RDS mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RdsMode {
    /// Standard (default)
    Standard,
    /// Verbose
    Verbose,
}

/// Band
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Band {
    /// 87.5-108 Mhz (USA, Europe) (default)
    Mhz875_108,
    /// 76 - 108 MHz (Japan wide band)
    Mhz76_108,
    /// 76 - 90 MHz (Japan)
    Mhz76_90,
}

/// Channel spacing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelSpacing {
    /// 200 kHz (USA, Australia) (default)
    Khz200,
    /// 100 kHz (Europe, Japan)
    Khz100,
    /// 50 kHz
    Khz50,
}

/// Output mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// Stereo (default)
    Stereo,
    /// Mono
    Mono,
}

/// Stereo to mono blend level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StereoToMonoBlendLevel {
    /// 19–37 RSSI dBμV (–12 dB)
    Dbuv19_37,
    /// 25–43 RSSI dBμV (–6 dB).
    Dbuv25_43,
    /// 31–49 RSSI dBμV (default)
    Dbuv31_49,
    /// 37–55 RSSI dBμV (+6 dB)
    Dbuv37_55,
}

/// Volume
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Volume {
    /// Mute (0 volume) (default)
    Mute,
    /// –58 dBFS (extended volume range).
    Dbfsm58,
    /// –56 dBFS (extended volume range).
    Dbfsm56,
    /// –54 dBFS (extended volume range).
    Dbfsm54,
    /// –52 dBFS (extended volume range).
    Dbfsm52,
    /// –50 dBFS (extended volume range).
    Dbfsm50,
    /// –48 dBFS (extended volume range).
    Dbfsm48,
    /// –46 dBFS (extended volume range).
    Dbfsm46,
    /// –44 dBFS (extended volume range).
    Dbfsm44,
    /// –42 dBFS (extended volume range).
    Dbfsm42,
    /// –40 dBFS (extended volume range).
    Dbfsm40,
    /// –38 dBFS (extended volume range).
    Dbfsm38,
    /// –36 dBFS (extended volume range).
    Dbfsm36,
    /// –34 dBFS (extended volume range).
    Dbfsm34,
    /// –32 dBFS (extended volume range).
    Dbfsm32,
    /// –30 dBFS (extended volume range).
    Dbfsm30,
    /// –28 dBFS.
    Dbfsm28,
    /// –26 dBFS.
    Dbfsm26,
    /// –24 dBFS.
    Dbfsm24,
    /// –22 dBFS.
    Dbfsm22,
    /// –20 dBFS.
    Dbfsm20,
    /// –18 dBFS.
    Dbfsm18,
    /// –16 dBFS.
    Dbfsm16,
    /// –14 dBFS.
    Dbfsm14,
    /// –12 dBFS.
    Dbfsm12,
    /// –10 dBFS.
    Dbfsm10,
    /// –8 dBFS.
    Dbfsm8,
    /// –6 dBFS.
    Dbfsm6,
    /// –4 dBFS.
    Dbfsm4,
    /// –2 dBFS.
    Dbfsm2,
    /// 0 dBFS (maximum).
    Dbfs0,
}

impl Default for StereoToMonoBlendLevel {
    fn default() -> Self {
        StereoToMonoBlendLevel::Dbuv31_49
    }
}

mod private {
    use super::ic;
    pub trait Sealed {}

    impl Sealed for ic::Si4702 {}
    impl Sealed for ic::Si4703 {}
}
