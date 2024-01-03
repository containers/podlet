use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use serde::{Serialize, Serializer};

use super::{DowngradeError, PodmanVersion};

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Kube {
    /// Indicates whether containers will be auto-updated.
    pub auto_update: Vec<AutoUpdate>,

    /// Pass the Kubernetes ConfigMap YAML at path to `podman kube play`.
    pub config_map: Vec<PathBuf>,

    /// Set the log-driver Podman uses when running the container.
    pub log_driver: Option<String>,

    /// Specify a custom network for the container.
    pub network: Vec<String>,

    /// This key contains a list of arguments passed directly to the end of the `podman kube play`
    /// command in the generated file, right before the path to the yaml file in the command line.
    pub podman_args: Option<String>,

    /// Exposes a port, or a range of ports, from the container to the host.
    pub publish_port: Vec<String>,

    /// Set the user namespace mode for the container.
    #[serde(rename = "UserNS")]
    pub user_ns: Option<String>,

    /// The path, absolute or relative to the location of the unit file,
    /// to the Kubernetes YAML file to use.
    pub yaml: String,
}

impl Kube {
    pub fn new(yaml: String) -> Self {
        Self {
            auto_update: Vec::new(),
            config_map: Vec::new(),
            log_driver: None,
            network: Vec::new(),
            podman_args: None,
            publish_port: Vec::new(),
            user_ns: None,
            yaml,
        }
    }

    /// Downgrade compatibility to `version`.
    ///
    /// This is a one-way transformation, calling downgrade a second time with a higher version
    /// will not increase the quadlet options used.
    ///
    /// # Errors
    ///
    /// Returns an error if a used quadlet option is incompatible with the given [`PodmanVersion`].
    pub fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_7 {
            for auto_update in std::mem::take(&mut self.auto_update) {
                self.push_arg("annotation", &auto_update.to_annotation());
            }
        }

        if version < PodmanVersion::V4_6 {
            if let Some(podman_args) = self.podman_args.take() {
                return Err(DowngradeError {
                    quadlet_option: String::from("PodmanArgs"),
                    value: podman_args,
                    supported_version: PodmanVersion::V4_6,
                });
            }
        }

        if version < PodmanVersion::V4_5 {
            if let Some(log_driver) = self.log_driver.take() {
                return Err(DowngradeError {
                    quadlet_option: String::from("LogDriver"),
                    value: log_driver,
                    supported_version: PodmanVersion::V4_5,
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

impl Display for Kube {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let kube = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&kube)
    }
}

/// Valid values for the `AutoUpdate=` kube quadlet option.
#[derive(Debug, Clone, PartialEq)]
pub enum AutoUpdate {
    All(super::AutoUpdate),
    Container {
        container: String,
        auto_update: super::AutoUpdate,
    },
}

impl AutoUpdate {
    /// Podman-specific annotation for `podman auto-update`.
    ///
    /// See <https://docs.podman.io/en/stable/markdown/podman-auto-update.1.html>
    const ANNOTATION_KEY: &'static str = super::AutoUpdate::LABEL_KEY;

    /// Extracts all valid values of the `io.containers.autoupdate` annotation from `annotations`,
    /// the last value of which is parsed into an [`AutoUpdate`].
    ///
    /// Returns an empty `Vec` if no valid `io.containers.autoupdate` annotation is found.
    ///
    /// `io.containers.autoupdate` annotations with invalid values are retained in `annotations`.
    pub fn extract_from_annotations(annotations: &mut Vec<String>) -> Vec<Self> {
        let mut auto_updates = Vec::new();
        annotations.retain(|annotation| {
            // auto-update annotations are in the form `io.containers.autoupdate=[registry|local]`
            // or `io.containers.autoupdate/$container=[registry|local]`
            // see https://docs.podman.io/en/stable/markdown/podman-auto-update.1.html#auto-updates-and-kubernetes-yaml
            annotation
                .strip_prefix(Self::ANNOTATION_KEY)
                .and_then(|auto_update| {
                    let (container, auto_update) = auto_update.split_once('=')?;
                    let auto_update = auto_update.parse().ok()?;
                    container
                        .strip_prefix('/')
                        .map(|container| Self::Container {
                            container: container.to_owned(),
                            auto_update,
                        })
                        .or_else(|| container.is_empty().then_some(Self::All(auto_update)))
                })
                .map_or(true, |auto_update| {
                    auto_updates.push(auto_update);
                    false
                })
        });

        auto_updates
    }

    fn to_annotation(&self) -> String {
        match self {
            Self::All(auto_update) => format!("{}={auto_update}", Self::ANNOTATION_KEY),
            Self::Container {
                container,
                auto_update,
            } => format!("{}/{container}={auto_update}", Self::ANNOTATION_KEY),
        }
    }
}

impl Serialize for AutoUpdate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::All(auto_update) => auto_update.serialize(serializer),
            Self::Container {
                container,
                auto_update,
            } => format!("{container}/{auto_update}").serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kube_default_empty() {
        let kube = Kube::new(String::from("yaml"));
        assert_eq!(kube.to_string(), "[Kube]\nYaml=yaml\n");
    }

    #[test]
    fn auto_update_extract() {
        let mut annotations = vec![
            String::from("annotation"),
            String::from("io.containers.autoupdate=invalid"),
            String::from("io.containers.autoupdate#invalid=registry"),
            String::from("io.containers.autoupdate=registry"),
            String::from("io.containers.autoupdate/container=local"),
        ];

        let auto_updates = AutoUpdate::extract_from_annotations(&mut annotations);

        assert_eq!(
            auto_updates,
            [
                AutoUpdate::All(crate::quadlet::AutoUpdate::Registry),
                AutoUpdate::Container {
                    container: String::from("container"),
                    auto_update: crate::quadlet::AutoUpdate::Local
                }
            ]
        );
        assert_eq!(
            annotations,
            [
                "annotation",
                "io.containers.autoupdate=invalid",
                "io.containers.autoupdate#invalid=registry"
            ]
        );
    }
}
