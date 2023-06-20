# Seymour Life

This is a personal/professional project, which makes use of serial communications (tty over USB via UART) and Raspberry Pi GPIO to simulate long term use of a Seymour device.

Note that this project will ONLY run properly on a Raspberry Pi. It is developed on, and tested for, the Raspberry Pi 400. This should also be compatible with the Raspberry Pi 4, and Raspberry Pi 3, although they have not been explicitly tested. Older models of Raspberry Pi may work properly, however as this project is originally intended for the `aarch64`/`ARM64` architecture, and older compatibility will not be tested.

## Install

### Precompiled Binary

Pre-compiled binaries can be found in the [releases tab](https://git.blizzard.systems/blizzardfinnegan/seymourLifeRust/releases/latest). Download the preferred version to your preferred directory, then run the following command to make the code executable:

```bash
sudo chmod u+x ./seymour_life
```

To run the binary, simply run:
```bash
sudo ./seymour_life
```

Note that this command MUST be run as `sudo`/root, due to the way it interacts with GPIO. For more information, please see [the GPIO documentation](https://github.com/golemparts/rppal).

## Build From Source

Note: *At this time, this project can only reliably be built on Linux. Build instructions for Windows will be written eventually.*

To build this project from source, first, download the repository. This can be done by using the Download ZIP button, or running the following command in a terminal where `git` is installed:
```bash
git clone https://git.blizzard.systems/blizzardfinnegan/seymourLifeRust
```

Once the repository has been downloaded, the project can be built with `cargo`. This can be done using any of the listed install methods on [the Rust install website](https://rustup.rs/).

Once cargo has been installed, run the following to build the project:
```bash
cargo build --release
```

The runnable command can then be run by the following:
```bash
sudo ./target/release/seymour_life
```

You can also build without the `--release` flag, which wil take less time, but will be less optimised for the hardware. If you do this, substitue `./target/release/seymour_life` for `./target/debug/seymour_life` in the above command.

### Build Dependencies

The following dependencies are also necessary for building this project:
- `pkg-config`
- `libudev`

See below for platform specific requirements.

#### Debian-based
This applies for all distributions of Linux using the `apt` package manager, including but not limited to Debian, Ubuntu, Raspbian/Raspberry Pi OS, and Linux Mint.

```bash
sudo apt-get install librust-libudev-sys-dev librust-pkg-config-dev
```

#### Fedora-based
This applies for all distributions of Linux using the `dnf` package manager, including but not limited to CentOS, Redhat Enterprise Linux (RHEL), and Fedora.
```bash
sudo dnf install rust-libudev-sys-devel rust-pkg-config-devel
```

#### Nix
This applies to both NixOS, and any distribution where the [Nix package manager](https://nixos.org/download.html) can be installed. 

If you have the Nix package manager installed, this project comes with a `shell.nix` containing the necessary build dependencies. Simply run `nix-shell` to download the necessary dependencies. 
