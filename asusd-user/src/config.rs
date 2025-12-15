use std::path::PathBuf;

use config_traits::{StdConfig, StdConfigLoad};
use rog_aura::effects::{AdvancedEffects as AuraSequences, Breathe, DoomFlicker, Effect, Static};
use rog_aura::keyboard::LedCode;
use rog_aura::{Colour, Speed};
use serde::{Deserialize, Serialize};

const ROOT_CONF_DIR: &str = "rog";

fn root_conf_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.push(ROOT_CONF_DIR);
    dir
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigAura {
    pub name: String,
    pub aura: AuraSequences,
}

impl ConfigAura {
    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

impl Default for ConfigAura {
    fn default() -> Self {
        let mut seq = AuraSequences::new(false);
        let mut key = Effect::Breathe(Breathe::new(
            LedCode::W,
            Colour {
                r: 255,
                g: 0,
                b: 20,
            },
            Colour {
                r: 20,
                g: 255,
                b: 0,
            },
            Speed::Low,
        ));

        seq.push(key.clone());
        key.set_led(LedCode::A);
        seq.push(key.clone());
        key.set_led(LedCode::S);
        seq.push(key.clone());
        key.set_led(LedCode::D);
        seq.push(key);

        let key = Effect::Breathe(Breathe::new(
            LedCode::F,
            Colour { r: 255, g: 0, b: 0 },
            Colour { r: 255, g: 0, b: 0 },
            Speed::High,
        ));
        seq.push(key);

        let mut key = Effect::Static(Static::new(LedCode::RCtrl, Colour { r: 0, g: 0, b: 255 }));
        seq.push(key.clone());
        key.set_led(LedCode::LCtrl);
        seq.push(key.clone());
        key.set_led(LedCode::Esc);
        seq.push(key);

        let key = Effect::DoomFlicker(DoomFlicker::new(
            LedCode::N9,
            Colour { r: 0, g: 0, b: 255 },
            80,
            40,
        ));
        seq.push(key);

        Self {
            name: "aura-default".to_owned(),
            aura: seq,
        }
    }
}

impl StdConfig for ConfigAura {
    fn new() -> Self {
        Self::default()
    }

    fn file_name(&self) -> String {
        format!("{}.ron", self.name)
    }

    fn config_dir() -> std::path::PathBuf {
        root_conf_dir()
    }
}

impl StdConfigLoad for ConfigAura {}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ConfigBase {
    /// Name of active aura config file in the user config directory
    pub active_aura: Option<String>,
}

impl StdConfig for ConfigBase {
    fn new() -> Self {
        Self {
            active_aura: Some("aura-default".to_owned()),
        }
    }

    fn file_name(&self) -> String {
        "rog-user.ron".to_owned()
    }

    fn config_dir() -> std::path::PathBuf {
        root_conf_dir()
    }
}

impl StdConfigLoad for ConfigBase {}
