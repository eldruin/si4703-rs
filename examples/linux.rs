/// Seek a channel, listen for 5 seconds and then seek again.
use embedded_hal::blocking::delay::DelayMs;
use linux_embedded_hal::{Delay, I2cdev, Pin};
use si4703::{
    reset_and_select_i2c_method1, ChannelSpacing, DeEmphasis, ErrorWithPin, SeekDirection,
    SeekMode, Si4703, Volume,
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

    // use STC interrupt pin method
    let stc_int = Pin::new(27);
    loop {
        match radio.seek_with_stc_int_pin(SeekMode::Wrap, SeekDirection::Up, &stc_int) {
            Err(nb::Error::WouldBlock) => {
                let channel = radio.channel().unwrap_or(-1.0);
                println!("Trying channel at {:1} MHz", channel);
            }
            Err(nb::Error::Other(ErrorWithPin::SeekFailed)) => {
                println!("Seek Failed");
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
            Ok(_) => {
                let channel = radio.channel().unwrap_or(-1.0);
                println!("Found channel at {:1} MHz", channel);
                delay.delay_ms(5000_u16); // listen for 5 seconds, then seek again
            }
        }
        delay.delay_ms(50_u8);
    }
}
