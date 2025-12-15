# `asusctl` for ASUS ROG (Ubuntu/Pop!_OS minimal fork)

> **⚠️ FORK NOTICE:** This is a fork of the original `asusctl` project. Many features from the original project have been removed, including the GUI (`rog-control-center`), AniMe Matrix display support (`rog-anime`), simulators, power profiles, and fan curves. This fork keeps only keyboard backlight (aura) and LED slash functionality, expecting the distro (like `system76-power` on Pop!_OS) to handle power profiles and fan curves.

> **🎯 TARGET CONFIGURATION:** This fork is built and tested with **Pop!_OS 24.04 LTS** and **ROG Zephyrus G16** in mind. It may work on other Ubuntu-based distributions and ASUS ROG laptops, but should be used with caution. Compatibility with other configurations is not guaranteed.

## Support the Original Authors

This fork is based on the excellent work of the original `asusctl` project. Please consider supporting the original authors:

[![Become a Patron!](https://github.com/codebard/patron-button-and-widgets-by-codebard/blob/master/images/become_a_patron_button.png?raw=true)](https://www.patreon.com/bePatron?u=7602281) [![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/V7V5CLU67) - [Asus Linux Website](https://asus-linux.org/)

Original project: [asus-linux/asusctl](https://gitlab.com/asus-linux/asusctl)

**WARNING:** Many features are developed in tandem with kernel patches. If you see a feature is missing you either need a patched kernel or latest release.

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

## Kernel support

Due to on-going driver work the minimum suggested kernel version is always **the latest*, as improvements and fixes are continuous.

Support for some new features is not avilable unless you run a patched kernel with the work I am doing [in this github repo](https://github.com/flukejones/linux/tree/wip/ally-6.13). Use the linked branch, or `wip/ally-6.12`. Everything that is done here is upstreamed eventually (a long process).

Z13 devices will need [these](https://lore.kernel.org/linux-input/20240416090402.31057-1-luke@ljones.dev/T/#t)

## X11 support

X11 is not supported at all, as in I will not help you with X11 issues if there are any due to limited time and it being unmaintained itself. You can however build `rog-control-center` with it enabled `cargo build --features "rog-control-center/x11"`.

## Goals

The main goal of this fork is to provide keyboard LED (aura) and LED slash control for ASUS ROG laptops on Ubuntu/Pop!_OS, while relying on the distribution for power profile and fan curve management.

1. Provide safe dbus interface
2. Respect the users resources: be small, light, and fast

Point 4? asusd currently uses a tiny fraction of cpu time, and less than 1Mb of ram, the way
a system-level daemon should. Languages such as JS and python should never be used for system level daemons (please stop).

## Keyboard LEDs

The level of support for laptops is dependent on folks submitting data to include in [`./rog-aura/data/layouts/aura_support.ron`](./rog-aura/data/layouts/aura_support.ron), typically installed in `/usr/share/asusd/aura_support.ron`. This is because the controller used for keyboards and LEDs is used across many years and many laptop models, all with different firmware configurations - the only way to track this is with the file mentioned above. Why not just enable all by default? Because it confuses people.

See the [rog-aura readme](./rog-aura/README.md) for more details.

## Discord

[![Discord](https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/B8GftRW2Hd)

> **Note:** The Discord server is for the original project. This fork may have different features and limitations.

## SUPPORTED LAPTOPS

> **⚠️ IMPORTANT:** This fork is specifically tested and optimized for **ROG Zephyrus G16**. While it may work on other ASUS gaming laptops that have a USB keyboard, compatibility is not guaranteed. Use at your own risk.

Most ASUS gaming laptops that have a USB keyboard. If `lsusb` shows something similar
to this:

```text
Bus 001 Device 002: ID 0b05:1866 ASUSTek Computer, Inc. N-KEY Device
```

or

```text
Bus 003 Device 002: ID 0b05:19b6 ASUSTek Computer, Inc. [unknown]
```

then it may work without tweaks.

## Implemented

The list reflects features available in this fork. This fork focuses solely on LED control.

- [x] System daemon
- [x] Keyboard backlight (aura) - built-in LED modes
- [x] Per-key LED setting (via asusd-user)
- [x] LED slash display control
- [x] Fancy LED modes

## Removed Features (Fork)

This fork has removed the following features to streamline the codebase:

- [ ] GUI app (`rog-control-center`) - removed
- [ ] AniMe Matrix display support (`rog-anime`) - removed
- [ ] Simulators - removed
- [ ] Power profiles - removed (use distro tools like `system76-power` on Pop!_OS)
- [ ] Fan curves - removed (use distro tools like `system76-power` on Pop!_OS)
- [ ] Battery charge limit - removed
- [ ] BIOS/EFI controls (POST sound, GPU MUX) - removed

These features may be available in the original project at [asus-linux/asusctl](https://gitlab.com/asus-linux/asusctl).

## BUILDING

Rust and cargo are required, they can be installed from [rustup.rs](https://rustup.rs/) or from the distro repos if newer than 1.75.

**fedora:**

```bash
    dnf install cmake clang-devel  libxkbcommon-devel systemd-devel expat-devel pcre2-devel libzstd-devel gtk3-devel
    make
    sudo make install
```

**openSUSE:**

Works with KDE Plasma (without GTK packages)

```bash
    zypper in -t pattern devel_basis
    zypper in rustup make cmake clang-devel libxkbcommon-devel systemd-devel expat-devel pcre2-devel libzstd-devel gtk3-devel
    make
    sudo make install
```

**Debian(unsupported):**

officially unsupported,but you can still try and test it by yourself(some features may not be available).

```bash
    sudo apt install libclang-dev libudev-dev libfontconfig-dev build-essential cmake libxkbcommon-dev
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    make
    sudo make install
```

**Ubuntu, Pop!_OS:**

```bash
    sudo apt install libclang-dev libudev-dev libfontconfig-dev build-essential cmake libxkbcommon-dev pkg-config
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    make
    sudo make install
```

## Installing

Build from source (see BUILDING section above).

=======

The default init method is to use the udev rule, this ensures that the service is
started when the device is initialised and ready.

You may also need to activate the service for Ubuntu/Pop!_OS install. Note that this fork does not handle power profiles or fan curves - use `system76-power` on Pop!_OS for those features.

## Upgrading

If you are upgrading from a previous installed version, you will need to restart the service or reboot.

```bash
systemctl daemon-reload && systemctl restart asusd
```

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd/`.

## Contributing

See `CONTRIBUTING.md`. Additionally, also do `cargo clean` and `cargo test` on first checkout to ensure the commit hooks are used (via `cargo-husky`).

Generation of the bindings with `make bindings` requires `typeshare` to be installed.

Dbus introspection XML can be generated with `make introspection` (requires asusd to be running).

## Supporting more laptops

Please file a support request.

## License & Trademarks

Mozilla Public License 2 (MPL-2.0)

---

ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.

Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.

The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.

---
