#!/bin/bash

cargo clean
cargo build --release

sudo cp ./target/release/rpi-temp-fan-pwm /usr/bin/rpi-temp-fan-pwm
sudo chmod +x /usr/bin/rpi-temp-fan-pwm

sudo cp ./rpi-temp-fan-pwm.service /etc/systemd/system/rpi-temp-fan-pwm.service
sudo cp ./rpi-temp-fan-pwm.timer /etc/systemd/system/rpi-temp-fan-pwm.timer

sudo systemctl daemon-reload
sudo systemctl enable rpi-temp-fan-pwm.timer
sudo systemctl start rpi-temp-fan-pwm.timer

sudo systemctl start rpi-temp-fan-pwm.service

sudo systemctl status rpi-temp-fan-pwm.timer
sudo systemctl status rpi-temp-fan-pwm.service