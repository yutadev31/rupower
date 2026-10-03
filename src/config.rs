use std::{error::Error, fs, path::Path};

use serde::Deserialize;

const DEFAULT_PADDING: f32 = 16.0;
const DEFAULT_BUTTON_WIDTH: f32 = 104.0;
const DEFAULT_BUTTON_HEIGHT: f32 = 124.0;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) style: Style,
    pub(crate) actions: Actions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub(crate) struct Style {
    pub(crate) padding: f32,
    pub(crate) button_width: f32,
    pub(crate) button_height: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub(crate) struct Actions {
    pub(crate) poweroff: bool,
    pub(crate) reboot: bool,
    pub(crate) suspend: bool,
    pub(crate) logout: bool,
    pub(crate) lock: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: DEFAULT_PADDING,
            button_width: DEFAULT_BUTTON_WIDTH,
            button_height: DEFAULT_BUTTON_HEIGHT,
        }
    }
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            poweroff: true,
            reboot: true,
            suspend: false,
            logout: true,
            lock: true,
        }
    }
}

impl Config {
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let Some(config_dir) = dirs::config_dir() else {
            return Ok(Self::default());
        };
        let path = config_dir.join("rupower").join("config.toml");

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let config: Self = toml::from_str(&contents)
                    .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
                config.style.validate(&path)?;
                Ok(config)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!("failed to read {}: {error}", path.display()).into()),
        }
    }
}

impl Style {
    fn validate(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        if !self.padding.is_finite()
            || self.padding < 0.0
            || !self.button_width.is_finite()
            || self.button_width <= 0.0
            || !self.button_height.is_finite()
            || self.button_height <= 0.0
        {
            return Err(format!(
                "invalid dimensions in {}: padding must be non-negative, button_width and button_height must be positive",
                path.display()
            )
            .into());
        }
        Ok(())
    }
}
