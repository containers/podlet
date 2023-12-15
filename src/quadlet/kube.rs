use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Kube {
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
            config_map: Vec::new(),
            log_driver: None,
            network: Vec::new(),
            podman_args: None,
            publish_port: Vec::new(),
            user_ns: None,
            yaml,
        }
    }
}

impl Display for Kube {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let kube = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&kube)
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
}
