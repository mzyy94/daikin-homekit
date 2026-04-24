Daikin Matter Bridge
---

<a href="https://github.com/mzyy94/daikin-matter/releases"><img src="https://img.shields.io/github/release/mzyy94/daikin-matter.svg" alt="Latest Release"></a>
<a href="https://github.com/mzyy94/daikin-matter/actions"><img src="https://github.com/mzyy94/daikin-matter/actions/workflows/build.yml/badge.svg" alt="Build Status"></a>

Control Daikin Air Conditioner via Matter. Compatible with new Daikin API. ([Legacy API] is not supported.)

> [!NOTE]
> v0.3 and later use Matter. For HomeKit support, see [v0.2.4](https://github.com/mzyy94/daikin-homekit/releases/tag/v0.2.4) and [homekit branch](https://github.com/mzyy94/daikin-homekit/tree/homekit).

![daikin-homekit](/docs/daikin-homekit.png)


[Legacy API]: https://github.com/ael-code/daikin-control/wiki/API-System


## Usage

```
$ daikin-matter
```

Open your Matter controller (Apple Home, Google Home, Home Assistant, etc.) and commission the bridge using the QR code displayed in the terminal.

By default, a device is automatically discovered at startup when run the command without any arguments.
If you want to specify a device, run with the IP address as an argument. Run `daikin-matter -h` for more detail.

## Installation

Get and unarchive latest release from [Releases Page](https://github.com/mzyy94/daikin-matter/releases) and install it with the following command.

```bash
$ install -m0755 daikin-matter /usr/local/bin/
```

On Linux, the Avahi daemon is required for mDNS discovery:

```bash
$ sudo apt-get install avahi-daemon
```

## Build

```bash
$ git clone https://github.com/mzyy94/daikin-matter
$ cd daikin-matter
$ cargo build --release
$ install target/release/daikin-matter /usr/local/bin/
```

On Linux, build with the `avahi` feature instead of the default:

```bash
$ cargo build --release --no-default-features --features avahi
```

## Debug

```bash
$ RUST_LOG=daikin_matter=debug daikin-matter
```

## Compatibility

The app is compatible with year 2022 or later model Daikin Air Conditioners.
It has been tested on [Daikin risora] which has built-in Wi-Fi modules and an IR remote control like the following.

[Daikin risora]: https://www.ac.daikin.co.jp/kabekake/products/sx_series

<img alt="risora ir remote display" src="/docs/remote.png" width="540">

To check compatibility, run a command below.

```bash
$ cargo run --example compatibility_check <your device ip address>
```

![compatibility_check](/docs/compatibility_check.png)

## License

GPL-3.0
