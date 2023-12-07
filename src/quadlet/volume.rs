use std::fmt::{self, Display, Formatter};

use color_eyre::eyre::{self, Context};

use crate::cli::volume::opt::Opt;

use super::writeln_escape_spaces;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Volume {
    pub copy: bool,
    pub device: Option<String>,
    pub group: Option<String>,
    pub label: Vec<String>,
    pub options: Option<String>,
    pub podman_args: Option<String>,
    pub fs_type: Option<String>,
    pub user: Option<String>,
}

impl TryFrom<docker_compose_types::ComposeVolume> for Volume {
    type Error = color_eyre::Report;

    fn try_from(value: docker_compose_types::ComposeVolume) -> Result<Self, Self::Error> {
        let unsupported_options = [
            ("driver", value.driver.is_none()),
            ("external", value.external.is_none()),
            ("name", value.name.is_none()),
        ];
        for (option, not_present) in unsupported_options {
            eyre::ensure!(not_present, "`{option}` is not supported");
        }

        let options: Vec<Opt> = value
            .driver_opts
            .into_iter()
            .map(|(key, value)| {
                let driver_opt = key.clone();
                match value {
                    Some(value) if key != "copy" => format!("{key}={value}"),
                    _ => key,
                }
                .parse()
                .wrap_err_with(|| {
                    format!("driver_opt `{driver_opt}` is not a valid podman volume driver option")
                })
            })
            .collect::<Result<_, _>>()?;

        let label = match value.labels {
            docker_compose_types::Labels::List(labels) => labels,
            docker_compose_types::Labels::Map(labels) => labels
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        };

        Ok(Self {
            label,
            ..options.into()
        })
    }
}

impl Display for Volume {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Volume]")?;

        if self.copy {
            writeln!(f, "Copy=true")?;
        }

        if let Some(device) = &self.device {
            writeln!(f, "Device={device}")?;
        }

        if let Some(group) = &self.group {
            writeln!(f, "Group={group}")?;
        }

        if !self.label.is_empty() {
            writeln_escape_spaces::<' ', _>(f, "Label", &self.label)?;
        }

        if let Some(options) = &self.options {
            writeln!(f, "Options={options}")?;
        }

        if let Some(podman_args) = &self.podman_args {
            writeln!(f, "PodmanArgs={podman_args}")?;
        }

        if let Some(fs_type) = &self.fs_type {
            writeln!(f, "Type={fs_type}")?;
        }

        if let Some(user) = &self.user {
            writeln!(f, "User={user}")?;
        }

        Ok(())
    }
}
