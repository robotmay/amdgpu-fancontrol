# amdgpu-fancontrol

[![builds.sr.ht status](https://builds.sr.ht/~robotmay/amdgpu-fancontrol.svg)](https://builds.sr.ht/~robotmay/amdgpu-fancontrol?)

## What is this?

A fan controller daemon for Linux to control your AMD graphics cards, built in Rust.

## Should I use this?

Maybe. Use at your own risk.

## Does it work?

Yes.

## Installation

### Ubuntu/Debian
For Ubuntu/Debian you can download a .deb binary from [the releases page](https://git.sr.ht/~robotmay/amdgpu-fancontrol/refs). Then install it with:

```
sudo dpkg -i amdgpu-fancontrol_0.1.2_amd64.deb
sudo systemctl enable amdgpu-fancontrol.service
```

Now skip to the configuration section.

## Pre-built binary

On [each release](https://git.sr.ht/~robotmay/amdgpu-fancontrol/refs) there is also a pre-built binary attached, which can be used, in theory, on other
distributions. You will need to add your own systemd service or equivalent, and point it to a valid config file.

## Building a .deb with Rust

To build and install this under Ubuntu/Debian, you will first need [Rust](https://www.rust-lang.org) installed, then run:

```
cargo deb --install
sudo systemctl enable amdgpu-fancontrol.service
```

## Configuration

Configure your card if needed at: `/etc/amdgpu-fancontrol/config.toml`. You can find your cards (on Ubuntu/Debian, at least), at: `/sys/class/drm/`.
In theory multiple cards are supported, but I don't own multiple cards, so _bon chance_.
If you want to adjust the window used to decide whether the fan can adjust downwards, you can specify it in seconds. Default is `30`.

```toml
cards = ["card0"]
fan_wind_down = 30
cards_path = "/sys/class/drm"
endpoint_path = "device/hwmon/hwmon0"
```

The two paths get joined together with the card names, for example: `/sys/class/drm/card0/device/hwmon/hwmon0`.

Start the service:

```
sudo systemctl start amdgpu-fancontrol.service
```

## Potential problems

This works well on my machine, which is running a Sapphire RX 580 Nitro under Ubuntu 20.04, but I am not sure what issues could occur with different setups.
If something weird happens and you want to quickly restore hardware fan control, first disable the service so it doesn't start on next boot:

```
sudo systemctl disable amdgpu-fancontrol.service
```

Then you can either stop the running service with:

```
sudo systemctl stop amdgpu-fancontrol.service
```

which _should_ restore hardware control (although it seems to result in the fans running higher by default), or alternatively you can manually restore it with:

```
sudo echo "2" > /sys/class/drm/card0/device/hwmon/hwmon0/pwm1_enable
```

Lastly, rebooting after disabling the service should restore everything to normal.

## Running the tests

Tests currently have to be run serially due to file modification gubbins, this can be done either by setting:

```
RUST_TEST_THREADS=1
```

in your environment, and running `cargo test`, or by running the tests with:

```
cargo test -- --test-threads=1
```

## Contributing

Patches or questions are welcome via the [mailing list](https://lists.sr.ht/~robotmay/amdgpu-fancontrol).
