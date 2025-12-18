use std::env::args;
use std::process::Command;

use aura_cli::{LedPowerCommand1, LedPowerCommand2};
use dmi_id::DMIID;
use gumdrop::{Opt, Options};
use log::error;
use rog_aura::keyboard::{AuraPowerState, LaptopAuraPower};
use rog_aura::{self, AuraDeviceType, AuraEffect, PowerZones};
use rog_dbus::list_iface_blocking;
use rog_dbus::scsi_aura::ScsiAuraProxyBlocking;
use rog_dbus::zbus_aura::AuraProxyBlocking;
use rog_dbus::zbus_slash::SlashProxyBlocking;
use rog_scsi::AuraMode;
use rog_slash::SlashMode;
use scsi_cli::ScsiCommand;
use zbus::blocking::proxy::ProxyImpl;
use zbus::blocking::Connection;

use crate::aura_cli::{AuraPowerStates, LedBrightness};
use crate::cli_opts::*;
use crate::slash_cli::SlashCommand;

mod aura_cli;
mod cli_opts;
mod scsi_cli;
mod slash_cli;

fn main() {
    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let self_version = env!("CARGO_PKG_VERSION");
    println!("Starting version {self_version}");
    let args: Vec<String> = args().skip(1).collect();

    let missing_argument_k = gumdrop::Error::missing_argument(Opt::Short('k'));
    let parsed = match CliStart::parse_args_default(&args) {
        Ok(p) => p,
        Err(err) if err.to_string() == missing_argument_k.to_string() => CliStart {
            kbd_bright: Some(LedBrightness::new(None)),
            ..Default::default()
        },
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let conn = Connection::system().unwrap();
    let supported_interfaces = match list_iface_blocking() {
        Ok(ifaces) => ifaces,
        Err(e) => {
            error!("Could not get supported interfaces: {e:?}");
            check_service("asusd");
            println!("\nError: {e}\n");
            print_info();
            return;
        }
    };

    if parsed.version {
        println!("asusctl v{}", env!("CARGO_PKG_VERSION"));
        println!();
        print_info();
    }

    if let Err(err) = do_parsed(&parsed, &supported_interfaces, conn) {
        print_error_help(&*err, &supported_interfaces);
    }
}

fn print_error_help(err: &dyn std::error::Error, supported_interfaces: &[String]) {
    check_service("asusd");
    println!("\nError: {}\n", err);
    print_info();
    println!();
    println!("Supported interfaces:\n\n{:#?}\n", supported_interfaces);
}

fn print_info() {
    let dmi = DMIID::new().unwrap_or_default();
    let board_name = dmi.board_name;
    let prod_family = dmi.product_family;
    println!("asusctl version: {}", env!("CARGO_PKG_VERSION"));
    println!(" Product family: {}", prod_family.trim());
    println!("     Board name: {}", board_name.trim());
}

fn check_service(name: &str) -> bool {
    if name != "asusd" && !check_systemd_unit_enabled(name) {
        println!(
            "\n\x1b[0;31m{} is not enabled, enable it with `systemctl enable {}\x1b[0m",
            name, name
        );
        return true;
    } else if !check_systemd_unit_active(name) {
        println!(
            "\n\x1b[0;31m{} is not running, start it with `systemctl start {}\x1b[0m",
            name, name
        );
        return true;
    }
    false
}

fn find_iface<T>(iface_name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: ProxyImpl<'static> + From<zbus::Proxy<'static>>,
{
    let conn = zbus::blocking::Connection::system().unwrap();
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "xyz.ljones.Asusd", "/").unwrap();
    let interfaces = f.get_managed_objects().unwrap();
    let mut paths = Vec::new();
    for v in interfaces.iter() {
        // let o: Vec<zbus::names::OwnedInterfaceName> = v.1.keys().map(|e|
        // e.to_owned()).collect(); println!("{}, {:?}", v.0, o);
        for k in v.1.keys() {
            if k.as_str() == iface_name {
                // println!("Found {iface_name} device at {}, {}", v.0, k);
                paths.push(v.0.clone());
            }
        }
    }
    if paths.len() > 1 {
        println!("Multiple asusd interfaces devices found");
    }
    if !paths.is_empty() {
        let mut ctrl = Vec::new();
        paths.sort_by(|a, b| a.cmp(b));
        for path in paths {
            ctrl.push(
                T::builder(&conn)
                    .path(path.clone())?
                    .destination("xyz.ljones.Asusd")?
                    .build()?,
            );
        }
        return Ok(ctrl);
    }

    Err(format!("Did not find {iface_name}").into())
}

fn do_parsed(
    parsed: &CliStart,
    supported_interfaces: &[String],
    _conn: Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::Aura(mode)) => handle_led_mode(mode)?,
        Some(CliCommand::AuraPowerOld(pow)) => handle_led_power1(pow)?,
        Some(CliCommand::AuraPower(pow)) => handle_led_power2(pow)?,
        Some(CliCommand::Slash(cmd)) => handle_slash(cmd)?,
        Some(CliCommand::Scsi(cmd)) => handle_scsi(cmd)?,
        None => {
            if (!parsed.show_supported
                && parsed.kbd_bright.is_none()
                && !parsed.next_kbd_bright
                && !parsed.prev_kbd_bright)
                || parsed.help
            {
                println!("{}", CliStart::usage());
                println!();
                if let Some(cmdlist) = CliStart::command_list() {
                    let dev_type =
                        if let Ok(proxy) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
                            // TODO: commands on all?
                            proxy
                                .first()
                                .unwrap()
                                .device_type()
                                .unwrap_or(AuraDeviceType::Unknown)
                        } else {
                            AuraDeviceType::Unknown
                        };
                    let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                    for command in commands.iter().filter(|command| {
                        if command.trim().starts_with("aura")
                            && !supported_interfaces.contains(&"xyz.ljones.Aura".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("slash")
                            && !supported_interfaces.contains(&"xyz.ljones.Slash".to_string())
                        {
                            return false;
                        }

                        if !dev_type.is_old_laptop()
                            && !dev_type.is_tuf_laptop()
                            && command.trim().starts_with("aura-power-old")
                        {
                            return false;
                        }
                        if !dev_type.is_new_laptop() && command.trim().starts_with("aura-power") {
                            return false;
                        }
                        true
                    }) {
                        println!("{}", command);
                    }
                }

                println!("\nExtra help can be requested on any command or subcommand:");
                println!(" asusctl aura --help");
                println!(" asusctl aura static --help");
            }
        }
    }

    if let Some(brightness) = &parsed.kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                match brightness.level() {
                    None => {
                        let level = aura.brightness()?;
                        println!("Current keyboard led brightness: {level:?}");
                    }
                    Some(level) => aura.set_brightness(rog_aura::LedBrightness::from(level))?,
                }
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.next_kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                let brightness = aura.brightness()?;
                aura.set_brightness(brightness.next())?;
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.prev_kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                let brightness = aura.brightness()?;
                aura.set_brightness(brightness.prev())?;
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.show_supported {
        println!("Supported Core Functions:\n{:#?}", supported_interfaces);
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            // TODO: multiple RGB check
            let bright = aura.first().unwrap().supported_brightness()?;
            let modes = aura.first().unwrap().supported_basic_modes()?;
            let zones = aura.first().unwrap().supported_basic_zones()?;
            let power = aura.first().unwrap().supported_power_zones()?;
            println!("Supported Keyboard Brightness:\n{:#?}", bright);
            println!("Supported Aura Modes:\n{:#?}", modes);
            println!("Supported Aura Zones:\n{:#?}", zones);
            println!("Supported Aura Power Zones:\n{:#?}", power);
        } else {
            println!("No aura interface found");
        }
    }

    Ok(())
}

