[![status-badge](https://ci.blizzard.systems/api/badges/blizzardfinnegan/seymourLifeRust/status.svg)](https://ci.blizzard.systems/blizzardfinnegan/seymourLifeRust)
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

To build this project from source *ON A RASPBERRY PI*, first, download the repository. This can be done by using the Download ZIP button, or running the following command in a terminal where `git` is installed:
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


## Cross-Compilation

Cross compilation is possible with this project, if you do not have a Raspberry Pi available specifically for compilation. Compilation directly on the Pi is rather intensive, and takes significantly longer than cross-compiling. 

If you are compiling on Linux, cross-compilation has a dependency of `lld`. This can be found in your distribution's package manager, or directly distributed by LLVM. For Nix users, a predefined `shell.nix` file has been provided for your convenience.

If you are compiling on Windows, to safely cross-compile, you must modify the `.cargo/config.toml` file. Replace `lld` with `rust-lld`, then cross-compilation should work properly.

```bash
cargo build --target aarch64-unknown-linux-musl
# OR
cargo build --release --target aarch64-unknown-linux-musl
```


