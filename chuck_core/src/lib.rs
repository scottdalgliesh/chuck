#![no_std]

use embassy_futures::join::join;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;

/// Micro-step mode for DRV8825 stepper motor.
#[derive(Debug, Clone, Copy)]
pub enum StepMode {
    Full = 1,
    Half = 2,
    Quarter = 4,
    Eighth = 8,
    Sixteenth = 16,
    ThirtySecondth = 32,
}

impl StepMode {
    pub fn value(self) -> u32 {
        self as u32
    }
}

/// Configuration parameters for DRV8825 stepper motor.
#[derive(Debug)]
pub struct MotorConfig {
    full_steps_per_rev: u32,
    step_mode: StepMode,
    min_rpm: u32,
    max_rpm: u32,
    accel: f32,
    min_pulse_on_us: u32,
}

impl MotorConfig {
    pub fn new(
        full_steps_per_rev: u32,
        step_mode: StepMode,
        min_rpm: u32,
        max_rpm: u32,
        accel: f32,
        min_pulse_on_us: u32,
    ) -> Self {
        MotorConfig {
            full_steps_per_rev,
            step_mode,
            min_rpm,
            max_rpm,
            accel,
            min_pulse_on_us,
        }
    }

    /// calculates total steps per motor revolution, based on motor full steps per rev and micro-step mode
    pub fn steps_per_rev(&self) -> u32 {
        self.full_steps_per_rev * self.step_mode.value()
    }

    /// min step period (micros)
    pub fn min_step_period(&self) -> u32 {
        step_period_micros(self.max_rpm, self.full_steps_per_rev, self.step_mode)
    }

    /// max step period (micros)
    pub fn max_step_period(&self) -> u32 {
        step_period_micros(self.min_rpm, self.full_steps_per_rev, self.step_mode)
    }
}

/// calculates step period (micros) based on target motor rpm, full steps per rev, and micro-step mode
fn step_period_micros(rpm: u32, full_steps_per_rev: u32, step_mode: StepMode) -> u32 {
    let freq = step_frequency(rpm, full_steps_per_rev, step_mode);
    1_000_000 / freq
}

/// calculates step frequency (hz) based on target motor rpm, full steps per rev, and micro-step mode
fn step_frequency(rpm: u32, full_steps_per_rev: u32, step_mode: StepMode) -> u32 {
    full_steps_per_rev * step_mode.value() * rpm / 60
}

/// basic stepper motor driver with support for linearly-ramped acceleration
#[derive(Debug)]
pub struct Motor<'a, Step: OutputPin, Dir: OutputPin> {
    pub step_pin: &'a mut Step,
    pub dir_pin: &'a mut Dir,
    pub config: &'a MotorConfig,
}

impl<Step: OutputPin, Dir: OutputPin> Motor<'_, Step, Dir> {
    /// move motor by one step
    pub async fn step_once(&mut self, period_us: u32) {
        join(Timer::after_micros(period_us.into()), async {
            let _ = self.step_pin.set_high();
            Timer::after_micros(self.config.min_pulse_on_us.into()).await;
            let _ = self.step_pin.set_low();
        })
        .await;
    }

    /// move specified number of steps, starting from min_rpm and observing max accel  
    pub async fn move_to_position(&mut self, steps: u32) {
        let init_step_period = self.config.max_step_period();
        let target_step_period = self.config.min_step_period();
        let ramp_increment = (init_step_period as f32 * self.config.accel) as u32;
        let max_ramp_steps = (init_step_period - target_step_period) / ramp_increment;
        let ramp_steps = if 2 * max_ramp_steps >= steps {
            steps / 2
        } else {
            max_ramp_steps
        };
        let constant_steps = steps - 2 * ramp_steps;

        // ramp-up
        let mut current_period = init_step_period;
        for _ in 0..ramp_steps {
            self.step_once(current_period).await;
            current_period -= ramp_increment;
        }

        // full-speed
        for _ in 0..constant_steps {
            self.step_once(target_step_period).await;
        }

        // ramp-down
        let mut current_period = target_step_period;
        for _ in 0..ramp_steps {
            self.step_once(current_period).await;
            current_period += ramp_increment;
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::prelude::rust_2024::*;

    #[test]
    fn step_mode_test() {
        assert_eq!(StepMode::Full.value(), 1_u32);
        assert_eq!(StepMode::Quarter.value(), 4_u32);
    }

    #[test]
    fn motor_config_test() {
        let config = MotorConfig::new(200, StepMode::Half, 120, 360, 0.01, 2);
        assert_eq!(config.steps_per_rev(), 400);
        assert_eq!(config.min_step_period(), 416);
        assert_eq!(config.max_step_period(), 1_250);

        let config = MotorConfig::new(200, StepMode::Eighth, 147, 412, 0.01, 2);
        assert_eq!(config.steps_per_rev(), 1_600);
        assert_eq!(config.min_step_period(), 91);
        assert_eq!(config.max_step_period(), 255);
    }
}
