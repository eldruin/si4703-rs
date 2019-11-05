use embedded_hal::blocking::delay::DelayMs;
use linux_embedded_hal::{Delay, I2cdev, Pin};
use nb::block;
use si4703::{
    reset_and_select_i2c_method1, ChannelSpacing, DeEmphasis, SeekDirection, SeekMode, Si4703,
    Volume,
};

fn main() {
    let mut delay = Delay {};
    {
        // Reset and communication protocol selection must be done beforehand
        let mut sda = Pin::new(2);
        let mut rst = Pin::new(17);
        reset_and_select_i2c_method1(&mut rst, &mut sda, &mut delay).unwrap();
    }
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut radio = Si4703::new(dev);
    radio.enable_oscillator().unwrap();
    // Wait for the oscillator to stabilize
    delay.delay_ms(500_u16);
    radio.enable().unwrap();
    // Wait for powerup
    delay.delay_ms(110_u16);

    radio.set_volume(Volume::Dbfsm28).unwrap();
    radio.set_deemphasis(DeEmphasis::Us50).unwrap();
    radio.set_channel_spacing(ChannelSpacing::Khz100).unwrap();
    radio.unmute().unwrap();

    let stc_int = Pin::new(27);
    // Seek using STC interrupt pin
    block!(radio.seek_with_stc_int_pin(SeekMode::Wrap, SeekDirection::Up, &stc_int)).unwrap();
    let channel = radio.channel().unwrap_or(-1.0);
    println!("Found channel at {:1} MHz", channel);
}
