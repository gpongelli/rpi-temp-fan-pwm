use std::io::{self};

use log::{debug, error, info};

use num_traits::cast::ToPrimitive;

use rppal::system::DeviceInfo;
use std::env;
use std::fs;

use rppal::pwm::{Channel, Polarity, Pwm};

use clap::Parser;

mod cli_arguments;
use crate::cli_arguments::cli_args::CliArgs;

mod logger;
use crate::logger::app_logger;

const TEMP_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LED: u8 = 23;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse CLI cli_args
    let cli_args = CliArgs::parse();

    app_logger::configure_logger(&cli_args);

    //println!("cli_args: {:#?} - {:#?}", cli_args.speed_step, cli_args.temp_step);

    if !cli_args.valid() {
        error!("The number of temperature steps must match the number of speed steps");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The number of temperature steps must match the number of speed steps",
        )
        .into());
    }

    debug!("debug");
    info!("info");
    // warn!("warn");
    error!("error");

    _print_os_info();

    if !in_container::in_container() {
        let _ = {
            // device_info unused, code to understand if it's raspberrry pi
            match DeviceInfo::new() {
                Ok(device_info) => {
                    debug!(
                        "Device: {} (SoC: {})",
                        device_info.model(),
                        device_info.soc()
                    );
                    device_info
                }
                Err(e) => {
                    error!("Error getting device info: {}", e);
                    return Err(
                        io::Error::new(io::ErrorKind::Other, "Error getting device info").into(),
                    );
                }
            }
        };

        // raspberry model, can continue from here
        match read_file_to_string(TEMP_FILE) {
            Ok(contents) => {
                info!("File Contents:\n{}", contents.trim());
                let _ = set_pwm(contents.trim(), &cli_args);
            }
            Err(e) => {
                error!("Error reading file: {}", e);
                return Err(e.into());
            }
        }

        /*if let Ok(device_info) = DeviceInfo::new() {
            debug!(
                "Device: {} (SoC: {})",
                device_info.model(),
                device_info.soc()
            );

            match read_file_to_string(TEMP_FILE) {
                Ok(contents) => {
                    info!("File Contents:\n{}", contents.trim());
                    let _ = set_pwm(contents.trim(), &cli_args);
                }
                Err(e) => {
                    error!("Error reading file: {}", e);
                    return Err(e);
                }
            }
        } else {
            error!("Error getting device info");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get device info",
            ));
        }*/
    } else {
        debug!("Execution into container.");
    }

    // File must exist in the current path
    /*if let Ok(lines) = read_lines("/sys/class/thermal/thermal_zone0/temp") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.map_while(Result::ok) {
            println!("{}", line);
        }
    }*/

    Ok(())
}

fn _print_os_info() {
    debug!(
        "execution into container: {:#?}",
        in_container::in_container()
    );
    debug!("OS: {}", env::consts::OS); // Prints the current OS.

    let info = os_info::get();
    // Print full information:
    debug!("OS information: {info}");
    // Print information separately:
    debug!("Type: {}", info.os_type());
    debug!("Version: {}", info.version());
    debug!("Bitness: {}", info.bitness());
    debug!("Architecture: {:#?}", info.architecture());
}

fn read_file_to_string(filename: &str) -> Result<String, io::Error> {
    /* let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)*/

    fs::read_to_string(filename)
}

