use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use serde::Serialize;

use super::{DowngradeError, PodmanVersion};

/// Global quadlet options that apply to all resource types.
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Globals {
    /// Load the specified containers.conf module.
    pub containers_conf_module: Vec<PathBuf>,

    /// A list of arguments passed directly after `podman`.
    pub global_args: Option<String>,
}

impl Globals {
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
            if let Some(containers_conf_module) =
                std::mem::take(&mut self.containers_conf_module).first()
            {
                return Err(DowngradeError::Option {
                    quadlet_option: String::from("ContainersConfModule"),
                    value: containers_conf_module.display().to_string(),
                    supported_version: PodmanVersion::V4_8,
                });
            }

            if let Some(global_args) = self.global_args.take() {
                return Err(DowngradeError::Option {
                    quadlet_option: String::from("GlobalArgs"),
                    value: global_args,
                    supported_version: PodmanVersion::V4_8,
                });
            }
        }

        Ok(())
    }
}

impl Display for Globals {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let globals =
            crate::serde::quadlet::to_string_no_table_name(self).map_err(|_| fmt::Error)?;
        f.write_str(&globals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_display_empty() {
        let globals = Globals::default();
        assert!(globals.to_string().is_empty(), "globals: {globals}");
    }
}
