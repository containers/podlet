use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
    path::PathBuf,
};

use color_eyre::eyre::{ensure, Context};
use serde::Serialize;

use crate::{cli::volume::Opt, serde::quadlet::quote_spaces_join_space};

use super::{Downgrade, DowngradeError, HostPaths, PodmanVersion};

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Volume {
    /// If enabled, the content of the image located at the mount point of the volume
    /// is copied into the volume on the first run.
    #[serde(skip_serializing_if = "Not::not")]
    pub copy: bool,

    /// The path of a device which is mounted for the volume.
    pub device: Option<PathBuf>,

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

impl HostPaths for Volume {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.device.iter_mut()
    }
}

impl Volume {
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

impl Downgrade for Volume {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
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
                    quadlet_option: "PodmanArgs",
                    value: podman_args,
                    supported_version: PodmanVersion::V4_6,
                });
            }
        }

        Ok(())
    }
}

impl TryFrom<compose_spec::Volume> for Volume {
    type Error = color_eyre::Report;

    fn try_from(
        compose_spec::Volume {
            driver,
            driver_opts,
            labels,
            name,
            extensions,
        }: compose_spec::Volume,
    ) -> Result<Self, Self::Error> {
        ensure!(name.is_none(), "`name` is not supported");
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        let options: Vec<Opt> = driver_opts
            .into_iter()
            .enumerate()
            .map(|(index, (option, value))| {
                let value = String::from(value);
                let value = (!value.is_empty()
                    && (option != "copy" || !matches!(value.as_str(), "true" | "1")))
                .then_some(value);
                Opt::parse(option.as_str(), value)
                    .wrap_err_with(|| format!("error converting `driver_opts[{index}]`"))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            driver,
            label: labels.into_list().into_iter().collect(),
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
