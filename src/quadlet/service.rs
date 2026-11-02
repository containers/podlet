use clap::{Args, ValueEnum};
use color_eyre::eyre::ensure;
use compose_spec::{
    duration,
    service::{
        Restart,
        deploy::{RestartCondition, RestartPolicy},
    },
};
use serde::Serialize;

/// The `[Service]` section of a systemd unit / Quadlet file.
///
/// Only includes options needed to convert [Podman CLI](crate::cli::PodmanCommands) and
/// [`Compose`](compose_spec::Compose) files.
#[derive(Args, Serialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Service {
    /// Configure if and when the service should be restarted.
    #[arg(long, value_name = "POLICY")]
    pub restart: Option<RestartConfig>,

    /// How long to wait before restarting the service.
    ///
    /// Converts to "RestartSec=DURATION" in the [Service] section.
    #[arg(long, value_name = "DURATION")]
    pub restart_sec: Option<String>,
}

impl Service {
    /// Returns `true` if all fields are [`None`].
    pub fn is_empty(&self) -> bool {
        let Self {
            restart,
            restart_sec,
        } = self;

        restart.is_none() && restart_sec.is_none()
    }
}

impl From<RestartConfig> for Service {
    fn from(restart: RestartConfig) -> Self {
        Self {
            restart: Some(restart),
            ..Self::default()
        }
    }
}

impl From<Restart> for Service {
    fn from(restart: Restart) -> Self {
        RestartConfig::from(restart).into()
    }
}

impl TryFrom<RestartPolicy> for Service {
    type Error = color_eyre::Report;

    fn try_from(
        RestartPolicy {
            condition,
            delay,
            max_attempts,
            window,
            extensions,
        }: RestartPolicy,
    ) -> Result<Self, Self::Error> {
        ensure!(
            max_attempts.is_none(),
            "`deploy.restart_policy.max_attempts` is not supported"
        );
        ensure!(
            window.is_none(),
            "`deploy.restart_policy.window` is not supported"
        );
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        Ok(Self {
            restart: Some(condition.unwrap_or_default().into()),
            restart_sec: Some(duration::to_string(delay.unwrap_or_default())),
        })
    }
}

/// Possible service restart configurations.
///
/// From [systemd.service](https://www.freedesktop.org/software/systemd/man/systemd.service.html#Restart=).
#[derive(ValueEnum, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RestartConfig {
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

impl From<RestartCondition> for RestartConfig {
    fn from(value: RestartCondition) -> Self {
        match value {
            RestartCondition::None => Self::No,
            RestartCondition::OnFailure => Self::OnFailure,
            RestartCondition::Any => Self::Always,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{Command, FromArgMatches};

    use super::*;

    #[test]
    fn restart_sec_cli_and_serialization() -> color_eyre::Result<()> {
        let matches = Service::augment_args(Command::new("test")).try_get_matches_from([
            "test",
            "--restart-sec",
            "5s",
        ])?;
        let service = Service::from_arg_matches(&matches)?;

        assert!(!service.is_empty());
        assert_eq!(
            crate::serde::quadlet::to_string_join_all(service)?,
            "[Service]\nRestartSec=5s\n"
        );
        Ok(())
    }
}
