# asusctl manual

> **⚠️ FORK NOTICE:** This manual is for a fork of asusctl optimized for Ubuntu/Pop!_OS. This fork is built and tested with **Pop!_OS 24.04 LTS** and **ROG Zephyrus G16** in mind. Features related to GUI (`rog-control-center`), AniMe Matrix displays, power profiles, and fan curves have been removed in this fork. This fork focuses solely on keyboard backlight (aura) and LED slash control. It may work on other configurations but should be used with caution.

**NOTE:** this manual is in need of an update in some places. If you find issues please file issue reports.

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

## Programs Available

- `asusd`: The main system daemon. It is autostarted by a udev rule and systemd unit.
- `asusd-user`: The user level daemon for per-key RGB keyboard lighting.
- `asusctl`: The CLI for interacting with the system daemon

## `asusd`

`asusd` is the main system-level daemon which will control/load/save various settings in a safe way for the user, along with exposing a _safe_ dbus interface for these interactions. This section covers only the daemon plus the various configuration file options.

The functionality that `asusd` exposes is:

- led keyboard control (aura)
- LED slash display control

each of these will be detailed in sections.

> **Note:** AniMe Matrix display control, power profiles, fan curves, charge limiting, and BIOS controls have been removed in this fork. For those features, please refer to the original project or use distribution tools (e.g., `system76-power` on Pop!_OS for power profiles and fan curves).

### Led keyboard control

The LED controller (e.g, aura) enables setting many of the factory modes available if a laptop supports them. It also enables per-key RGB settings but this is a WIP and will likely be similar to how AniMe sequences can be created.

#### Supported laptops

There are over 80 supported laptops as of 01-01-2023. Please see [the rog-aura crate readme for further details](/rog-aura/README.md).

### LED Slash display control

Some ASUS ROG laptops feature a LED slash display (a small LED strip or display). This fork supports controlling this display through the LED slash controller.

> **Note:** Charge control, BIOS controls (POST sound, GPU MUX), power profiles, and fan curves have been removed in this fork. For power profiles and fan curves, use distribution tools like `system76-power` on Pop!_OS. For other features, refer to the original project.

### Support controller

There is one more controller; the support controller. The sole pupose of this controller is to querie all the other controllers for information about their support level for the host laptop. Returns a json string.

## asusd-user

`asusd-user` is a usermode daemon. The intended purpose is to provide a method for users to run their own custom per-key keyboard effects and modes - all without overwriting the _base_ system config.

In this fork, `asusd-user` focuses on per-key RGB keyboard lighting functionality. AniMe Matrix support has been removed.

The main config is `~/.config/rog/rog-user.cfg`

### Config options: Aura, per-key and zoned

I'm unsure of how many laptops this works on, so please try it.

`led_type: Key` works only on actual per-key RGB keyboards.

`led_type: Zone` works on zoned laptops.

`led_type: Zone` set to `None` works on zoned ROG laptops, unzoned ROG laptops, and TUF laptops (and yes this does mean an audio EQ can be done now).

`~/.config/rog/rog-user.cfg` contains a setting `"active_aura": "<FILENAME>"` where `<FILENAME>` is the name of the Aura config to use, located in the same directory and without the file postfix, e.g, `"active_anime": "aura-default"`

An Aura config itself is a file with contents:

```ron
(
    name: "aura-default",
    aura: (
        effects: [
            Breathe((
                led: W,
                start_colour1: (255, 0, 20),
                start_colour2: (20, 255, 0),
                speed: Low,
            )),
            Breathe((
                led: A,
                start_colour1: (255, 0, 20),
                start_colour2: (20, 255, 0),
                speed: Low,
            )),
            Breathe((
                led: S,
                start_colour1: (255, 0, 20),
                start_colour2: (20, 255, 0),
                speed: Low,
            )),
            Breathe((
                led: D,
                start_colour1: (255, 0, 20),
                start_colour2: (20, 255, 0),
                speed: Low,
            )),
            Breathe((
                led: F,
                start_colour1: (255, 0, 0),
                start_colour2: (255, 0, 0),
                speed: High,
            )),
            Static((
                led: RCtrl,
                colour: (0, 0, 255),
            )),
            Static((
                led: LCtrl,
                colour: (0, 0, 255),
            )),
            Static((
                led: Esc,
                colour: (0, 0, 255),
            )),
            DoomFlicker((
                led: N9,
                start_colour: (0, 0, 255),
                max_percentage: 80,
                min_percentage: 40,
            )),
        ],
        zoned: false,
    ),
)
```

If your laptop supports multizone, `"led"` can also be `"Zone": <one of the following>`

- `SingleZone` // Keyboards with only one zone
- `ZonedKbLeft` // keyboard left
- `ZonedKbLeftMid` // keyboard left-middle
- `ZonedKbRightMid` // etc
- `ZonedKbRight`
- `LightbarRight`
- `LightbarRightCorner`
- `LightbarRightBottom`
- `LightbarLeftBottom`
- `LightbarLeftCorner`
- `LightbarLeft`

Single zone example:

```ron
(
    name: "aura-default",
    aura: (
        effects: [
            DoomFlicker((
                led: SingleZone,
                start_colour: (200, 40, 5),
                max_percentage: 80,
                min_percentage: 40,
            )),
        ],
        zoned: true,
    ),
)
```

At the moment there are only three effects available as shown in the example. More will come in the future
but this may take some time.

> **Note:** AniMe Matrix configuration options have been removed in this fork. For AniMe support, please refer to the original project.

## asusctl

`asusctl` is a commandline interface which intends to be the main method of interacting with `asusd`. It can be used in any place a terminal app can be used.

This program will query `asusd` for the `Support` level of the laptop and show or hide options according to this support level.

Most commands are self-explanatory.

### CLI Usage and help

Commands are given by:

```bash
asusctl <option> <command> <command-options>
```

Help is available through:

```bash
asusctl --help
asusctl <command> --help
```

Some commands may have subcommands:

```bash
asusctl <command> <subcommand> --help
```

### Keybinds

To switch to next/previous Aura modes you will need to bind both the aura keys (if available) to one of:

### Next

```bash
asusctl aura -n
```

### Previous

```bash
asusctl aura -p
```

> **Note:** Power profile switching has been removed in this fork. Use your distribution's power management tools (e.g., `system76-power` on Pop!_OS) to switch power profiles.

## License & Trademarks

Mozilla Public License 2 (MPL-2.0)

---

ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.

Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.

The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.

---
