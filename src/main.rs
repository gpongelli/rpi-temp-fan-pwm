use std::fs::File;
//use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::io::{self, Read};

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;
use rppal::pwm::{Channel, Polarity, Pwm};

const temp_file: &str = "/sys/class/thermal/thermal_zone0/temp";

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 23;

fn main() -> Result<(), std::io::Error> {
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

    match read_file_to_string(temp_file) {
        Ok(contents) => {
            println!("File Contents:\n{}", contents.trim());
            set_pwm(contents.trim());

        },
        Err(e) => {
            return Err(e)
            /*eprintln!("Error reading file: {}", e);
            Err(e); */
        },
    }

    Ok(())
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