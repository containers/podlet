use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use serde::Serialize;

use super::{container::Volume, Downgrade, DowngradeError, HostPaths, PodmanVersion, ResourceKind};

/// Options for the \[Pod\] section of a `.pod` quadlet file.
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Pod {
    /// Specify a custom network for the pod.
    pub network: Vec<String>,

    /// A list of arguments passed directly to the end of the `podman pod create` command in the
    /// generated file.
    pub podman_args: Option<String>,

    /// The name of the Podman pod.
    ///
    /// If not set, the default value is `systemd-%N`.
    #[allow(clippy::struct_field_names)]
    pub pod_name: Option<String>,

    /// Exposes a port, or a range of ports, from the pod to the host.
    pub publish_port: Vec<String>,

    /// Mount a volume in the pod.
    pub volume: Vec<Volume>,
}

impl Display for Pod {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let pod = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&pod)
    }
}

impl HostPaths for Pod {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.volume.iter_mut().flat_map(Volume::host_paths)
    }
}

impl Downgrade for Pod {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V5_0 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Pod,
                supported_version: PodmanVersion::V5_0,
            });
        }

        Ok(())
    }
}