fn handle_slash(cmd: &SlashCommand) -> Result<(), Box<dyn std::error::Error>> {
    let slashes = find_iface::<SlashProxyBlocking>("xyz.ljones.Slash")?;

    // Handle --get/-g flag
    if cmd.get {
        for proxy in &slashes {
            println!("Slash Ledbar Current Settings:");
            println!("{}", "=".repeat(80));

            match proxy.enabled() {
                Ok(val) => println!("Enabled: {}", val),
                Err(e) => println!("Enabled: <error: {}>", e),
            }

            match proxy.brightness() {
                Ok(val) => println!("Brightness: {} (0-255)", val),
                Err(e) => println!("Brightness: <error: {}>", e),
            }

            match proxy.interval() {
                Ok(val) => println!("Interval: {} (0-5)", val),
                Err(e) => println!("Interval: <error: {}>", e),
            }

            // Mode property may have type mismatch - try to get it, but don't fail if it
            // errors
            match proxy.mode() {
                Ok(val) => println!("Mode: {:?}", val),
                Err(e) => {
                    println!("Mode: <error: {}>", e);
                    println!("  Note: This may indicate a server-side type mismatch.");
                    println!(
                        "  Please ensure asusd is rebuilt and restarted with the latest code."
                    );
                }
            }

            println!("\nPower State Settings:");

            match proxy.show_on_boot() {
                Ok(val) => println!("  Show on boot: {}", val),
                Err(e) => println!("  Show on boot: <error: {}>", e),
            }

            match proxy.show_on_shutdown() {
                Ok(val) => println!("  Show on shutdown: {}", val),
                Err(e) => println!("  Show on shutdown: <error: {}>", e),
            }

            match proxy.show_on_sleep() {
                Ok(val) => println!("  Show on sleep: {}", val),
                Err(e) => println!("  Show on sleep: <error: {}>", e),
            }

            match proxy.show_on_battery() {
                Ok(val) => println!("  Show on battery: {}", val),
                Err(e) => println!("  Show on battery: <error: {}>", e),
            }

            match proxy.show_battery_warning() {
                Ok(val) => println!("  Show battery warning: {}", val),
                Err(e) => println!("  Show battery warning: <error: {}>", e),
            }
        }
        return Ok(());
    }

    if (cmd.brightness.is_none()
        && cmd.interval.is_none()
        && cmd.show_on_boot.is_none()
        && cmd.show_on_shutdown.is_none()
        && cmd.show_on_sleep.is_none()
        && cmd.show_on_battery.is_none()
        && cmd.show_battery_warning.is_none()
        // && cmd.show_on_lid_closed.is_none()
        && cmd.mode.is_none()
        && !cmd.list
        && !cmd.enable
        && !cmd.disable)
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }

    for proxy in slashes {
        if cmd.enable {
            proxy.set_enabled(true)?;
        }
        if cmd.disable {
            proxy.set_enabled(false)?;
        }
        if let Some(brightness) = cmd.brightness {
            proxy.set_brightness(brightness)?;
        }
        if let Some(interval) = cmd.interval {
            proxy.set_interval(interval)?;
        }
        if let Some(slash_mode) = cmd.mode {
            proxy.set_mode(slash_mode)?;
        }
        if let Some(show) = cmd.show_on_boot {
            proxy.set_show_on_boot(show)?;
        }

        if let Some(show) = cmd.show_on_shutdown {
            proxy.set_show_on_shutdown(show)?;
        }
        if let Some(show) = cmd.show_on_sleep {
            proxy.set_show_on_sleep(show)?;
        }
        if let Some(show) = cmd.show_on_battery {
            proxy.set_show_on_battery(show)?;
        }
        if let Some(show) = cmd.show_battery_warning {
            proxy.set_show_battery_warning(show)?;
        }
        // if let Some(show) = cmd.show_on_lid_closed {
        //     proxy.set_show_on_lid_closed(show)?;
        // }
    }
    if cmd.list {
        let res = SlashMode::list();
        for p in &res {
            println!("{:?}", p);
        }
    }

    Ok(())
}

