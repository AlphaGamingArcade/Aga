<div align="center">

![logo](docs/aga-banner-2.png)

# [AGA](https://aga.network)

[![Twitter URL](https://img.shields.io/twitter/follow/Aga?style=social)](https://twitter.com/Aga) [![Telegram](https://img.shields.io/endpoint?color=neon&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2FAllfeat_musi)](https://t.me/Allfeat_fndn) [![Discord](https://img.shields.io/badge/Discord-gray?logo=discord)](https://allfeat.discord.com)
[![GitHub code lines](https://tokei.rs/b1/github/allfeat/allfeat)](https://github.com/allfeat/allfeat) [![GitHub last commit](https://img.shields.io/github/last-commit/allfeat/allfeat?color=red&style=plastic)](https://github.com/allfeat/allfeat) [![CI](https://github.com/allfeat/allfeat/actions/workflows/checks.yml/badge.svg)](https://github.com/allfeat/allfeat/actions/workflows/checks.yml/badge.svg)
</div>

</div>

## Introduction

- **AGA: Seamless Asset Transfers Across Games**

  At the core of our blockchain lies **AGA**, a revolutionary system designed to enable seamless asset transfers between different games. AGA ensures a secure and frictionless experience for players and developers alike.

- **Game Asset Portability**

  AGA utilizes cutting-edge blockchain technology to facilitate decentralized storage and verification of game assets. By implementing a robust Proof of Authority (PoA) consensus mechanism and leveraging a cross-chain bridge, AGA ensures that assets remain secure, immutable, and auditable, empowering players to truly own their digital possessions.

- **Interoperability and Integration**

  Built with a focus on flexibility, AGA offers compatibility with EVM-compatible chains and other ecosystems. This cross-chain functionality, powered by the cross-chain bridge, allows game developers to integrate their titles effortlessly, while players can enjoy a unified experience across multiple games.

- **Player-Centric Economy**

  AGA opens the door to new economic opportunities by enabling a player-driven marketplace for in-game assets. Players can trade, sell, and purchase assets transparently, while developers benefit from increased engagement and monetization avenues.

- **Developer-Friendly Framework**

  AGA provides an intuitive platform with tools and SDKs designed for game developers to easily onboard and integrate asset transfer functionalities. The streamlined process reduces development overhead and fosters innovation within the gaming industry.

## Release

### Testnet (Pioneer Phase)

Experience the capabilities of AGA through our Aga Testnet. Test asset transfers, explore cross-game compatibility, and provide valuable feedback to shape the future of the platform. Join the community of developers and players pioneering this new era of gaming interoperability.

### Mainnet

The Mainnet launch is on the horizon. Stay connected for updates and announcements as we bring AGA to life and redefine the gaming experience.

### Tags and Runtime Versions

Each release tag includes the different versions of the runtimes corresponding to on-chain upgrades. This ensures that all changes and updates to the Allfeat network and runtime environments are fully traceable and easy to follow.

## Documentation

- [AGA Docs](https://docs.aga.com)

# Installation

This guide is for reference only, please check the latest information on getting started with Substrate [here](https://docs.substrate.io/main-docs/install/).

This page will guide you through the **2 steps** needed to prepare a computer for **Substrate** development. Since
Substrate is built with [the Rust programming language](https://www.rust-lang.org/), the first thing you will need to do
is prepare the computer for Rust development - these steps will vary based on the computer's operating system. Once Rust
is configured, you will use its toolchains to interact with Rust projects; the commands for Rust's toolchains will be
the same for all supported, Unix-based operating systems.

## Build dependencies

Substrate development is easiest on Unix-based operating systems like macOS or Linux. The examples in the [Substrate
Docs](https://docs.substrate.io) use Unix-style terminals to demonstrate how to interact with Substrate from the command
line.

### Ubuntu/Debian

Use a terminal shell to execute the following commands:

```bash
sudo apt update
# May prompt for location information
sudo apt install -y git clang curl libssl-dev llvm libudev-dev
```

### Arch Linux

Run these commands from a terminal:

```bash
pacman -Syu --needed --noconfirm curl git clang
```

### Fedora

Run these commands from a terminal:

```bash
sudo dnf update
sudo dnf install clang curl git openssl-devel
```

### OpenSUSE

Run these commands from a terminal:

```bash
sudo zypper install clang curl git openssl-devel llvm-devel libudev-devel
```

### macOS

> **Apple M1 ARM** If you have an Apple M1 ARM system on a chip, make sure that you have Apple Rosetta 2 installed
> through `softwareupdate --install-rosetta`. This is only needed to run the `protoc` tool during the build. The build
> itself and the target binaries would remain native.

Open the Terminal application and execute the following commands:

```bash
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl
brew update
brew install openssl
```

### Windows

**_PLEASE NOTE:_** Native Windows development of Substrate is _not_ very well supported! It is _highly_
recommended to use [Windows Subsystem Linux](https://docs.microsoft.com/en-us/windows/wsl/install-win10)
(WSL) and follow the instructions for [Ubuntu/Debian](#ubuntudebian).
Please refer to the separate
[guide for native Windows development](https://docs.substrate.io/main-docs/install/windows/).

## Rust developer environment

This guide uses <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain. First install and
configure `rustup`:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version, add nightly and the nightly wasm target:

```bash
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

## Test your set-up

Now the best way to ensure that you have successfully prepared a computer for Substrate development is to follow the
steps in [our first Substrate tutorial](https://docs.substrate.io/tutorials/v3/create-your-first-substrate-chain/).

## Troubleshooting Substrate builds

Sometimes you can't get the Substrate node template to compile out of the box. Here are some tips to help you work
through that.

### Rust configuration check

To see what Rust toolchain you are presently using, run:

```bash
rustup show
```

This will show something like this (Ubuntu example) output:

```text
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/user/.rustup

installed toolchains
--------------------

stable-x86_64-unknown-linux-gnu (default)
nightly-2020-10-06-x86_64-unknown-linux-gnu
nightly-x86_64-unknown-linux-gnu

installed targets for active toolchain
--------------------------------------

wasm32-unknown-unknown
x86_64-unknown-linux-gnu

active toolchain
----------------

stable-x86_64-unknown-linux-gnu (default)
rustc 1.50.0 (cb75ad5db 2021-02-10)
```

As you can see above, the default toolchain is stable, and the `nightly-x86_64-unknown-linux-gnu` toolchain as well as
its `wasm32-unknown-unknown` target is installed. You also see that `nightly-2020-10-06-x86_64-unknown-linux-gnu` is
installed, but is not used unless explicitly defined as illustrated in the [specify your nightly
version](#specifying-nightly-version) section.

### WebAssembly compilation

Substrate uses [WebAssembly](https://webassembly.org) (Wasm) to produce portable blockchain runtimes. You will need to
configure your Rust compiler to use [`nightly` builds](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) to
allow you to compile Substrate runtime code to the Wasm target.

> There are upstream issues in Rust that need to be resolved before all of Substrate can use the stable Rust toolchain.
> [This is our tracking issue](https://github.com/paritytech/substrate/issues/1252) if you're curious as to why and how
> this will be resolved.

#### Latest nightly for Substrate `master`

Developers who are building Substrate _itself_ should always use the latest bug-free versions of Rust stable and
nightly. This is because the Substrate codebase follows the tip of Rust nightly, which means that changes in Substrate
often depend on upstream changes in the Rust nightly compiler. To ensure your Rust compiler is always up to date, you
should run:

```bash
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

> NOTE: It may be necessary to occasionally rerun `rustup update` if a change in the upstream Substrate codebase depends
> on a new feature of the Rust compiler. When you do this, both your nightly and stable toolchains will be pulled to the
> most recent release, and for nightly, it is generally _not_ expected to compile WASM without error (although it very
> often does). Be sure to [specify your nightly version](#specifying-nightly-version) if you get WASM build errors from
> `rustup` and [downgrade nightly as needed](#downgrading-rust-nightly).

#### Rust nightly toolchain

If you want to guarantee that your build works on your computer as you update Rust and other dependencies, you should
use a specific Rust nightly version that is known to be compatible with the version of Substrate they are using; this
version will vary from project to project and different projects may use different mechanisms to communicate this
version to developers. For instance, the Polkadot client specifies this information in its [release
notes](https://github.com/paritytech/polkadot-sdk/releases).

```bash
# Specify the specific nightly toolchain in the date below:
rustup install nightly-<yyyy-MM-dd>
```

#### Wasm toolchain

Now, configure the nightly version to work with the Wasm compilation target:

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly-<yyyy-MM-dd>
```

### Specifying nightly version

Use the `WASM_BUILD_TOOLCHAIN` environment variable to specify the Rust nightly version a Substrate project should use
for Wasm compilation:

```bash
WASM_BUILD_TOOLCHAIN=nightly-<yyyy-MM-dd> cargo build --release
```

> Note that this only builds _the runtime_ with the specified nightly. The rest of project will be compiled with **your
> default toolchain**, i.e. the latest installed stable toolchain.

### Downgrading Rust nightly

If your computer is configured to use the latest Rust nightly and you would like to downgrade to a specific nightly
version, follow these steps:

```bash
rustup uninstall nightly
rustup install nightly-<yyyy-MM-dd>
rustup target add wasm32-unknown-unknown --toolchain nightly-<yyyy-MM-dd>
```





## Contribution

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

AGA is open-source under the GPLv3 license. We welcome community contributions. Please review [CONTRIBUTIONS.md](doc/CONTRIBUTIONS.md) for details on how to contribute to the project.
