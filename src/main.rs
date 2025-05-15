use std::fs::File;
//use std::io::{self, BufRead, BufReader};
//use std::path::Path;
use std::fmt::Debug;
use std::io::{self, Read};
// use std::error::Error;
use rppal::gpio::Gpio;
use log4rs::encode::pattern::PatternEncoder;

use log::{debug, error, info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

use rppal::system::DeviceInfo;
use std::env;
use std::ops::RangeInclusive;

//use rppal::pwm::{Channel, Polarity, Pwm};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct CliArgs {
    #[arg(short, long, default_value_t = 21)]
    bcm_pin: u8,

    //https://stackoverflow.com/questions/73240901/how-to-get-clap-to-process-a-single-argument-with-multiple-values-without-having
    #[arg(short,long, value_delimiter=',', default_value = "60,80", num_args = 1.., value_parser = percentage_in_range)]
    temp_step: Vec<u8>,

    #[arg(short,long, value_delimiter=',', default_value = "0,100", num_args = 1.. )]
    speed_step: Vec<u8>,

    #[arg(short, long, default_value_t = 2000)]
    pwm_freq: u16,

    #[arg(short, long, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,
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
const GPIO_LED: u8 = 23;

fn main() -> Result<(), std::io::Error> {
    // parse CLI args
    let args = CliArgs::parse();

    // https://medium.com/nerd-for-tech/logging-in-rust-e529c241f92e
    // https://tms-dev-blog.com/log-to-a-file-in-rust-with-log4rs/
    let stdout = ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new("{h({d(%Y-%m-%d %H:%M:%S)(local)} - {l}: {m}{n})}"))).build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(args.log_level))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();


    //println!("ARGS: {:#?} - {:#?}", args.speed_step, args.temp_step);
    //println!("Hello, world!");

    let device_info = DeviceInfo::new().unwrap();
    println!("Modello: {} (SoC: {})", device_info.model(), device_info.soc());
   

    // File must exist in the current path
    /*if let Ok(lines) = read_lines("/sys/class/thermal/thermal_zone0/temp") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.map_while(Result::ok) {
            println!("{}", line);
        }
    }*/

    match read_file_to_string(TEMP_FILE) {
        Ok(contents) => {
            println!("File Contents:\n{}", contents.trim());
            set_pwm(contents.trim());
        }
        Err(e) => {
            return Err(e);
            /*eprintln!("Error reading file: {}", e);
            Err(e); */
        }
    }

    Ok(())
}


fn _print_os_info() {
    
    debug!("execution into container: {:#?}", in_container::in_container());
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
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn set_pwm(temp: &str) {

    //println!("Ancora {}", temp);
    // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) at 2 Hz with a 25% duty cycle.
    //let pwm = Pwm::with_frequency(Channel::Pwm0, 2.0, 0.25, Polarity::Normal, true)?;

    // Reconfigure the PWM channel for an 8 Hz frequency, 50% duty cycle.
    //pwm.set_frequency(8.0, 0.5)?;
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
/*fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}*/