fn handle_scsi(cmd: &ScsiCommand) -> Result<(), Box<dyn std::error::Error>> {
    if (!cmd.list && cmd.enable.is_none() && cmd.mode.is_none() && cmd.colours.is_empty())
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }

    let scsis = find_iface::<ScsiAuraProxyBlocking>("xyz.ljones.ScsiAura")?;

    for scsi in scsis {
        if let Some(enable) = cmd.enable {
            scsi.set_enabled(enable)?;
        }

        if let Some(mode) = cmd.mode {
            dbg!(mode as u8);
            scsi.set_led_mode(mode).unwrap();
        }

        let mut mode = scsi.led_mode_data()?;
        let mut do_update = false;
        if !cmd.colours.is_empty() {
            for (count, c) in cmd.colours.iter().enumerate() {
                if count == 0 {
                    mode.colour1 = *c;
                }
                if count == 1 {
                    mode.colour2 = *c;
                }
                if count == 2 {
                    mode.colour3 = *c;
                }
                if count == 3 {
                    mode.colour4 = *c;
                }
            }
            do_update = true;
        }

        if let Some(speed) = cmd.speed {
            mode.speed = speed;
            do_update = true;
        }

        if let Some(dir) = cmd.direction {
            mode.direction = dir;
            do_update = true;
        }

        if do_update {
            scsi.set_led_mode_data(mode.clone())?;
        }

        // let mode_ret = scsi.led_mode_data()?;
        // assert_eq!(mode, mode_ret);
        println!("{mode}");
    }

    if cmd.list {
        let res = AuraMode::list();
        for p in &res {
            println!("{:?}", p);
        }
    }

    Ok(())
}

