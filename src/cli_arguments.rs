// https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html

pub mod cli_args {
    use clap::Parser;
    use mockall::predicate::*;
    use mockall::*;
    use std::fmt::Debug;
    use std::ops::RangeInclusive;

    const PERCENTAGE: RangeInclusive<usize> = 1..=100;

    #[automock]
    pub trait CliArgsTrait {
        //fn valid(&self) -> bool;

        fn get_temp_step(&self) -> Vec<u8>;

        fn get_speed_step(&self) -> Vec<u8>;

        fn get_manual_speed(&self) -> Option<u8>;

        fn get_verbose(&self) -> clap_verbosity_flag::Verbosity;

        fn get_pwm_channel(&self) -> u8;

        fn get_pwm_freq(&self) -> f64;

        fn get_sleep_secs(&self) -> u64;
    }

    #[derive(Parser, Debug)]
    #[command(version, about, long_about=None)]
    pub struct CliArgs {
        //https://stackoverflow.com/questions/73240901/how-to-get-clap-to-process-a-single-argument-with-multiple-values-without-having
        #[arg(short = 't', long, value_delimiter=',', default_value = "50,70,80", num_args = 1..)]
        temp_step: Vec<u8>,

        #[arg(short = 's', long, value_delimiter=',', default_value = "20,50,100", num_args = 1.., value_parser = percentage_in_range)]
        speed_step: Vec<u8>,

        // Manually set speed step in percentage
        #[arg(short = 'u', long, value_parser = percentage_in_range)]
        manual_speed: Option<u8>,

        #[command(flatten)]
        verbose: clap_verbosity_flag::Verbosity,

        #[arg(short = 'c', long, default_value_t = 0)]
        pwm_channel: u8,

        /// Frequency in Hz
        /// Default: 2.0
        #[arg(short = 'f', long, default_value_t = 2.0)]
        pwm_freq: f64,

        /// Set the sleep period between pwm updates
        #[arg(short = 'e', long, default_value_t = 0)]
        sleep_secs: u64,
    }

    impl CliArgs {
        #[allow(dead_code)]
        pub fn new(
            temp_step: Vec<u8>,
            speed_step: Vec<u8>,
            manual_speed: Option<u8>,
            verbose: clap_verbosity_flag::Verbosity,
            pwm_channel: u8,
            pwm_freq: f64,
            sleep_secs: u64,
        ) -> Self {
            CliArgs {
                temp_step,
                speed_step,
                manual_speed,
                verbose,
                pwm_channel,
                pwm_freq,
                sleep_secs,
            }
        }

        pub fn valid(&self) -> bool {
            self.temp_step.len() == self.speed_step.len()
        }
    }

    impl CliArgsTrait for CliArgs {
        fn get_temp_step(&self) -> Vec<u8> {
            self.temp_step.clone()
        }

        fn get_speed_step(&self) -> Vec<u8> {
            self.speed_step.clone()
        }

        fn get_manual_speed(&self) -> Option<u8> {
            self.manual_speed
        }

        fn get_verbose(&self) -> clap_verbosity_flag::Verbosity {
            self.verbose
        }

        fn get_pwm_channel(&self) -> u8 {
            self.pwm_channel
        }

        fn get_pwm_freq(&self) -> f64 {
            self.pwm_freq
        }

        fn get_sleep_secs(&self) -> u64 {
            self.sleep_secs
        }
    }

    fn percentage_in_range(s: &str) -> Result<u8, String> {
        let port: usize = s
            .parse()
            .map_err(|_| format!("`{s}` isn't a percentage number"))?;
        if PERCENTAGE.contains(&port) {
            Ok(port as u8)
        } else {
            Err("Value not in percentage range 0-100".to_string())
        }
    }
}
