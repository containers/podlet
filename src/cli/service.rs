use std::fmt::{self, Display, Formatter};

use clap::{Args, ValueEnum};
use color_eyre::eyre;

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

impl TryFrom<&docker_compose_types::Service> for Service {
    type Error = color_eyre::Report;

    fn try_from(value: &docker_compose_types::Service) -> Result<Self, Self::Error> {
        let restart = value
            .restart
            .as_ref()
            .map(|s| RestartConfig::from_str(s, true))
            .transpose()
            .map_err(|error| eyre::eyre!("Service's restart value is invalid: {error}"))?;
        Ok(Self { restart })
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
