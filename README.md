Daikin HomeKit
---

<a href="https://github.com/mzyy94/daikin-homekit/releases"><img src="https://img.shields.io/github/release/mzyy94/daikin-homekit.svg" alt="Latest Release"></a>
<a href="https://github.com/mzyy94/daikin-homekit/actions"><img src="https://github.com/mzyy94/daikin-homekit/workflows/build/badge.svg" alt="Build Status"></a>

Control Daikin Air Conditioner via HomeKit. Compatible with new Daikin API. ([Legacy API] is not supported.)

![daikin-homekit](/docs/daikin-homekit.png)


[Legacy API]: https://github.com/ael-code/daikin-control/wiki/API-System


## Usage

```
$ daikin-homekit
```

Open Home.app on your iOS device and go to "Add Accessory" - "More options...".
Select your Daikin Device and input setup code **2023-0420**.

By default, a device is automatically discovered at startup when run the command without any arguments.
If you want to specify a device, run with the IP address as an argument. Run `daikin-homekit -h` for more detail.

## Installation

Get and unarchive latest release from [Releases Page](https://github.com/mzyy94/daikin-homekit/releases) and install it with the following command.

```bash
$ install -m0755 daikin-homekit /usr/local/bin/
```

## Build

```bash
$ git clone https://github.com/mzyy94/daikin-homekit
$ cd daikin-homekit
$ cargo build --release
$ install target/release/daikin-homekit /usr/local/bin/
```

## Debug

```bash
$ RUST_LOG=daikin_homekit=debug daikin-homekit
```

## Compatibility

The app is compatible with year 2022 or later model Daikin Air Conditioners.
It has been tested on [Daikin risora] which has built-in Wi-Fi modules and an IR remote control like the following.

[Daikin risora]: https://www.ac.daikin.co.jp/kabekake/products/sx_series

![remote](/docs/remote.png)

To check compatibility, run a command below.

```bash
$ cargo run --example compatibility_check <your device ip address>
```

![compatibility_check](/docs/compatibility_check.png)

## License

GPL-3.0
