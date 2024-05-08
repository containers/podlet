use std::{
    fmt::{self, Display, Formatter},
    net::IpAddr,
    ops::Not,
    path::PathBuf,
};

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::quadlet::kube::{AutoUpdate, YamlFile};

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Kube {
    /// Generate a Podman Quadlet `.kube` file,
    ///
    /// Only options supported by Quadlet are present,
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-kube-play.1.html and
    /// https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html#kube-units-kube
    #[allow(clippy::doc_markdown)]
    #[group(skip)]
    Play {
        #[command(flatten)]
        play: Play,
    },
}

impl From<Kube> for crate::quadlet::Kube {
    fn from(value: Kube) -> Self {
        let Kube::Play { play } = value;
        play.into()
    }
}

impl From<Kube> for crate::quadlet::Resource {
    fn from(value: Kube) -> Self {
        crate::quadlet::Kube::from(value).into()
    }
}

impl Kube {
    pub fn name(&self) -> &str {
        let Kube::Play { play } = self;

        play.file.name().unwrap_or("pod")
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Play {
    /// The path to a Kubernetes YAML file containing a configmap
    ///
    /// Converts to "ConfigMap=PATH"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH", value_delimiter = ',')]
    configmap: Vec<PathBuf>,

    /// Set logging driver for the pod
    ///
    /// Converts to "LogDriver=DRIVER"
    #[arg(long, value_name = "DRIVER")]
    log_driver: Option<String>,

    /// Specify a custom network for the pod
    ///
    /// Converts to "Network=MODE"
    ///
    /// Can be specified multiple times
    #[arg(long, visible_alias = "net", value_name = "MODE")]
    network: Vec<String>,

    /// Define or override a port definition in the YAML file
    ///
    /// Converts to "PublishPort=PORT"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "[[IP:][HOST_PORT]:]CONTAINER_PORT[/PROTOCOL]")]
    publish: Vec<String>,

    /// Set the user namespace mode for the pod
    ///
    /// Converts to "UserNS=MODE"
    #[arg(long, value_name = "MODE")]
    userns: Option<String>,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: PodmanArgs,

    /// The path to the Kubernetes YAML file to use
    ///
    /// Converts to "Yaml=FILE"
    file: YamlFile,
}

impl From<Play> for crate::quadlet::Kube {
    fn from(mut value: Play) -> Self {
        let auto_update = AutoUpdate::extract_from_annotations(&mut value.podman_args.annotation);
        let podman_args = value.podman_args.to_string();
        Self {
            auto_update,
            config_map: value.configmap,
            log_driver: value.log_driver,
            network: value.network,
            podman_args: (!podman_args.is_empty()).then_some(podman_args),
            publish_port: value.publish,
            user_ns: value.userns,
            yaml: value.file,
        }
    }
}

#[derive(Args, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PodmanArgs {
    /// Add an annotation to the container or pod
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    annotation: Vec<String>,

    /// Build images even if they are found in the local storage
    ///
    /// Use `--build=false` to completely disable builds
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    build: Option<bool>,

    /// Use certificates at `path` (*.crt, *.cert, *.key) to connect to the registry
    #[arg(long, value_name = "PATH")]
    cert_dir: Option<PathBuf>,

    /// Use `path` as the build context directory for each image
    #[arg(long, requires = "build", value_name = "PATH")]
    context_dir: Option<PathBuf>,

    /// The username and password to use to authenticate with the registry, if required
    #[arg(long, value_name = "USERNAME[:PASSWORD]")]
    creds: Option<String>,

    /// Assign a static ip address to the pod
    ///
    /// Can be specified multiple times
    #[arg(long)]
    ip: Vec<IpAddr>,

    /// Logging driver specific options
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "NAME=VALUE")]
    log_opt: Vec<String>,

    /// Assign a static mac address to the pod
    ///
    /// Can be specified multiple times
    #[arg(long)]
    mac_address: Vec<String>,

    /// Do not create `/etc/hosts` for the pod
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_hosts: bool,

    /// Directory path for seccomp profiles
    #[arg(long, value_name = "PATH")]
    seccomp_profile_root: Option<PathBuf>,

    /// Require HTTPS and verify certificates when contacting registries
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    tls_verify: Option<bool>,
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn podman_args_default_display_empty() {
        let args = PodmanArgs::default();
        assert!(args.to_string().is_empty());
    }
}
