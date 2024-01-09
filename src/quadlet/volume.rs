use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
};

use color_eyre::eyre::{self, Context};
use serde::Serialize;

use crate::{cli::volume::opt::Opt, serde::quadlet::quote_spaces_join_space};

use super::{DowngradeError, PodmanVersion};

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Volume {
    /// If enabled, the content of the image located at the mount point of the volume
    /// is copied into the volume on the first run.
    #[serde(skip_serializing_if = "Not::not")]
    pub copy: bool,

    /// The path of a device which is mounted for the volume.
    pub device: Option<String>,

    /// Specify the volume driver name.
    pub driver: Option<String>,

    /// The host (numeric) GID, or group name to use as the group for the volume.
    pub group: Option<String>,

    /// Specifies the image the volume is based on when `Driver` is set to the `image`.
    pub image: Option<String>,

    /// Set one or more OCI labels on the volume.
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub label: Vec<String>,

    /// The mount options to use for a filesystem as used by the `mount` command -o option.
    pub options: Option<String>,

    /// This key contains a list of arguments passed directly to the end of the `podman volume create`
    /// command in the generated file, right before the name of the network in the command line.
    pub podman_args: Option<String>,

    /// The filesystem type of `Device` as used by the `mount` commands `-t` option.
    #[serde(rename = "Type")]
    pub fs_type: Option<String>,

    /// The host (numeric) UID, or user name to use as the owner for the volume.
    pub user: Option<String>,
}

impl Volume {
    /// Downgrade compatibility to `version`.
    ///
    /// This is a one-way transformation, calling downgrade a second time with a higher version
    /// will not increase the quadlet options used.
    ///
    /// # Errors
    ///
    /// Returns an error if a used quadlet option is incompatible with the given [`PodmanVersion`].
    pub fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_8 {
            if let Some(driver) = self.driver.take() {
                self.push_arg("driver", &driver);
            }

            if let Some(image) = self.image.take() {
                self.push_arg("opt", &format!("image={image}"));
            }
        }

        if version < PodmanVersion::V4_6 {
            if let Some(podman_args) = self.podman_args.take() {
                return Err(DowngradeError::Option {
                    quadlet_option: String::from("PodmanArgs"),
                    value: podman_args,
                    supported_version: PodmanVersion::V4_6,
                });
            }
        }

        Ok(())
    }

    /// Add `--{flag} {arg}` to `PodmanArgs=`.
    fn push_arg(&mut self, flag: &str, arg: &str) {
        let podman_args = self.podman_args.get_or_insert_with(String::new);
        if !podman_args.is_empty() {
            podman_args.push(' ');
        }
        podman_args.push_str("--");
        podman_args.push_str(flag);
        podman_args.push(' ');
        podman_args.push_str(arg);
    }
}

impl TryFrom<docker_compose_types::ComposeVolume> for Volume {
    type Error = color_eyre::Report;

    fn try_from(value: docker_compose_types::ComposeVolume) -> Result<Self, Self::Error> {
        let unsupported_options = [
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
            driver: value.driver,
            label,
            ..options.into()
        })
    }
}

impl Display for Volume {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let volume = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&volume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_default_empty() {
        let volume = Volume::default();
        assert_eq!(volume.to_string(), "[Volume]\n");
    }
}
