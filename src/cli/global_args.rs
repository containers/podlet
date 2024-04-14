use std::{mem, ops::Not, path::PathBuf};

use clap::{Args, ValueEnum};
use serde::Serialize;

use crate::quadlet::Globals;

/// Podman global options
///
/// Converted into [`Globals`] for inclusion in [`crate::quadlet::File`]s.
/// All options, except `module`, are serialized with the args serializer.
#[derive(Args, Serialize, Debug, Default, Clone, PartialEq)]
#[command(next_help_heading = "Podman Global Options")]
#[serde(rename_all = "kebab-case")]
pub struct GlobalArgs {
    /// Cgroup manager to use
    #[arg(long, global = true, value_name = "MANAGER")]
    cgroup_manager: Option<CGroupManager>,

    /// Path of the conmon binary
    #[arg(long, global = true, value_name = "PATH")]
    conmon: Option<PathBuf>,

    /// Connection to use for remote Podman service
    #[arg(long, global = true, value_name = "CONNECTION_URI")]
    connection: Option<String>,

    /// Backend to use for storing events
    #[arg(long, global = true, value_name = "TYPE")]
    events_backend: Option<EventsBackend>,

    /// Set the OCI hooks directory path
    ///
    /// Can be specified multiple times
    #[arg(long, global = true, value_name = "PATH")]
    hooks_dir: Vec<PathBuf>,

    /// Path to ssh identity file
    #[arg(long, global = true, value_name = "PATH")]
    identity: Option<PathBuf>,

    /// Path to the 'image store'
    ///
    /// Different from 'graph root'
    ///
    /// Use this to split storing the image into a separate 'image store',
    /// see 'man containers-storage.conf' for details.
    #[arg(long, global = true, value_name = "PATH")]
    imagestore: Option<PathBuf>,

    /// Log messages at and above specified level
    #[arg(long, global = true, value_name = "LEVEL", default_value = "warn")]
    #[serde(skip_serializing_if = "LogLevel::is_warn")]
    log_level: LogLevel,

    /// Load the specified `containers.conf(5)` module
    ///
    /// Converts to "ContainersConfModule=PATH"
    ///
    /// Can be specified multiple times
    #[arg(long, global = true, value_name = "PATH")]
    #[serde(skip_serializing)]
    module: Vec<PathBuf>,

    /// Path to the `slirp4netns(1)` command binary
    ///
    /// Note: This option is deprecated and will be removed with Podman 5.0.
    /// Use the helper_binaries_dir option in containers.conf instead.
    #[arg(long, global = true, value_name = "PATH")]
    network_cmd_path: Option<PathBuf>,

    /// Path of the configuration directory for networks
    #[arg(long, global = true, value_name = "DIRECTORY")]
    network_config_dir: Option<PathBuf>,

    /// Redirect the output of podman to a file without affecting the container output or its logs
    #[arg(long, global = true, value_name = "PATH")]
    out: Option<PathBuf>,

    /// Access remote Podman service
    #[arg(
        short,
        long,
        global = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    remote: Option<bool>,

    /// Path to the graph root directory where images, containers, etc. are stored
    #[arg(long, global = true, value_name = "VALUE")]
    root: Option<PathBuf>,

    /// Storage state directory where all state information is stored
    #[arg(long, global = true, value_name = "VALUE")]
    runroot: Option<PathBuf>,

    /// Path to the OCI-compatible binary used to run containers
    #[arg(long, global = true, value_name = "VALUE")]
    runtime: Option<PathBuf>,

    /// Add global flags for the container runtime
    ///
    /// Can be specified multiple times
    #[arg(long, global = true, value_name = "FLAG")]
    runtime_flag: Vec<String>,

    /// Define the ssh mode
    #[arg(long, global = true, value_name = "VALUE")]
    ssh: Option<SshMode>,

    /// Select which storage driver is used to manage storage of images and containers
    #[arg(long, global = true, value_name = "VALUE")]
    storage_driver: Option<String>,

    /// Specify a storage driver option
    ///
    /// Can be specified multiple times
    #[arg(long, global = true, value_name = "VALUE")]
    storage_opt: Vec<String>,

    /// Output logging information to syslog as well as the console
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Not::not")]
    syslog: bool,

    /// Path to the tmp directory for libpod state content
    #[arg(long, global = true, value_name = "PATH")]
    tmpdir: Option<PathBuf>,

    /// Enable transient container storage
    #[arg(
        long,
        global = true,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
    )]
    transient_store: Option<bool>,

    /// URL to access Podman service
    #[arg(long, global = true, value_name = "VALUE")]
    url: Option<String>,

    /// Volume directory where builtin volume information is stored
    #[arg(long, global = true, value_name = "VALUE")]
    volumepath: Option<PathBuf>,
}

impl GlobalArgs {
    /// Consruct [`GlobalArgs`] by taking fields from a [`compose_spec::Service`].
    ///
    /// Takes the `runtime` and `storage_opt` fields.
    pub fn from_compose(service: &mut compose_spec::Service) -> Self {
        Self {
            runtime: service.runtime.take().map(Into::into),
            storage_opt: mem::take(&mut service.storage_opt)
                .into_iter()
                .map(|(key, value)| {
                    let mut opt = String::from(key);
                    opt.push('=');
                    if let Some(value) = value {
                        opt.push_str(&String::from(value));
                    }
                    opt
                })
                .collect(),
            ..Self::default()
        }
    }
}

impl From<GlobalArgs> for Globals {
    fn from(value: GlobalArgs) -> Self {
        let global_args =
            crate::serde::args::to_string(&value).expect("GlobalArgs serializes to args");
        Self {
            containers_conf_module: value.module,
            global_args: (!global_args.is_empty()).then_some(global_args),
        }
    }
}

/// Valid values for `podman --cgroup-manager`
///
/// See <https://docs.podman.io/en/stable/markdown/podman.1.html#cgroup-manager-manager>
#[derive(ValueEnum, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lower")]
#[serde(rename_all = "lowercase")]
enum CGroupManager {
    CGroupFs,
    Systemd,
}

/// Valid values for `podman --events-backend`
///
/// See <https://docs.podman.io/en/stable/markdown/podman.1.html#events-backend-type>
#[derive(ValueEnum, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lower")]
#[serde(rename_all = "lowercase")]
enum EventsBackend {
    File,
    Journald,
    None,
}

/// Valid values for `podman --log-level`
///
/// See <https://docs.podman.io/en/stable/markdown/podman.1.html#log-level-level>
#[derive(ValueEnum, Serialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lower")]
#[serde(rename_all = "lowercase")]
enum LogLevel {
    Debug,
    Info,
    #[default]
    Warn,
    Error,
    Fatal,
    Panic,
}

impl LogLevel {
    /// Returns `true` if the log level is [`Warn`].
    ///
    /// [`Warn`]: LogLevel::Warn
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    fn is_warn(&self) -> bool {
        matches!(self, Self::Warn)
    }
}

/// Valid values for `podman --ssh`
///
/// See <https://docs.podman.io/en/stable/markdown/podman.1.html#ssh-value>
#[derive(ValueEnum, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lower")]
#[serde(rename_all = "lowercase")]
enum SshMode {
    GoLang,
    Native,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn default_args_serialize_empty() {
        let global_args = crate::serde::args::to_string(GlobalArgs::default()).unwrap();
        assert!(global_args.is_empty());
    }
}
