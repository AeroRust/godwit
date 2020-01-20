use crate::wind_estimator::WindEstimator;
use crate::extensions::FloatExt;
use crate::calculations::{calculate_eas_from_ias, calculate_tas_from_eas};
use std::time::Duration;
use std::cmp::Ordering;

const TAS_INNOVATION_FAIL_DELAY : Duration = Duration::from_secs(1);

pub enum AirspeedIndex {
    Disabled,
    GroundMinusWind,
    FirstSensor,
    SecondSensor,
    ThirdSensor
}

pub struct AirSpeedModule {

}

impl AirSpeedModule{

}

#[derive(Clone, Debug)]
pub struct UpdaterData {
    timestamp: Duration,
    airspeed_indicated_raw: f32,
    airspeed_true_raw: f32,
    airspeed_timestamp: Duration,
    lpos: Lpos,
    air_pressure: f32,
    air_temperature: f32,
    acceleration: f32,
    velocity_test_ratio: f32,
    magnitude_test_ratio: f32,
    in_fixed_wing_flight: bool

}

#[derive(Clone, Debug)]
pub struct Lpos {
    vx: f32,
    vy: f32,
    vz: f32,
    evh: f32,
    evv: f32,

}

#[derive(Clone, Debug)]
pub struct AirSpeedValidator {
    previous_timestamp: Duration,
    time_last_airspeed: Duration,
    wind_estimator: Option<WindEstimator>,
    eas_scale: f32,
    airspeed_scale_manual: f32,
    ias: f32,
    eas: f32,
    tas: f32,
    innovation_check: InnovationCheck,
    airspeed_stall: f32,
    load_factor_ratio: f32,
    load_factor_check_failed: bool,
    data_stopped_failed: bool,
    airspeed_valid: bool,
    airspeed_failing: bool,
    time_checks_passed: Duration,
    time_checks_failed: Duration,
    checks_fail_delay: Duration,
    checks_clear_delay: Duration,
}

impl AirSpeedValidator {
    pub fn update_validator(&mut self, input: UpdaterData) {

        if input.timestamp != self.previous_timestamp && input.timestamp > Duration::default() {
            self.time_last_airspeed = input.timestamp;
            self.previous_timestamp = input.airspeed_timestamp;
        }
    }

    pub fn update_eas_scale(&mut self) {
        self.eas_scale = match &self.wind_estimator {
            Some(w) => w.tas_scale.constrain(0.5, 2.0),
            None => self.airspeed_scale_manual
        }
    }

    pub fn update_eas_tas(&mut self, air_pressure: f32, air_temperature: f32) {
        self.eas = calculate_eas_from_ias(self.ias, self.eas_scale);
        self.tas = calculate_tas_from_eas(self.eas, air_pressure, air_temperature);
    }

    pub fn check_load_factor(&mut self, acceleration: f32) {
        if self.innovation_check.in_fixed_wing_flight {
            let max_lift_ratio = (self.eas.max(0.7) / self.airspeed_stall.max(1.)).powi(2);
            let lf = self.load_factor_ratio * 0.95 + 0.05 * acceleration.abs() / 9.81 / max_lift_ratio;
            self.load_factor_ratio = lf.constrain(0.25, 2.);
            self.load_factor_check_failed = self.load_factor_ratio > 1.1;
        } else {
            self.load_factor_ratio = 0.5;
        }
    }

    pub fn update_airspeed_validation_status(&mut self, timestamp: Duration) {

        let missing = timestamp - self.time_last_airspeed > Duration::from_millis(200);
        self.data_stopped_failed = timestamp - self.time_last_airspeed > Duration::from_secs(1);

        self.airspeed_failing = self.innovation_check.failed || self.load_factor_check_failed || missing;
        self.time_checks_passed = timestamp;

        if self.airspeed_valid {
            let single_check_failed_timeout = (timestamp - self.time_checks_passed ) > self.checks_fail_delay;
            if self.data_stopped_failed || self.innovation_check.failed && self.load_factor_check_failed || single_check_failed_timeout {
                self.airspeed_valid = false;
            }
        } else if timestamp - self.time_checks_failed > self.checks_clear_delay * Duration::from_secs(1u64).as_secs() as u32 {
            self.airspeed_valid = true;


        }
    }

}

#[derive(Clone, Debug)]
pub struct EstimatorStatusTestRatio {
    velocity: f32,
    magnitude: f32
}

impl PartialEq<f32> for EstimatorStatusTestRatio {
    fn eq(&self, other: &f32) -> bool {
        self.velocity == *other && self.magnitude == *other
    }
}

impl PartialOrd<f32> for EstimatorStatusTestRatio {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.magnitude.partial_cmp(other)
    }

    fn lt(&self, other: &f32) -> bool {
        self.magnitude.lt(other)
    }

    fn le(&self, other: &f32) -> bool {
        self.magnitude.le(other)
    }

    fn gt(&self, other: &f32) -> bool {
        self.magnitude.gt(other)
    }

    fn ge(&self, other: &f32) -> bool {
        self.magnitude.ge(other)
    }
}

#[derive(Clone, Debug)]
pub struct InnovationCheck {
    previous_timestamp: Duration,
    wind_estimator: WindEstimator,
    tas: Tas,
    in_fixed_wing_flight: bool,
    airspeed_innovation_state: f64,
    failed: bool,
    airspeed_valid: bool
}

impl InnovationCheck {
    pub fn check_airspeed_innovation(&mut self, now: Duration, estimator_status_test_ratio: EstimatorStatusTestRatio) {
        if self.in_fixed_wing_flight {
                let dt_s = ((now - self.previous_timestamp).as_secs_f64()/ 1e+6f64).max(0.01);

                if dt_s < 1.0 {
                    let tas_test_ratio = (self.wind_estimator.tas_innovation.powi(2) / self.tas.gate.max(1.).powi(2) * self.wind_estimator.tas_innovation_var) as f64;

                    if tas_test_ratio < self.tas.innovation_threshold as f64 {
                       self.previous_timestamp = now;
                       self.airspeed_innovation_state = 0.;

                    } else {
                        self.airspeed_innovation_state += dt_s * (tas_test_ratio - self.tas.innovation_threshold as f64);
                    }

                    if estimator_status_test_ratio < 1.0 {
                        if self.tas.check_if_innovation_state_passes(self.airspeed_innovation_state as f32) {
                            self.tas.last_timestamp = now;

                        }
                    }

                    self.failed = if  self.failed {
                        (now - self.tas.last_pass) > TAS_INNOVATION_FAIL_DELAY
                    } else {
                        ( now - self.tas.last_fail) < TAS_INNOVATION_FAIL_DELAY * 100
                    };

                }
        } else {
            self.failed = false;
            self.tas.last_pass = now;
            self.tas.last_fail = Duration::default();
            self.airspeed_valid = true;
            self.previous_timestamp = now;
        }
        self.previous_timestamp = now;
    }
    }

#[derive(Debug, Clone)]
pub struct Tas {
    gate: f32,
    innovation_threshold: f32,
    innovation_integration_threshold: f32,
    last_timestamp: Duration,
    last_pass: Duration,
    last_fail: Duration

}

impl Tas {
    pub fn check_if_innovation_state_passes(&self, innovation_state: f32) -> bool {
        self.innovation_integration_threshold > 0. && innovation_state > self.innovation_integration_threshold

    }
}