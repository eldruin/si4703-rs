use embedded_hal_mock::{
    delay::MockNoop as NoopDelay,
    pin::{Mock as PinMock, State as PinState, Transaction as PinTrans},
};
use si4703::{
    reset_and_select_i2c_method1, reset_and_select_i2c_method1_with_gpio3,
    reset_and_select_i2c_method2,
};

#[test]
fn can_reset_method1() {
    let mut rst = PinMock::new(&[PinTrans::set(PinState::Low), PinTrans::set(PinState::High)]);
    let mut sda = PinMock::new(&[PinTrans::set(PinState::Low)]);
    let mut delay = NoopDelay::new();
    reset_and_select_i2c_method1(&mut rst, &mut sda, &mut delay).unwrap();
    rst.done();
    sda.done()
}

#[test]
fn can_reset_method1_with_gpio3() {
    let mut rst = PinMock::new(&[PinTrans::set(PinState::Low), PinTrans::set(PinState::High)]);
    let mut sda = PinMock::new(&[PinTrans::set(PinState::Low)]);
    let mut gpio3 = PinMock::new(&[PinTrans::set(PinState::Low)]);
    let mut delay = NoopDelay::new();
    reset_and_select_i2c_method1_with_gpio3(&mut rst, &mut sda, &mut gpio3, &mut delay).unwrap();
    rst.done();
    sda.done();
    gpio3.done()
}

#[test]
fn can_reset_method2() {
    let mut rst = PinMock::new(&[PinTrans::set(PinState::Low), PinTrans::set(PinState::High)]);
    let mut gpio1 = PinMock::new(&[PinTrans::set(PinState::High)]);
    let mut gpio3 = PinMock::new(&[PinTrans::set(PinState::High)]);
    let mut delay = NoopDelay::new();
    reset_and_select_i2c_method2(&mut rst, &mut gpio1, &mut gpio3, &mut delay).unwrap();
    rst.done();
    gpio1.done();
    gpio3.done()
}
