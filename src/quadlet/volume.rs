use std::{ops::Not, path::PathBuf};

use color_eyre::eyre::{Context, ensure};
use serde::Serialize;

use crate::{cli::volume::Opt, serde::quadlet::seq_quote_whitespace};

use super::{Downgrade, DowngradeError, HostPaths, PodmanVersion, push_arg};

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
    #[serde(serialize_with = "seq_quote_whitespace")]
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

    /// Override the name of the Podman volume created by this Quadlet.
    #[expect(clippy::struct_field_names, reason = "Quadlet option")]
    pub volume_name: Option<String>,
}

impl HostPaths for Volume {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.device.iter_mut()
    }
}

impl Volume {
    /// Add `--{flag} {arg}` to `PodmanArgs=`.
    fn push_arg(&mut self, flag: &str, arg: &str) {
        let podman_args = self.podman_args.get_or_insert_default();
        push_arg(podman_args, flag, arg);
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

        if version < PodmanVersion::V4_7 {
            if let Some(volume_name) = self.volume_name.take() {
                return Err(DowngradeError::Option {
                    quadlet_option: "VolumeName",
                    value: volume_name,
                    supported_version: PodmanVersion::V4_7,
                });
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
            name: volume_name,
            extensions,
        }: compose_spec::Volume,
    ) -> Result<Self, Self::Error> {
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
            volume_name,
            ..options.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::eyre::eyre;

    use super::*;

    #[test]
    fn volume_default_empty() -> Result<(), crate::serde::quadlet::Error> {
        let volume = Volume::default();
        assert_eq!(
            crate::serde::quadlet::to_string_join_all(volume)?,
            "[Volume]\n"
        );
        Ok(())
    }

    #[test]
    fn compose_volume_name() -> color_eyre::Result<()> {
        let volume = compose_spec::Volume {
            name: Some("explicit-volume-name".into()),
            ..compose_spec::Volume::default()
        };
        let volume = Volume::try_from(volume)?;

        assert_eq!(
            crate::serde::quadlet::to_string_join_all(volume)?,
            "[Volume]\nVolumeName=explicit-volume-name\n"
        );
        Ok(())
    }

    #[test]
    fn volume_name_added_in_v4_7() -> color_eyre::Result<()> {
        let mut volume = Volume {
            volume_name: Some("explicit-volume-name".into()),
            ..Volume::default()
        };

        let error = volume
            .downgrade(PodmanVersion::V4_6)
            .err()
            .ok_or_else(|| eyre!("expected VolumeName to be rejected before Podman v4.7"))?;

        assert_eq!(
            error.to_string(),
            "Quadlet option `VolumeName=explicit-volume-name` was not supported until Podman v4.7"
        );

        let mut volume = Volume {
            volume_name: Some("explicit-volume-name".into()),
            ..Volume::default()
        };

        volume.downgrade(PodmanVersion::V4_7)?;
        assert_eq!(volume.volume_name.as_deref(), Some("explicit-volume-name"));
        Ok(())
    }
}
