# amdgpu-fancontrol

[![builds.sr.ht status](https://builds.sr.ht/~robotmay/amdgpu-fancontrol.svg)](https://builds.sr.ht/~robotmay/amdgpu-fancontrol?)

## What is this?

A fan controller daemon for Linux to control your AMD graphics cards, built in Rust.

## Should I use this?

Maybe. Use at your own risk.

## Does it work?

Yes.

## Installation

Until I sort out something better, under Ubuntu/Debian you will currently need a Rust setup, then clone the repo and run:

```
cargo deb --install
sudo systemctl enable amdgpu-fancontrol.service
```

Configure your card if needed at: `/etc/amdgpu-fancontrol/config.toml`. You can find your cards (on Debian, at least), at: `/sys/class/drm/`.
In theory multiple cards are supported, but I don't own multiple cards, so _bon chance_.
If you want to adjust the window used to decide whether the fan can adjust downwards, you can specify it in seconds. Default is `30`.

```toml
cards = ["card0"]
fan_wind_down = 30
```

Start the service:

```
sudo systemctl start amdgpu-fancontrol.service
```

## Running the tests

Tests currently have to be run serially due to file modification gubbins, this can be done either by setting:

```
RUST_TEST_THREADS=1
```

in your environment, and running `cargo test`, or by running the tests with:

```
cargo test -- --test-threads=1
```
