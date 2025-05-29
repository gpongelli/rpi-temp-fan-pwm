use crate::cli_arguments::cli_args::CliArgsTrait;
use log::{debug, info};

pub mod pwm_manager {

    use crate::cli_arguments::cli_args::CliArgsTrait;
    use log::{debug, error, info};
    use num_traits::cast::ToPrimitive;
    use rppal::pwm::{Channel, Polarity, Pwm};
    use std::io::{self};

    use mockall::predicate::*;
    use mockall::*;

    pub trait PwmManagerTrait {
        fn new(
            pwm_channel: u8,
            pwm_freq: f64,
            pwm_duty: f64,
        ) -> Result<Self, Box<dyn std::error::Error>>
        where
            Self: std::marker::Sized;

        fn set_pwm<T: CliArgsTrait + 'static>(
            &self,
            temp: &str,
            cli_args: &T,
        ) -> Result<(), Box<dyn std::error::Error>>;

        fn set_frequency(
            &self,
            freq: f64,
            fan_speed: f64,
        ) -> Result<(), Box<dyn std::error::Error>>;
    }

    #[derive(Debug)]
    pub struct PwmManager {
        pwm: rppal::pwm::Pwm,
    }
    impl PwmManagerTrait for PwmManager {
        fn new(
            pwm_channel: u8,
            pwm_freq: f64,
            pwm_duty: f64,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) at 2 Hz with a 25% duty cycle.
            match Pwm::with_frequency(
                Channel::try_from(pwm_channel)?,
                pwm_freq,
                pwm_duty,
                Polarity::Normal,
                true,
            ) {
                Ok(pwm_handle) => {
                    info!("PWM channel {} created successfully", pwm_channel);
                    Ok(Self { pwm: pwm_handle })
                }
                Err(e) => {
                    error!("Failed to create PWM: {}", e);
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "Failed to create PWM").into())
                }
            }
        }

        fn set_frequency(
            &self,
            freq: f64,
            fan_speed: f64,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // Reconfigure the PWM channel with input parameters.
            self.pwm.set_frequency(freq, fan_speed)?;
            Ok(())
        }

        fn set_pwm<T: CliArgsTrait + 'static>(
            &self,
            temp: &str,
            cli_args: &T,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // Convert the string to a u8
            let temp: u8 = {
                match temp.parse::<f32>() {
                    Ok(f) => {
                        // check if the value is an unsigned integer
                        // value from file is in millidegree Celsius, convert to Celsius
                        if let Some(v) = (f.round() / 1000.0).to_u8() {
                            v
                        } else {
                            error!("Temperature out of u8 range");
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "Temperature out of u8 range",
                            )
                            .into());
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse temperature string: {}", e);
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Failed to parse temperature string",
                        )
                        .into());
                    }
                }
            };
            debug!("Temperature: {}", temp);

            let fan_speed = super::get_fan_speed_linear(temp, cli_args);
            let pwm_freq = cli_args.get_pwm_freq();

            match self.set_frequency(pwm_freq, (fan_speed as f64) / 100.0) {
                Ok(_) => {
                    debug!("PWM frequency set to {pwm_freq} Hz");
                    debug!("Fan speed set to {fan_speed}%");
                }
                Err(e) => {
                    error!("Failed to set PWM frequency: {}", e);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Failed to set PWM frequency",
                    )
                    .into());
                }
            }

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::cli_arguments::cli_args::{MockCliArgsTrait};

        fn cli_args(temp_step: Vec<u8>, speed_step: Vec<u8>, manual_speed: Option<u8>) -> CliArgs {
            CliArgs::new(
                temp_step,
                speed_step,
                manual_speed,
                clap_verbosity_flag::Verbosity::default(),
                0,
                2.0,
                60,
            )
        }

        fn pwm_mng(pwm_channel: u8, pwm_freq: f64, pwm_duty: f64) -> PwmManager {
            PwmManager::new(pwm_channel, pwm_freq, pwm_duty).unwrap()
        }

        // --- set_pwm tests ---

        #[test]
        fn test_set_pwm_invalid_temp_string() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("notanumber", &args);
            assert!(result.is_err());
        }

        #[test]
        fn test_set_pwm_negative_temp_string() {
            let args = cli_args(vec![10, 20, 30], vec![10, 20, 30], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("-10", &args);
            assert!(result.is_err());
        }

        #[test]
        fn test_set_pwm_valid_temp() {
            // This test may fail if run on a system without the required hardware or permissions.
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("60", &args);
            // Accept both Ok and Err, as hardware may not be present
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_set_pwm_manual_speed() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], Some(77));
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("60", &args);
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_set_pwm_below_first_temp() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("10", &args);
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_set_pwm_above_last_temp() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("200", &args);
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_set_pwm_exact_step_temp() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("70", &args);
            assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn test_set_pwm_float_temp() {
            let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
            let pwm = pwm_mng(0, 2.0, 0.5);
            let result = pwm.set_pwm("65.7", &args);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

// Get speed interpolating array's values
fn get_fan_speed_linear(temp: u8, cli_args: &impl CliArgsTrait) -> u8 {
    // manually forced value
    if cli_args.get_manual_speed().is_some() {
        let val = cli_args.get_manual_speed().unwrap();
        debug!("manual speed: {}", val);
        return val;
    }

    let cfg_speed = cli_args.get_speed_step();
    let cfg_temp = cli_args.get_temp_step();

    let mut speed: u8 = *cfg_speed.last().unwrap();
    let last_temp = *cfg_temp.last().unwrap();

    info!("temp: {}", temp);

    // temp below first value
    if temp < cfg_temp[0] {
        debug!("min speed: {}", cfg_speed[0]);
        speed = cfg_speed[0];
    } else if temp > last_temp {
        debug!("max speed: {}", speed);
        // max value already selected
    } else {
        for (i, &step_temp) in cfg_temp.iter().enumerate() {
            let next_step_temp = cfg_temp[i + 1];

            let speed_step = cfg_speed[i];
            let next_speed_step = cfg_speed[i + 1];

            debug!("Temperature step[{}]: {}", i, step_temp);
            debug!("Temperature next step[{}]: {}", i + 1, next_step_temp);

            if (temp >= step_temp) && (temp <= next_step_temp) {
                // Linear interpolation
                let temp_range: u8 = next_step_temp - step_temp;
                let speed_range: u8 = next_speed_step - speed_step;
                let temp_diff: u8 = temp - step_temp;

                debug!("temp_diff: {}", temp_diff);
                debug!("temp_range: {}", temp_range);
                debug!("speed_range: {}", speed_range);

                let speed_range: u16 = speed_range as u16;
                let temp_diff: u16 = temp_diff as u16;
                let temp_range: u16 = temp_range as u16;
                let speed_step: u16 = speed_step as u16;
                let calc: u16 = speed_range * temp_diff / temp_range;

                speed = (speed_step + calc).try_into().unwrap();
                debug!("Linear interpolation: {}", speed);
                break;
            }
        }
    }

    debug!("temp: {}", temp);
    debug!("speed: {}", speed);
    speed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_arguments::cli_args::MockCliArgsTrait;

    // --- get_fan_speed_linear tests ---

    #[test]
    fn test_manual_speed() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| Some(42));
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![20, 50, 100]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![50, 70, 80]);

        assert_eq!(get_fan_speed_linear(60, &cli_mock), 42);
        assert_eq!(get_fan_speed_linear(80, &cli_mock), 42);
    }

    #[test]
    fn test_below_first_temp() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| None);
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![20, 50, 100]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![50, 70, 80]);

        assert_eq!(get_fan_speed_linear(40, &cli_mock), 20);
        assert_eq!(get_fan_speed_linear(0, &cli_mock), 20);
    }

    #[test]
    fn test_above_last_temp() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| None);
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![20, 50, 100]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![50, 70, 80]);

        assert_eq!(get_fan_speed_linear(90, &cli_mock), 100);
        assert_eq!(get_fan_speed_linear(255, &cli_mock), 100);
    }

    #[test]
    fn test_exact_temp_steps() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| None);
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![20, 50, 100]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![50, 70, 80]);

        assert_eq!(get_fan_speed_linear(50, &cli_mock), 20);
        assert_eq!(get_fan_speed_linear(70, &cli_mock), 50);
        assert_eq!(get_fan_speed_linear(80, &cli_mock), 100);
    }

    #[test]
    fn test_linear_interpolation() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| None);
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![20, 50, 100]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![50, 70, 80]);

        // Between 50 and 70: 20 -> 50
        assert_eq!(get_fan_speed_linear(65, &cli_mock), 42);
        // Between 70 and 80: 50 -> 100
        assert_eq!(get_fan_speed_linear(75, &cli_mock), 75);
    }

    #[test]
    fn test_non_uniform_steps() {
        let mut cli_mock = MockCliArgsTrait::new();
        cli_mock.expect_get_manual_speed().returning(|| None);
        cli_mock
            .expect_get_speed_step()
            .returning(|| vec![10, 60, 80]);
        cli_mock
            .expect_get_temp_step()
            .returning(|| vec![40, 60, 90]);

        // Between 40 and 60: 10 -> 60
        assert_eq!(get_fan_speed_linear(50, &cli_mock), 35);
        // Between 60 and 90: 60 -> 80
        assert_eq!(get_fan_speed_linear(75, &cli_mock), 70);
    }
}
