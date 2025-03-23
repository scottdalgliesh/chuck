#![no_std]
#![no_main]

use chuck_core::{Motor, MotorConfig, StepMode};
use defmt::info;
use embassy_executor::Spawner;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::timer::timg::TimerGroup;
use {defmt_rtt as _, esp_backtrace as _};

const GEAR_RATIO: u32 = 16;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // initialize mcu
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    // initialize embassy
    esp_hal_embassy::init(TimerGroup::new(peripherals.TIMG0).timer0);

    // initialize button GPIO
    let input_config = InputConfig::default().with_pull(Pull::Up);
    let wheel_button = Input::new(peripherals.GPIO6, input_config);
    let mut trigger_button = Input::new(peripherals.GPIO7, input_config);

    // initialize stepper motor control GPIO
    let output_config = OutputConfig::default();
    let mut dir = Output::new(peripherals.GPIO20, Level::High, output_config);
    let mut step = Output::new(peripherals.GPIO21, Level::Low, output_config);

    // initialize stepper motor driver
    let motor_config = MotorConfig::new(200, StepMode::Full, 180, 640, 0.01, 2);
    let mut motor = Motor {
        step_pin: &mut step,
        dir_pin: &mut dir,
        config: &motor_config,
    };

    // event loop
    loop {
        info!("Waiting for trigger button press");
        info!("Delay time (us): {}", motor.config.min_step_period());
        trigger_button.wait_for_falling_edge().await;

        if wheel_button.is_high() {
            info!("Moving motor to home position");
            while wheel_button.is_high() {
                motor.step_once(motor.config.max_step_period()).await;
            }
        } else {
            info!("Triggered - starting wheel rotation");
            motor
                .move_to_position(motor.config.steps_per_rev() * GEAR_RATIO)
                .await;
        }
    }
}
