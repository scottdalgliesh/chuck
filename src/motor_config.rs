//! Motor configuration constants

// Inputs
pub const MOTOR_STEPS_PER_REV: u32 = 200;
pub const MICRO_STEP_MODE_DIVISOR: u32 = 2;
pub const RPM: u32 = 320;
pub const GEAR_RATIO: u32 = 16;

// Calculated values
pub const WHEEL_PERIOD_US: u32 = 60 * 1_000_000 * GEAR_RATIO / RPM;
pub const DELAY_TIME_US: u32 =
    60 * 1_000_000 / (2 * RPM * MOTOR_STEPS_PER_REV * MICRO_STEP_MODE_DIVISOR);
pub const MOTOR_FREQ: u32 = 1_000_000 / (2 * DELAY_TIME_US);
