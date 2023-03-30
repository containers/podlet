use std::fmt::Display;

use clap::{Args, Subcommand};

use super::container::{user_namespace, Output};

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Kube {
    /// Generate a podman quadlet `.kube` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html
    /// and https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#kube-units-kube
    #[group(skip)]
    Play {
        #[command(flatten)]
        play: Play,
    },
}

impl Display for Kube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Play { play } = self;
        writeln!(f, "[Kube]")?;
        write!(f, "{play}")
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
    configmap: Vec<String>,

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
    /// Converts to "RemapUsers=MODE"
    /// and potentially "RemapUid=UID" and "RemapGid=GID"
    #[arg(long, value_name = "MODE")]
    userns: Option<user_namespace::Mode>,

    /// The path to the Kubernetes YAML file to use
    ///
    /// Converts to "Yaml=FILE"
    file: String,
}

impl Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Yaml={}", self.file)?;

        for configmap in &self.configmap {
            writeln!(f, "ConfigMap={configmap}")?;
        }

        if let Some(log_driver) = &self.log_driver {
            writeln!(f, "LogDriver={log_driver}")?;
        }

        for network in &self.network {
            writeln!(f, "Network={network}")?;
        }

        for port in &self.publish {
            writeln!(f, "PublishPort={port}")?;
        }

        if let Some(Output::QuadletOptions(option)) = self.userns.as_ref().map(Output::from) {
            writeln!(f, "{option}")?;
        }

        Ok(())
    }
}
