use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin};

/// Reset the device and select I2C communication (method 1, no GPIO3)
///
/// This should be used when using GPIO3 for the external crystal like in
/// some popular breakout modules. e.g. from Sparkfun.
/// This includes a 2ms delay to allow the pins to settle and the device
/// to perform the reset.
pub fn reset_and_select_i2c_method1<
    E,
    RST: OutputPin<Error = E>,
    SDA: OutputPin<Error = E>,
    DELAY: DelayMs<u8>,
>(
    rst: &mut RST,
    sda: &mut SDA,
    delay: &mut DELAY,
) -> Result<(), E> {
    sda.set_low()?;
    rst.set_low()?;
    reset(rst, delay)
}

/// Reset the device and select I2C communication (method 1 including GPIO3)
///
/// This includes a 2ms delay to allow the pins to settle and the device
/// to perform the reset.
pub fn reset_and_select_i2c_method1_with_gpio3<
    E,
    RST: OutputPin<Error = E>,
    SDA: OutputPin<Error = E>,
    GPIO3: OutputPin<Error = E>,
    DELAY: DelayMs<u8>,
>(
    rst: &mut RST,
    sda: &mut SDA,
    gpio3: &mut GPIO3,
    delay: &mut DELAY,
) -> Result<(), E> {
    sda.set_low()?;
    rst.set_low()?;
    gpio3.set_low()?;
    reset(rst, delay)
}

/// Reset the device and select I2C communication (method 2)
///
/// This includes a 2ms delay to allow the pins to settle and the device
/// to perform the reset.
pub fn reset_and_select_i2c_method2<
    E,
    RST: OutputPin<Error = E>,
    GPIO1: OutputPin<Error = E>,
    GPIO3: OutputPin<Error = E>,
    DELAY: DelayMs<u8>,
>(
    rst: &mut RST,
    gpio1: &mut GPIO1,
    gpio3: &mut GPIO3,
    delay: &mut DELAY,
) -> Result<(), E> {
    rst.set_low()?;
    gpio3.set_high()?;
    gpio1.set_high()?;
    reset(rst, delay)
}

fn reset<E, RST: OutputPin<Error = E>, DELAY: DelayMs<u8>>(
    rst: &mut RST,
    delay: &mut DELAY,
) -> Result<(), E> {
    delay.delay_ms(1);
    rst.set_high()?;
    delay.delay_ms(1);
    Ok(())
}