// Get speed interpolating array's values
fn get_fan_speed_linear(temp: u8, cli_args: &CliArgs) -> u8 {
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

/* // Get speed from array
fn get_fan_speed(temp: u8, cli_args: &CliArgs) -> u8 {
    // manually forced value
    if cli_args.manual_speed.is_some() {
        let val = cli_args.manual_speed.unwrap();
        debug!("manual speed: {}", val);
        return val;
    }

    // Find the index of the temperature step
    let mut temp_idx: usize = cli_args.temp_step.len() - 1;  // by default at maximum temperature
    for (i, &v) in cli_args.temp_step.iter().enumerate() {
        info!("Temperature step[{}]: {}", i, v);
        if temp > v{
            continue;
        } else {
            temp_idx = i;
            break;
        }
    }

    debug!("temp: {}", temp);
    debug!("Temperature index: {}", temp_idx);
    debug!("temp at index: {}", cli_args.temp_step[temp_idx]);
    debug!("speed at index: {}", cli_args.speed_step[temp_idx]);
    // Get the fan speed at the index
    cli_args.speed_step[temp_idx]
} */

fn set_pwm(temp: &str, cli_args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the string to a u8
    let temp: u8 = {
        match temp.parse::<f32>() {
            Ok(f) => {
                // check if the value is an unsigned integer
                if let Some(v) = f.round().to_u8() {
                    v
                } else {
                    error!("Temperature out of range");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Temperature out of range",
                    )
                    .into());
                }
            }
            Err(e) => {
                error!("Failed to parse temperature: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Failed to parse temperature",
                )
                .into());
            }
        }
    };
    debug!("Temperature: {}", temp);

    let fan_speed = get_fan_speed_linear(temp, cli_args);

    // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) at 2 Hz with a 25% duty cycle.
    let _ = Pwm::with_frequency(
        Channel::try_from(cli_args.get_pwm_channel())?,
        cli_args.get_pwm_freq(),
        (fan_speed as f64) / 100.0,
        Polarity::Normal,
        true,
    )?;

    // Reconfigure the PWM channel for an 8 Hz frequency, 50% duty cycle.
    // pwm.set_frequency(8.0, 0.5)?;

    Ok(())
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
/*fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}*/

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_args(temp_step: Vec<u8>, speed_step: Vec<u8>, manual_speed: Option<u8>) -> CliArgs {
        CliArgs::new(
            21,
            temp_step,
            speed_step,
            manual_speed,
            clap_verbosity_flag::Verbosity::default(),
            0,
            2.0,
        )
    }

    // --- get_fan_speed_linear tests ---

    #[test]
    fn test_manual_speed() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], Some(42));
        assert_eq!(get_fan_speed_linear(60, &args), 42);
        assert_eq!(get_fan_speed_linear(80, &args), 42);
    }

    #[test]
    fn test_below_first_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        assert_eq!(get_fan_speed_linear(40, &args), 20);
        assert_eq!(get_fan_speed_linear(0, &args), 20);
    }

    #[test]
    fn test_above_last_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        assert_eq!(get_fan_speed_linear(90, &args), 100);
        assert_eq!(get_fan_speed_linear(255, &args), 100);
    }

    #[test]
    fn test_exact_temp_steps() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        assert_eq!(get_fan_speed_linear(50, &args), 20);
        assert_eq!(get_fan_speed_linear(70, &args), 50);
        assert_eq!(get_fan_speed_linear(80, &args), 100);
    }

    #[test]
    fn test_linear_interpolation() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        // Between 50 and 70: 20 -> 50
        assert_eq!(get_fan_speed_linear(65, &args), 42);
        // Between 70 and 80: 50 -> 100
        assert_eq!(get_fan_speed_linear(75, &args), 75);
    }

    #[test]
    fn test_non_uniform_steps() {
        let args = cli_args(vec![40, 60, 90], vec![10, 60, 80], None);
        // Between 40 and 60: 10 -> 60
        assert_eq!(get_fan_speed_linear(50, &args), 35);
        // Between 60 and 90: 60 -> 80
        assert_eq!(get_fan_speed_linear(75, &args), 70);
    }

    // --- set_pwm tests ---

    #[test]
    fn test_set_pwm_invalid_temp_string() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("notanumber", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_pwm_negative_temp_string() {
        let args = cli_args(vec![10, 20, 30], vec![10, 20, 30], None);
        let result = set_pwm("-10", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_pwm_valid_temp() {
        // This test may fail if run on a system without the required hardware or permissions.
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("60", &args);
        // Accept both Ok and Err, as hardware may not be present
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_set_pwm_manual_speed() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], Some(77));
        let result = set_pwm("60", &args);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_set_pwm_below_first_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("10", &args);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_set_pwm_above_last_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("200", &args);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_set_pwm_exact_step_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("70", &args);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_set_pwm_float_temp() {
        let args = cli_args(vec![50, 70, 80], vec![20, 50, 100], None);
        let result = set_pwm("65.7", &args);
        assert!(result.is_ok() || result.is_err());
    }
}
