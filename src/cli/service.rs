use std::fmt::Display;

use clap::{Args, ValueEnum};

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    #[arg(long, value_name = "POLICY")]
    restart: Option<RestartConfig>,
}

impl Config {
    pub fn is_empty(&self) -> bool {
        self.restart.is_none()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(restart) = self.restart.and_then(|restart| restart.to_possible_value()) {
            writeln!(f, "Restart={}", restart.get_name())?;
        }
        Ok(())
    }
}

/// Possible service restart configurations
///
/// From [systemd.service](https://www.freedesktop.org/software/systemd/man/systemd.service.html#Restart=)
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum RestartConfig {
    No,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnWatchdog,
    OnAbort,
    Always,
}