fn handle_led_mode(mode: &LedModeCommand) -> Result<(), Box<dyn std::error::Error>> {
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;

    // Handle --get/-g flag
    if mode.get {
        for aura_proxy in &aura {
            let current_mode = aura_proxy.led_mode()?;
            let all_mode_data = aura_proxy.all_mode_data()?;

            println!("Current Aura Mode: {:?}", current_mode);
            println!("\nAll Aura Mode Options:");
            println!("{}", "=".repeat(80));

            for (mode_num, effect) in &all_mode_data {
                let mode_name = <&str>::from(mode_num);
                let is_current = *mode_num == current_mode;
                let current_marker = if is_current { " (current)" } else { "" };

                println!("\nMode: {}{}", mode_name, current_marker);
                println!("  Zone: {:?}", effect.zone);
                println!(
                    "  Colour 1: RGB({}, {}, {}) / #{:02x}{:02x}{:02x}",
                    effect.colour1.r,
                    effect.colour1.g,
                    effect.colour1.b,
                    effect.colour1.r,
                    effect.colour1.g,
                    effect.colour1.b
                );

                // Only show colour2 if it's not black (default)
                if effect.colour2.r != 0 || effect.colour2.g != 0 || effect.colour2.b != 0 {
                    println!(
                        "  Colour 2: RGB({}, {}, {}) / #{:02x}{:02x}{:02x}",
                        effect.colour2.r,
                        effect.colour2.g,
                        effect.colour2.b,
                        effect.colour2.r,
                        effect.colour2.g,
                        effect.colour2.b
                    );
                }

                println!("  Speed: {:?}", effect.speed);
                println!("  Direction: {:?}", effect.direction);
            }
        }
        return Ok(());
    }

    if mode.command.is_none() && !mode.prev_mode && !mode.next_mode {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedModeCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            // TODO: multiple rgb check
            let modes = aura.first().unwrap().supported_basic_modes()?;
            for command in commands.iter().filter(|command| {
                for mode in &modes {
                    let mut mode = <&str>::from(mode).to_string();
                    if let Some(pos) = mode.chars().skip(1).position(|c| c.is_uppercase()) {
                        mode.insert(pos + 1, '-');
                    }
                    if command.trim().starts_with(&mode.to_lowercase()) {
                        return true;
                    }
                }
                // TODO
                // if !supported.basic_zones.is_empty() && command.trim().starts_with("multi") {
                //     return true;
                // }
                false
            }) {
                println!("{}", command);
            }
        }

        println!("\nHelp can also be requested on modes, e.g: static --help");
        return Ok(());
    }

    if mode.next_mode && mode.prev_mode {
        println!("Please specify either next or previous");
        return Ok(());
    }
    if mode.next_mode {
        for aura in aura {
            let mode = aura.led_mode()?;
            let modes = aura.supported_basic_modes()?;
            let mut pos = modes.iter().position(|m| *m == mode).unwrap() + 1;
            if pos >= modes.len() {
                pos = 0;
            }
            aura.set_led_mode(modes[pos])?;
        }
    } else if mode.prev_mode {
        for aura in aura {
            let mode = aura.led_mode()?;
            let modes = aura.supported_basic_modes()?;
            let mut pos = modes.iter().position(|m| *m == mode).unwrap();
            if pos == 0 {
                pos = modes.len() - 1;
            } else {
                pos -= 1;
            }
            aura.set_led_mode(modes[pos])?;
        }
    } else if let Some(mode) = mode.command.as_ref() {
        if mode.help_requested() {
            println!("{}", mode.self_usage());
            return Ok(());
        }
        for aura in aura {
            aura.set_led_mode_data(<AuraEffect>::from(mode))?;
        }
    }

    Ok(())
}

