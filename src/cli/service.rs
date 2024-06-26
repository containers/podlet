use std::fmt::{self, Display, Formatter};

use clap::{Args, ValueEnum};
use compose_spec::service::Restart;

#[derive(Args, Default, Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Configure if and when the service should be restarted
    #[arg(long, value_name = "POLICY")]
    restart: Option<RestartConfig>,
}

impl Service {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Service]")?;
        if let Some(restart) = self.restart.and_then(|restart| restart.to_possible_value()) {
            writeln!(f, "Restart={}", restart.get_name())?;
        }
        Ok(())
    }
}

impl From<RestartConfig> for Service {
    fn from(restart: RestartConfig) -> Self {
        Self {
            restart: Some(restart),
        }
    }
}

impl From<Restart> for Service {
    fn from(restart: Restart) -> Self {
        RestartConfig::from(restart).into()
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
    #[value(alias = "unless-stopped")]
    Always,
}

impl From<Restart> for RestartConfig {
    fn from(value: Restart) -> Self {
        match value {
            Restart::No => Self::No,
            Restart::Always | Restart::UnlessStopped => Self::Always,
            Restart::OnFailure => Self::OnFailure,
        }
    }
}
