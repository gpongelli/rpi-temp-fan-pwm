# Raspberry PWM fan with Rust

This project is another one regarding controlling Raspberry PWM fan, but it's developed having performance in mind, so Rust was chosen and the executable is called from sysctl timer.

## Installation

Add following section to `/boot/config.txt` to enable PWM:
```text
[all]
# https://forums.raspberrypi.com/viewtopic.php?t=287786
dtoverlay=pwm,pin=18,func=2
```

Then run the shell script to launch build process and setup of systemd service.
