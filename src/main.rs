use log4rs::encode::pattern::PatternEncoder;

use std::fmt::Debug;
use std::io::{self};

use log::{debug, error, info, warn};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

use num_traits::cast::ToPrimitive;

use rppal::system::DeviceInfo;
use std::env;
use std::fs;
use std::ops::RangeInclusive;

use rppal::pwm::{Channel, Polarity, Pwm};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct CliArgs {
    #[arg(short, long, default_value_t = 21)]
    bcm_pin: u8,

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
}

const PERCENTAGE: RangeInclusive<usize> = 1..=100;

fn percentage_in_range(s: &str) -> Result<u8, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a percentage number"))?;
    if PERCENTAGE.contains(&port) {
        Ok(port as u8)
    } else {
        Err(format!("Value not in percentage range 0-100"))
    }
}

const TEMP_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LED: u8 = 23;

fn main() -> Result<(), std::io::Error> {
    // -> Result<(), Box<dyn Error>>
    // parse CLI cli_args
    let cli_args = CliArgs::parse();

    // https://medium.com/nerd-for-tech/logging-in-rust-e529c241f92e
    // https://tms-dev-blog.com/log-to-a-file-in-rust-with-log4rs/
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%Y-%m-%d %H:%M:%S)(local)} - {l}: {m}{n})}",
        )))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder().appender("stdout").build(
                cli_args
                    .verbose
                    .log_level()
                    .expect("Verbosity should be convertible to LevelFilter")
                    .to_level_filter(),
            ),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    //println!("cli_args: {:#?} - {:#?}", cli_args.speed_step, cli_args.temp_step);

    if cli_args.temp_step.len() != cli_args.speed_step.len() {
        error!("The number of temperature steps must match the number of speed steps");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The number of temperature steps must match the number of speed steps",
        ));
    }

    _print_os_info();

    if !in_container::in_container() {
        let device_info = {
            match DeviceInfo::new() {
                Ok(device_info) => {
                    debug!(
                        "Device: {} (SoC: {})",
                        device_info.model(),
                        device_info.soc()
                    );
                    device_info
                }
                Err(rppal::system::Error::UnknownModel) => {
                    error!("Unknown model from rppal");
                    return Err(io::Error::new(io::ErrorKind::Other, "Unknown device model"));
                }
            }
        };

        if let Ok(device_info) = DeviceInfo::new() {
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
        }
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
    debug!("{}", env::consts::OS); // Prints the current OS.

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
    if cli_args.manual_speed.is_some() {
        let val = cli_args.manual_speed.unwrap();
        debug!("manual speed: {}", val);
        return val;
    }

    let mut speed: u8 = *cli_args.speed_step.last().unwrap();
    let last_temp = *cli_args.temp_step.last().unwrap();

    info!("{}", temp);

    // temp below first value
    if temp < cli_args.temp_step[0] {
        debug!("min speed: {}", cli_args.speed_step[0]);
        return cli_args.speed_step[0];
    }

    if temp > last_temp {
        debug!("max speed: {}", speed);
        return speed;
    }

    for (i, &step_temp) in cli_args.temp_step.iter().enumerate() {
        let next_step_temp = cli_args.temp_step[i + 1];

        info!("Temperature step[{}]: {}", i, step_temp);

        if (temp >= step_temp) && (temp <= next_step_temp) {
            // Linear interpolation
            let temp_range = next_step_temp - step_temp;
            let speed_range = cli_args.speed_step[i + 1] - cli_args.speed_step[i];
            let temp_diff = temp - step_temp;
            speed = cli_args.speed_step[i] + (speed_range * temp_diff) / temp_range;
            debug!("Linear interpolation: {}", speed);
            return speed;
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
            Ok(f) => f.round().to_u8().unwrap(),
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
        Channel::try_from(cli_args.pwm_channel)?,
        cli_args.pwm_freq,
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
