#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use esp_hal::gpio::{Input, Level, Output, Pull};
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
    let wheel_button = Input::new(peripherals.GPIO6, Pull::Up);
    let mut trigger_button = Input::new(peripherals.GPIO7, Pull::Up);

    // initialize stepper motor control GPIO
    let mut _dir = Output::new(peripherals.GPIO20, Level::High);
    let mut step = Output::new(peripherals.GPIO21, Level::Low);

    // event loop
    loop {
        info!("Waiting for trigger button press");
        trigger_button.wait_for_falling_edge().await;

        // if not in home position, move to home position
        if wheel_button.is_high() {
            info!("Moving motor to home position");
            let mut ticker =
                Ticker::every(Duration::from_micros(motor_config::DELAY_TIME_US.into()));
            while wheel_button.is_high() {
                step_motor(1, &mut ticker, &mut step).await;
            }
        }

        info!("Triggered - starting wheel rotation");
        let mut ticker = Ticker::every(Duration::from_micros(motor_config::DELAY_TIME_US.into()));
        step_motor(motor_config::NUM_STEPS, &mut ticker, &mut step).await;
    }
}

// advance motor one step
async fn step_motor(steps: u32, ticker: &mut Ticker, step_pin: &mut Output<'_>) {
    for _ in 0..steps as usize {
        ticker.next().await;
        step_pin.set_high();
        ticker.next().await;
        step_pin.set_low();
    }
}
