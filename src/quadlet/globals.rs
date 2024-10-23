use std::path::PathBuf;

use serde::Serialize;

use super::{Downgrade, DowngradeError, HostPaths, PodmanVersion};

/// Global Quadlet options that apply to all resource types.
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Globals {
    /// Load the specified containers.conf module.
    pub containers_conf_module: Vec<PathBuf>,

    /// A list of arguments passed directly after `podman`.
    pub global_args: Option<String>,
}

impl Downgrade for Globals {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_8 {
            if let Some(containers_conf_module) =
                std::mem::take(&mut self.containers_conf_module).first()
            {
                return Err(DowngradeError::Option {
                    quadlet_option: "ContainersConfModule",
                    value: containers_conf_module.display().to_string(),
                    supported_version: PodmanVersion::V4_8,
                });
            }

            if let Some(global_args) = self.global_args.take() {
                return Err(DowngradeError::Option {
                    quadlet_option: "GlobalArgs",
                    value: global_args,
                    supported_version: PodmanVersion::V4_8,
                });
            }
        }

        Ok(())
    }
}

impl HostPaths for Globals {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.containers_conf_module.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() -> Result<(), crate::serde::quadlet::Error> {
        let globals = Globals::default();
        assert_eq!(crate::serde::quadlet::to_string(globals)?, "[Globals]\n");
        Ok(())
    }
}
