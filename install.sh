#!/bin/bash

#cargo clean
cargo build --release

sudo cp ./target/release/rpi-temp-fan-pwm /usr/bin/rpi-temp-fan-pwm
sudo chmod +x /usr/bin/rpi-temp-fan-pwm

sudo cp ./src/pwm-fan.service /etc/systemd/system/pwm-fan.service
sudo cp ./src/pwm-fan.timer /etc/systemd/system/pwm-fan.timer

sudo systemctl daemon-reload
sudo systemctl enable pwm-fan.timer
sudo systemctl start pwm-fan.timer

sudo systemctl start pwm-fan.service

sudo systemctl status pwm-fan.timer
sudo systemctl status pwm-fan.service