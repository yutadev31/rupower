use crate::config::Config;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Poweroff,
    Reboot,
    Suspend,
    Logout,
    Lock,
}

impl Action {
    const ALL: [Self; 5] = [
        Self::Poweroff,
        Self::Reboot,
        Self::Suspend,
        Self::Logout,
        Self::Lock,
    ];

    pub(crate) fn all() -> impl Iterator<Item = Self> {
        Self::ALL.into_iter()
    }

    pub(crate) fn command(self) -> CommandSpec {
        match self {
            Self::Poweroff => CommandSpec::new("poweroff", &[]),
            Self::Reboot => CommandSpec::new("reboot", &[]),
            Self::Suspend => CommandSpec::new("systemctl", &["suspend"]),
            Self::Lock => CommandSpec::new("loginctl", &["lock-session"]),
            Self::Logout if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() => {
                CommandSpec::new("hyprctl", &["dispatch", "hl.dsp.exit()"])
            }
            Self::Logout if std::env::var_os("SWAYSOCK").is_some() => {
                CommandSpec::new("swaymsg", &["exit"])
            }
            Self::Logout => CommandSpec::new("i3-msg", &["exit"]),
        }
    }

    pub(crate) fn keycode(self) -> u32 {
        match self {
            Self::Poweroff => 25,
            Self::Reboot => 19,
            Self::Suspend => 31,
            Self::Logout => 38,
            Self::Lock => 37,
        }
    }

    pub(crate) fn from_keycode(keycode: u32) -> Option<Self> {
        Self::all().find(|action| action.keycode() == keycode)
    }

    pub(crate) fn enabled(config: &Config) -> impl Iterator<Item = Self> {
        Self::all().filter(move |action| match action {
            Self::Poweroff => config.actions.poweroff,
            Self::Reboot => config.actions.reboot,
            Self::Suspend => config.actions.suspend,
            Self::Logout => config.actions.logout,
            Self::Lock => config.actions.lock,
        })
    }

    pub(crate) fn color(self) -> (u8, u8, u8) {
        match self {
            Self::Poweroff => (247, 118, 142),
            Self::Reboot => (224, 175, 104),
            Self::Suspend => (125, 207, 255),
            Self::Logout => (187, 154, 247),
            Self::Lock => (158, 206, 106),
        }
    }

    pub(crate) fn text(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Poweroff => ("\u{f011}", "Power off", "SHIFT+P"),
            Self::Reboot => ("\u{f021}", "Reboot", "SHIFT+R"),
            Self::Suspend => ("\u{f186}", "Suspend", "SHIFT+S"),
            Self::Logout => ("\u{f2f5}", "Logout", "SHIFT+L"),
            Self::Lock => ("\u{f023}", "Lock", "SHIFT+K"),
        }
    }
}

pub(crate) struct CommandSpec {
    pub(crate) program: &'static str,
    pub(crate) args: &'static [&'static str],
}

impl CommandSpec {
    const fn new(program: &'static str, args: &'static [&'static str]) -> Self {
        Self { program, args }
    }
}
