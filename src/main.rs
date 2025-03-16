#![no_std]
#![no_main]

mod motor_config;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::ledc::{
    self, LowSpeed,
    channel::{self, Channel, ChannelIFace},
    timer::{self, TimerIFace},
};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use {defmt_rtt as _, esp_backtrace as _};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // initialize mcu
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    // initialize embassy
    esp_hal_embassy::init(TimerGroup::new(peripherals.TIMG0).timer0);

    // initialize button GPIO
    let input_config = InputConfig::default().with_pull(Pull::Up);
    let mut wheel_button = Input::new(peripherals.GPIO6, input_config);
    let mut trigger_button = Input::new(peripherals.GPIO7, input_config);

    // initialize stepper motor control GPIO
    let output_config = OutputConfig::default();
    let mut _dir = Output::new(peripherals.GPIO20, Level::High, output_config);
    let step = Output::new(peripherals.GPIO21, Level::Low, output_config);

    // initialize LEDC (for PWM control)
    info!("Pulse frequency: {}", motor_config::MOTOR_FREQ);
    let mut ledc = ledc::Ledc::new(peripherals.LEDC);
    let mut lstimer0 = ledc.timer::<ledc::LowSpeed>(timer::Number::Timer0);
    let mut pwm_channel = ledc.channel(channel::Number::Channel0, step);
    configure_pwm(
        &mut ledc,
        &mut lstimer0,
        &mut pwm_channel,
        motor_config::MOTOR_FREQ,
    );

    // event loop
    loop {
        info!("Waiting for trigger button press");
        trigger_button.wait_for_falling_edge().await;

        if wheel_button.is_high() {
            // if not in home position, move to home position
            info!("Moving motor to home position");
            pwm_channel.set_duty(50).unwrap();
            wheel_button.wait_for_low().await;
            pwm_channel.set_duty(0).unwrap();
        } else {
            // if in home position, rotate wheel one turn to fire
            info!("Triggered - starting wheel rotation");
            pwm_channel.set_duty(50).unwrap();

            // ignore limit switch input until half of wheel rotation is complete
            // to debounce limit switch signal which is very noisy
            Timer::after_micros((motor_config::WHEEL_PERIOD_US / 2).into()).await;
            wheel_button.wait_for_falling_edge().await;
            pwm_channel.set_duty(0).unwrap();
        }
    }
}

/// Configure LEDC channel for PWM output
fn configure_pwm<'a>(
    ledc: &mut ledc::Ledc<'_>,
    lstimer: &'a mut timer::Timer<'a, LowSpeed>,
    pwm_channel: &mut Channel<'a, LowSpeed>,
    freq: u32,
) {
    ledc.set_global_slow_clock(ledc::LSGlobalClkSource::APBClk);
    lstimer
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty8Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(freq),
        })
        .unwrap();
    pwm_channel
        .configure(channel::config::Config {
            timer: lstimer,
            duty_pct: 0,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();
}
