use core::marker::PhantomData;

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
    use super::super::private;
    pub trait WithRds: private::Sealed {}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SeekingState {
    Idle,
    Seeking,
    WaitingForStcToClear(bool),
}

/// Si4703 device driver
#[derive(Debug)]
pub struct Si4703<I2C, IC> {
    pub(crate) i2c: I2C,
    pub(crate) seeking_state: SeekingState,
    pub(crate) _ic: PhantomData<IC>,
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

/// Softmute Attack/Recover Rate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoftmuteRate {
    /// Fastest (default)
    Fastest,
    /// Fast
    Fast,
    /// Slow
    Slow,
    /// Slowest
    Slowest,
}

/// Softmute Attenuation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoftmuteAttenuation {
    /// 16 dB (default)
    Db16,
    /// 14 dB
    Db14,
    /// 12 dB
    Db12,
    /// 10 dB
    Db10,
}

impl Default for StereoToMonoBlendLevel {
    fn default() -> Self {
        StereoToMonoBlendLevel::Dbuv31_49
    }
}
