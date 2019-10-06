//! This is a platform agnostic Rust driver for the Si470x FM radio
//! turner (receiver) using the [`embedded-hal`] traits.
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
pub use device_impl::reset;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

/// IC markers
#[doc(hidden)]
pub mod ic {
    /// Used for Si4703 devices
    pub struct Si4703(());
}

/// Si470x device driver
#[derive(Debug)]
pub struct Si470x<I2C, IC> {
    i2c: I2C,
    is_seeking: bool,
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