fn handle_led_power1(power: &LedPowerCommand1) -> Result<(), Box<dyn std::error::Error>> {
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
    for aura in aura {
        let dev_type = aura.device_type()?;
        if !dev_type.is_old_laptop() && !dev_type.is_tuf_laptop() {
            println!("This option applies only to keyboards 2021+");
        }

        if power.awake.is_none()
            && power.sleep.is_none()
            && power.boot.is_none()
            && !power.keyboard
            && !power.lightbar
        {
            if !power.help {
                println!("Missing arg or command\n");
            }
            println!("{}\n", power.self_usage());
            return Ok(());
        }

        if dev_type.is_old_laptop() || dev_type.is_tuf_laptop() {
            handle_led_power_1_do_1866(&aura, power)?;
            return Ok(());
        }
    }

    println!("These options are for keyboards of product ID 0x1866 or TUF only");
    Ok(())
}

fn handle_led_power_1_do_1866(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand1,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut states = Vec::new();
    if power.keyboard {
        states.push(AuraPowerState {
            zone: PowerZones::Keyboard,
            boot: power.boot.unwrap_or_default(),
            awake: power.awake.unwrap_or_default(),
            sleep: power.sleep.unwrap_or_default(),
            shutdown: false,
        });
    }
    if power.lightbar {
        states.push(AuraPowerState {
            zone: PowerZones::Lightbar,
            boot: power.boot.unwrap_or_default(),
            awake: power.awake.unwrap_or_default(),
            sleep: power.sleep.unwrap_or_default(),
            shutdown: false,
        });
    }

    let states = LaptopAuraPower { states };
    aura.set_led_power(states)?;
    Ok(())
}

fn handle_led_power2(power: &LedPowerCommand2) -> Result<(), Box<dyn std::error::Error>> {
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
    for aura in aura {
        let dev_type = aura.device_type()?;
        if !dev_type.is_new_laptop() {
            println!("This option applies only to keyboards 2021+");
            continue;
        }

        if power.command().is_none() {
            if !power.help {
                println!("Missing arg or command\n");
            }
            println!("{}\n", power.self_usage());
            println!("Commands available");

            if let Some(cmdlist) = LedPowerCommand2::command_list() {
                let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                for command in &commands {
                    println!("{}", command);
                }
            }

            println!("\nHelp can also be requested on commands, e.g: boot --help");
            return Ok(());
        }

        if let Some(pow) = power.command.as_ref() {
            if pow.help_requested() {
                println!("{}", pow.self_usage());
                return Ok(());
            }

            let mut states = aura.led_power()?;
            let mut set = |zone: PowerZones, set_to: &AuraPowerStates| {
                for state in states.states.iter_mut() {
                    if state.zone == zone {
                        state.boot = set_to.boot;
                        state.awake = set_to.awake;
                        state.sleep = set_to.sleep;
                        state.shutdown = set_to.shutdown;
                        break;
                    }
                }
            };

            if let Some(cmd) = &power.command {
                match cmd {
                    aura_cli::SetAuraZoneEnabled::Keyboard(k) => set(PowerZones::Keyboard, k),
                    aura_cli::SetAuraZoneEnabled::Logo(l) => set(PowerZones::Logo, l),
                    aura_cli::SetAuraZoneEnabled::Lightbar(l) => set(PowerZones::Lightbar, l),
                    aura_cli::SetAuraZoneEnabled::Lid(l) => set(PowerZones::Lid, l),
                    aura_cli::SetAuraZoneEnabled::RearGlow(r) => set(PowerZones::RearGlow, r),
                    aura_cli::SetAuraZoneEnabled::Ally(r) => set(PowerZones::Ally, r),
                }
            }

            aura.set_led_power(states)?;
        }
    }

    Ok(())
}

fn check_systemd_unit_active(name: &str) -> bool {
    if let Ok(out) = Command::new("systemctl")
        .arg("is-active")
        .arg(name)
        .output()
    {
        let buf = String::from_utf8_lossy(&out.stdout);
        return !buf.contains("inactive") && !buf.contains("failed");
    }
    false
}

fn check_systemd_unit_enabled(name: &str) -> bool {
    if let Ok(out) = Command::new("systemctl")
        .arg("is-enabled")
        .arg(name)
        .output()
    {
        let buf = String::from_utf8_lossy(&out.stdout);
        return buf.contains("enabled") || buf.contains("linked");
    }
    false
}
