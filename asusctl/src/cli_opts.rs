use gumdrop::Options;

use crate::anime_cli::AnimeCommand;
use crate::aura_cli::{LedBrightness, LedPowerCommand1, LedPowerCommand2, SetAuraBuiltin};
use crate::scsi_cli::ScsiCommand;
use crate::slash_cli::SlashCommand;

#[derive(Default, Options)]
pub struct CliStart {
    #[options(help_flag, help = "print help message")]
    pub help: bool,
    #[options(help = "show program version number")]
    pub version: bool,
    #[options(help = "show supported functions of this laptop")]
    pub show_supported: bool,
    #[options(meta = "", help = "<off, low, med, high>")]
    pub kbd_bright: Option<LedBrightness>,
    #[options(help = "Toggle to next keyboard brightness")]
    pub next_kbd_bright: bool,
    #[options(help = "Toggle to previous keyboard brightness")]
    pub prev_kbd_bright: bool,
    #[options(command)]
    pub command: Option<CliCommand>,
}

#[derive(Options)]
pub enum CliCommand {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    Aura(LedModeCommand),
    #[options(help = "Set the LED power states")]
    AuraPowerOld(LedPowerCommand1),
    #[options(help = "Set the LED power states")]
    AuraPower(LedPowerCommand2),
    #[options(name = "anime", help = "Manage AniMe Matrix")]
    Anime(AnimeCommand),
    #[options(name = "slash", help = "Manage Slash Ledbar")]
    Slash(SlashCommand),
    #[options(name = "scsi", help = "Manage SCSI external drive")]
    Scsi(ScsiCommand),
}

#[derive(Options)]
pub struct LedModeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "switch to next aura mode")]
    pub next_mode: bool,
    #[options(help = "switch to previous aura mode")]
    pub prev_mode: bool,
    #[options(command)]
    pub command: Option<SetAuraBuiltin>,
}

