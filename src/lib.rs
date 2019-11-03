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

extern crate embedded_hal as hal;
extern crate nb;

mod device_impl;
mod rds;
mod register_access;
mod seek;
pub use device_impl::reset;
use register_access::{BitFlags, Register};
mod types;
use types::OperationState;
pub use types::{
    ic, marker, Band, ChannelSpacing, DeEmphasis, Error, ErrorWithPin, Gpio1Config, Gpio2Config,
    Gpio3Config, OutputMode, RdsMode, SeekDirection, SeekFmImpulseThreshold, SeekMode,
    SeekSnrThreshold, Si4703, SoftmuteAttenuation, SoftmuteRate, StereoToMonoBlendLevel, Volume,
};

impl marker::WithRds for ic::Si4703 {}

mod private {
    use super::ic;
    pub trait Sealed {}

    impl Sealed for ic::Si4702 {}
    impl Sealed for ic::Si4703 {}
}
