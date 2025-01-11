#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::ledc;
use esp_hal::prelude::*;
use {defmt_rtt as _, esp_backtrace as _};

mod motor_config;

#[main]
async fn main(_spawner: Spawner) {
    // initialize mcu
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // initialize embassy
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // initialize button GPIO
    let mut wheel_button = Input::new(peripherals.GPIO6, Pull::Up);
    let mut trigger_button = Input::new(peripherals.GPIO7, Pull::Up);

    // initialize stepper motor control GPIO
    let mut _dir = Output::new(peripherals.GPIO20, Level::High);
    let step = Output::new(peripherals.GPIO21, Level::Low);

    // initialize LEDC (for PWM control)
    info!("Pulse frequency: {}", motor_config::MOTOR_FREQ);
    let mut ledc = ledc::Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(ledc::LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<ledc::LowSpeed>(ledc::timer::Number::Timer0);
    lstimer0
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty8Bit,
            clock_source: ledc::timer::LSClockSource::APBClk,
            frequency: motor_config::MOTOR_FREQ.Hz(),
        })
        .unwrap();
    let mut pwm_channel = ledc.channel(ledc::channel::Number::Channel0, step);
    pwm_channel
        .configure(ledc::channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            pin_config: ledc::channel::config::PinConfig::PushPull,
        })
        .unwrap();

    // event loop
    loop {
        info!("Waiting for trigger button press");
        trigger_button.wait_for_falling_edge().await;

        // if not in home position, move to home position
        if wheel_button.is_high() {
            info!("Moving motor to home position");
            pwm_channel.set_duty(50).unwrap();
            wheel_button.wait_for_low().await;
            pwm_channel.set_duty(0).unwrap();
        }

        info!("Triggered - starting wheel rotation");
        pwm_channel.set_duty(50).unwrap();
        wheel_button.wait_for_falling_edge().await;
        pwm_channel.set_duty(0).unwrap();
    }
}
