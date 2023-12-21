use std::{
    convert::Infallible,
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    net::IpAddr,
    ops::Not,
    path::PathBuf,
    str::FromStr,
};

use clap::{Args, Subcommand};
use serde::Serialize;
use url::Url;

use crate::quadlet::KubeAutoUpdate;

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Kube {
    /// Generate a podman quadlet `.kube` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html and
    /// https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#kube-units-kube
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
    file: File,
}

impl From<Play> for crate::quadlet::Kube {
    fn from(mut value: Play) -> Self {
        let auto_update =
            KubeAutoUpdate::extract_from_annotations(&mut value.podman_args.annotation);
        let podman_args = value.podman_args.to_string();
        Self {
            auto_update,
            config_map: value.configmap,
            log_driver: value.log_driver,
            network: value.network,
            podman_args: (!podman_args.is_empty()).then_some(podman_args),
            publish_port: value.publish,
            user_ns: value.userns,
            yaml: value.file.to_string(),
        }
    }
}

#[derive(Args, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PodmanArgs {
    /// Add an annotation to the container or pod
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    annotation: Vec<String>,

    /// Build images even if they are found in the local storage
    ///
    /// Use `--build=false` to completely disable builds
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<bool>,

    /// Use certificates at `path` (*.crt, *.cert, *.key) to connect to the registry
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cert_dir: Option<PathBuf>,

    /// Use `path` as the build context directory for each image
    #[arg(long, requires = "build", value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    context_dir: Option<PathBuf>,

    /// The username and password to use to authenticate with the registry, if required
    #[arg(long, value_name = "USERNAME[:PASSWORD]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    creds: Option<String>,

    /// Assign a static ip address to the pod
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ip: Vec<IpAddr>,

    /// Logging driver specific options
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "NAME=VALUE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    log_opt: Vec<String>,

    /// Assign a static mac address to the pod
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    mac_address: Vec<String>,

    /// Do not create `/etc/hosts` for the pod
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_hosts: bool,

    /// Directory path for seccomp profiles
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    seccomp_profile_root: Option<PathBuf>,

    /// Require HTTPS and verify certificates when contacting registries
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tls_verify: Option<bool>,
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum File {
    Url(Url),
    Path(PathBuf),
}

impl FromStr for File {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse()
            .map_or_else(|_| Self::Path(PathBuf::from(s)), Self::Url))
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Url(url) => write!(f, "{url}"),
            Self::Path(path) => write!(f, "{}", path.display()),
        }
    }
}

impl File {
    /// Return the name of the kube file (without the extension)
    fn name(&self) -> Option<&str> {
        match self {
            Self::Url(url) => url
                .path_segments()
                .and_then(Iterator::last)
                .filter(|file| !file.is_empty())
                .and_then(|file| file.split('.').next()),
            Self::Path(path) => path.file_stem().and_then(OsStr::to_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_file_name() {
        let sut = File::Url(Url::parse("https://example.com/test.yaml").expect("valid url"));
        assert_eq!(sut.name(), Some("test"));
    }

    #[test]
    fn path_file_name() {
        let sut = File::Path(PathBuf::from("test.yaml"));
        assert_eq!(sut.name(), Some("test"));
    }
}
