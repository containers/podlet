use clap::{Args, ValueEnum};
use compose_spec::service::Restart;
use serde::Serialize;

#[derive(Args, Serialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Service {
    /// Configure if and when the service should be restarted
    #[arg(long, value_name = "POLICY")]
    restart: Option<RestartConfig>,
}

impl Service {
    pub fn is_empty(&self) -> bool {
        let Self { restart } = self;
        restart.is_none()
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
#[derive(ValueEnum, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
