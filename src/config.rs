use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use winit::event::VirtualKeyCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub keybinds: KeyBinds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinds {
    #[serde(rename = "PreviousImage")]
    pub previous_image: String,
    #[serde(rename = "NextImage")]
    pub next_image: String,
    #[serde(rename = "Exit")]
    pub exit: String,
    #[serde(rename = "SetWallpaper")]
    pub set_wallpaper: String,
    #[serde(rename = "DeleteImage")]
    pub delete_image: String,
    #[serde(rename = "ConfirmDelete")]
    pub confirm_delete: String,
    #[serde(rename = "CancelDelete")]
    pub cancel_delete: String,
    #[serde(rename = "AlwaysDelete")]
    pub always_delete: String,
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            previous_image: "h, Left".to_string(),
            next_image: "l, Right".to_string(),
            exit: "Escape".to_string(),
            set_wallpaper: "W".to_string(),
            delete_image: "D".to_string(),
            confirm_delete: "Y".to_string(),
            cancel_delete: "N, Escape".to_string(),
            always_delete: "A".to_string(),
        }
    }
}

pub struct ConfigManager {
    #[allow(dead_code)]
    config: Config,
    keybind_map: HashMap<VirtualKeyCode, String>,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config = Self::load_config()?;
        let keybind_map = Self::build_keybind_map(&config.keybinds);

        Ok(Self {
            config,
            keybind_map,
        })
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("eleviewr");

        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    fn load_config() -> Result<Config> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let default_config = Config::default();
            Self::save_config(&default_config)?;
            return Ok(default_config);
        }

        let config_content =
            fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config =
            toml::from_str(&config_content).context("Failed to parse config file")?;

        Ok(config)
    }

    fn save_config(config: &Config) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_content =
            toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(&config_path, config_content).context("Failed to write config file")?;

        Ok(())
    }

    fn build_keybind_map(keybinds: &KeyBinds) -> HashMap<VirtualKeyCode, String> {
        let mut map = HashMap::new();

        // Parse each keybind string and map keys to actions
        Self::parse_keys(&keybinds.previous_image)
            .iter()
            .for_each(|&key| {
                map.insert(key, "previous_image".to_string());
            });

        Self::parse_keys(&keybinds.next_image)
            .iter()
            .for_each(|&key| {
                map.insert(key, "next_image".to_string());
            });

        Self::parse_keys(&keybinds.exit).iter().for_each(|&key| {
            map.insert(key, "exit".to_string());
        });

        Self::parse_keys(&keybinds.set_wallpaper)
            .iter()
            .for_each(|&key| {
                map.insert(key, "set_wallpaper".to_string());
            });

        Self::parse_keys(&keybinds.delete_image)
            .iter()
            .for_each(|&key| {
                map.insert(key, "delete_image".to_string());
            });

        Self::parse_keys(&keybinds.confirm_delete)
            .iter()
            .for_each(|&key| {
                map.insert(key, "confirm_delete".to_string());
            });

        Self::parse_keys(&keybinds.cancel_delete)
            .iter()
            .for_each(|&key| {
                map.insert(key, "cancel_delete".to_string());
            });

        Self::parse_keys(&keybinds.always_delete)
            .iter()
            .for_each(|&key| {
                map.insert(key, "always_delete".to_string());
            });

        map
    }

    fn parse_keys(key_string: &str) -> Vec<VirtualKeyCode> {
        key_string
            .split(',')
            .filter_map(|key| Self::string_to_keycode(key.trim()))
            .collect()
    }

    fn string_to_keycode(key: &str) -> Option<VirtualKeyCode> {
        match key.to_lowercase().as_str() {
            "a" => Some(VirtualKeyCode::A),
            "b" => Some(VirtualKeyCode::B),
            "c" => Some(VirtualKeyCode::C),
            "d" => Some(VirtualKeyCode::D),
            "e" => Some(VirtualKeyCode::E),
            "f" => Some(VirtualKeyCode::F),
            "g" => Some(VirtualKeyCode::G),
            "h" => Some(VirtualKeyCode::H),
            "i" => Some(VirtualKeyCode::I),
            "j" => Some(VirtualKeyCode::J),
            "k" => Some(VirtualKeyCode::K),
            "l" => Some(VirtualKeyCode::L),
            "m" => Some(VirtualKeyCode::M),
            "n" => Some(VirtualKeyCode::N),
            "o" => Some(VirtualKeyCode::O),
            "p" => Some(VirtualKeyCode::P),
            "q" => Some(VirtualKeyCode::Q),
            "r" => Some(VirtualKeyCode::R),
            "s" => Some(VirtualKeyCode::S),
            "t" => Some(VirtualKeyCode::T),
            "u" => Some(VirtualKeyCode::U),
            "v" => Some(VirtualKeyCode::V),
            "w" => Some(VirtualKeyCode::W),
            "x" => Some(VirtualKeyCode::X),
            "y" => Some(VirtualKeyCode::Y),
            "z" => Some(VirtualKeyCode::Z),
            "escape" | "esc" => Some(VirtualKeyCode::Escape),
            "left" | "larrow" => Some(VirtualKeyCode::Left),
            "right" | "rarrow" => Some(VirtualKeyCode::Right),
            "up" | "uarrow" => Some(VirtualKeyCode::Up),
            "down" | "darrow" => Some(VirtualKeyCode::Down),
            "space" => Some(VirtualKeyCode::Space),
            "enter" | "return" => Some(VirtualKeyCode::Return),
            "tab" => Some(VirtualKeyCode::Tab),
            "backspace" => Some(VirtualKeyCode::Back),
            "delete" => Some(VirtualKeyCode::Delete),
            "home" => Some(VirtualKeyCode::Home),
            "end" => Some(VirtualKeyCode::End),
            "pageup" => Some(VirtualKeyCode::PageUp),
            "pagedown" => Some(VirtualKeyCode::PageDown),
            _ => None,
        }
    }

    pub fn get_action_for_key(&self, key: VirtualKeyCode) -> Option<&str> {
        self.keybind_map.get(&key).map(|s| s.as_str())
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}
