use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
    path::PathBuf,
    time::Duration,
};

use clap::{builder::TypedValueParser, ArgAction, Args};
use color_eyre::{
    eyre::{eyre, Context},
    owo_colors::OwoColorize,
    Section,
};
use compose_spec::service::{
    blkio_config::{BpsLimit, IopsLimit, Weight, WeightDevice},
    BlkioConfig, Ipc,
};
use serde::Serialize;
use smart_default::SmartDefault;

use crate::serde::skip_true;

use super::compose;

#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Args, Serialize, SmartDefault, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PodmanArgs {
    /// Add a custom host-to-IP mapping
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "HOST:IP")]
    add_host: Vec<String>,

    /// Override the architecture of the image to be pulled
    ///
    /// Defaults to hosts architecture
    #[arg(long)]
    arch: Option<String>,

    /// Attach to STDIN, STDOUT, or STDERR
    #[arg(short, long, value_name = "STDIN | STDOUT | STDERR")]
    attach: Vec<String>,

    /// Path of the authentication file
    ///
    /// Default is `${XDG_RUNTIME_DIR}/containers/auth.json`
    #[arg(long, value_name = "PATH")]
    authfile: Option<PathBuf>,

    /// Block IO relative weight, between 10 and 1000
    #[arg(long, value_name = "WEIGHT", value_parser = blkio_weight_parser())]
    blkio_weight: Option<Weight>,

    /// Block IO relative device weight
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "DEVICE:WEIGHT")]
    blkio_weight_device: Vec<String>,

    /// Specify the cgroup file to write to and its value
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    cgroup_conf: Vec<String>,

    /// Path to cgroups under which the cgroup for the container will be created
    #[arg(long, value_name = "PATH")]
    cgroup_parent: Option<PathBuf>,

    /// Set the cgroup namespace for the container
    #[arg(long, value_name = "MODE")]
    cgroupns: Option<String>,

    /// Whether the container will create cgroups
    #[arg(long, value_name = "HOW")]
    cgroups: Option<String>,

    /// Chroot directories inside the container
    #[arg(long, value_name = "PATH")]
    chrootdirs: Option<String>,

    /// Write container ID to a file
    #[arg(long, value_name = "FILE")]
    cidfile: Option<PathBuf>,

    /// Write the pid of the conmon process to a file
    #[arg(long, value_name = "FILE")]
    conmon_pidfile: Option<PathBuf>,

    /// Limit the CPU CFS (Completely Fair Scheduler) period
    #[arg(long, value_name = "LIMIT")]
    cpu_period: Option<u128>,

    /// Limit the CPU CFS (Completely Fair Scheduler) quota
    #[arg(long, value_name = "LIMIT")]
    cpu_quota: Option<u128>,

    /// Limit the CPU real-time period in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    cpu_rt_period: Option<u128>,

    /// Limit the CPU real-time runtime in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    cpu_rt_runtime: Option<u128>,

    /// CPU shares (relative weight)
    #[arg(short, long, value_name = "SHARES")]
    cpu_shares: Option<u64>,

    /// Number of CPUs
    #[arg(long, value_name = "NUMBER")]
    cpus: Option<f64>,

    /// CPUs in which to allow execution
    #[arg(long, value_name = "NUMBER")]
    cpuset_cpus: Option<String>,

    /// Memory nodes (MEMs) in which to allow execution
    #[arg(long, value_name = "NODES")]
    cpuset_mems: Option<String>,

    /// Key needed to decrypt the image
    #[arg(long, value_name = "KEY[:PASSPHRASE]")]
    decryption_key: Option<String>,

    /// Detached mode: run the container in the background
    ///
    /// Automatically set by quadlet
    #[arg(short, long)]
    #[serde(skip_serializing)]
    detach: bool,

    /// Key sequence for detaching a container
    #[arg(long, value_name = "SEQUENCE")]
    detach_keys: Option<String>,

    /// Add a rule to the cgroup allowed devices list
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "TYPE MAJOR:MINOR MODE")]
    device_cgroup_rule: Vec<String>,

    /// Limit read rate (in bytes per second) from a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    device_read_bps: Vec<String>,

    /// Limit read rate (in IO operations per second) from a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    device_read_iops: Vec<String>,

    /// Limit write rate (in bytes per second) to a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    device_write_bps: Vec<String>,

    /// Limit write rate (in IO operations per second) to a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    device_write_iops: Vec<String>,

    /// This is a Docker specific option and is a NOOP
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    disable_content_trust: bool,

    /// Preprocess default environment variables for the container
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "ENV")]
    env_merge: Vec<String>,

    /// Assign additional groups to the primary user running within the container process
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "GROUP")]
    group_add: Vec<String>,

    /// Customize the entry that is written to the /etc/group file within the container
    #[arg(long, value_name = "ENTRY")]
    group_entry: Option<String>,

    /// Add a user account to /etc/passwd from the host to the container
    #[arg(long, value_name = "NAME")]
    hostuser: Vec<String>,

    /// Set proxy environment variables in the container based on the host proxy vars
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    http_proxy: bool,

    /// How to handle the builtin image volumes
    #[arg(long, value_name = "bind | tmpfs | ignore")]
    image_volume: Option<String>,

    /// Path to the container-init binary
    #[arg(long, value_name = "PATH")]
    init_path: Option<PathBuf>,

    /// keep stdin open even if not attached
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    interactive: bool,

    /// Set the IPC namespace mode for the container
    #[arg(long)]
    ipc: Option<String>,

    /// Read in a line-delimited file of labels
    #[arg(long, value_name = "FILE")]
    label_file: Option<PathBuf>,

    /// Not implemented
    #[arg(long, value_name = "IP")]
    link_local_ip: Option<String>,

    /// Logging driver specific options
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "NAME=VALUE")]
    log_opt: Vec<String>,

    /// Container network interface MAC address
    #[arg(long, value_name = "ADDRESS")]
    mac_address: Option<String>,

    /// Memory limit
    #[arg(short, long, value_name = "NUMBER[UNIT]")]
    memory: Option<String>,

    /// Memory soft limit
    #[arg(long, value_name = "NUMBER[UNIT]")]
    memory_reservation: Option<String>,

    /// Limit value equal to memory plus swap
    #[arg(long, value_name = "NUMBER[UNIT]")]
    memory_swap: Option<String>,

    /// Tune the container’s memory swappiness behavior
    #[arg(long, value_name = "NUMBER")]
    memory_swappiness: Option<u8>,

    /// Add a network-scoped alias for the container
    #[arg(long, value_name = "ALIAS")]
    network_alias: Option<String>,

    /// Disable healthchecks on the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_healthcheck: bool,

    /// Do not create /etc/hosts for the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    no_hosts: bool,

    /// Disable OOM Killer for the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    oom_kill_disable: bool,

    /// Tune the host’s OOM preferences for the container
    #[arg(long, value_name = "NUM")]
    oom_score_adj: Option<i16>,

    /// Override the OS, defaults to hosts, of the image to be pulled
    #[arg(long)]
    os: Option<String>,

    /// Add entries to /etc/passwd and /etc/group when used with the --user option
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    passwd: bool,

    /// Entry to write to /etc/passwd
    #[arg(long, value_name = "ENTRY")]
    passwd_entry: Option<String>,

    /// Configure execution domain using personality
    #[arg(long, value_name = "PERSONA")]
    personality: Option<String>,

    /// Set the PID namespace mode for the container
    #[arg(long, value_name = "MODE")]
    pid: Option<String>,

    /// Write the container process ID to the file
    #[arg(long, value_name = "PATH")]
    pidfile: Option<PathBuf>,

    /// Specify the platform for selecting the image
    #[arg(long, value_name = "OS/ARCH")]
    platform: Option<String>,

    /// Run the container in an existing pod
    #[arg(long, value_name = "NAME")]
    pod: Option<String>,

    /// Read the pod ID from the file
    #[arg(long, value_name = "FILE")]
    pod_id_file: Option<PathBuf>,

    /// Pass down to the process additional file descriptors
    #[arg(long, value_name = "FD1[,FD2,…]")]
    preserve_fd: Option<String>,

    /// Pass a number of additional file descriptors into the container
    #[arg(long, value_name = "N")]
    preserve_fds: Option<u16>,

    /// Give extended privileges to the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    privileged: bool,

    /// Publish all exposed ports to random ports on the host interfaces
    #[arg(short = 'P', long)]
    #[serde(skip_serializing_if = "Not::not")]
    publish_all: bool,

    /// Suppress output information when pulling images
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    quiet: bool,

    /// If a container with the same name exists, replace it
    ///
    /// Automatically set by quadlet
    #[arg(long)]
    #[serde(skip_serializing)]
    replace: bool,

    /// Add one or more requirement containers
    #[arg(long, value_name = "CONTAINER[,...]")]
    requires: Option<String>,

    /// Remove container (and pod if created) after exit
    ///
    /// Automatically set by quadlet
    #[arg(long)]
    #[serde(skip_serializing)]
    rm: bool,

    /// After the container exits, remove the container image unless it is used by other containers
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    rmi: bool,

    /// Specify the policy to select the seccomp profile
    #[arg(long, value_name = "POLICY")]
    seccomp_policy: Option<String>,

    /// Size of systemd-specific tmpfs mounts: /run, /run/lock, /var/log/journal, and /tmp
    #[arg(long, value_name = "NUMBER[UNIT]")]
    shm_size_systemd: Option<String>,

    /// Proxy received signals to the container process
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    #[default = true]
    sig_proxy: bool,

    /// Signal to stop a container
    #[arg(long, value_name = "SIGNAL")]
    stop_signal: Option<String>,

    /// Run container in systemd mode
    ///
    /// Default is true
    #[arg(long, value_name = "true | false | always")]
    systemd: Option<String>,

    /// Maximum length of time a container is allowed to run
    #[arg(long, value_name = "SECONDS")]
    timeout: Option<u16>,

    /// Require HTTPS and verify certificates when contacting registries
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    tls_verify: Option<bool>,

    /// Allocate a pseudo-TTY
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    tty: bool,

    /// Set the umask inside the container
    #[arg(long)]
    umask: Option<String>,

    /// Set the UTS namespace mode for the container
    #[arg(long, value_name = "MODE")]
    uts: Option<String>,

    /// Set variant to use instead of the default architecture variant of the container image
    #[arg(long)]
    variant: Option<String>,

    /// Mount volumes from the specified container
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CONTAINER[:OPTIONS]")]
    volumes_from: Vec<String>,
}

/// Create a [`TypedValueParser`] for parsing the `blkio_weight` field of [`PodmanArgs`].
fn blkio_weight_parser() -> impl TypedValueParser<Value = Weight> {
    clap::value_parser!(u16)
        .range(10..=1000)
        .try_map(TryInto::try_into)
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}

impl TryFrom<compose::PodmanArgs> for PodmanArgs {
    type Error = color_eyre::Report;

    fn try_from(
        compose::PodmanArgs {
            blkio_config,
            cpu_shares,
            cpu_period,
            cpu_quota,
            cpu_rt_runtime,
            cpu_rt_period,
            cpus,
            cpuset,
            cgroup,
            cgroup_parent,
            device_cgroup_rules,
            extra_hosts,
            group_add,
            ipc,
            uts,
            log_options,
            mac_address,
            mem_limit,
            mem_reservation,
            mem_swappiness,
            oom_kill_disable,
            oom_score_adj,
            pid,
            platform,
            privileged,
            stdin_open,
            stop_signal,
            tty,
        }: compose::PodmanArgs,
    ) -> Result<Self, Self::Error> {
        let BlkioConfig {
            device_read_bps,
            device_read_iops,
            device_write_bps,
            device_write_iops,
            weight: blkio_weight,
            weight_device: blkio_weight_device,
        } = blkio_config.unwrap_or_default();

        Ok(Self {
            device_read_bps: device_read_bps
                .into_iter()
                .map(bps_limit_into_short)
                .collect(),
            device_read_iops: device_read_iops
                .into_iter()
                .map(iops_limit_into_short)
                .collect(),
            device_write_bps: device_write_bps
                .into_iter()
                .map(bps_limit_into_short)
                .collect(),
            device_write_iops: device_write_iops
                .into_iter()
                .map(iops_limit_into_short)
                .collect(),
            blkio_weight,
            blkio_weight_device: blkio_weight_device
                .into_iter()
                .map(|WeightDevice { path, weight }| {
                    format!("{}:{weight}", path.as_path().display())
                })
                .collect(),
            cpu_shares,
            cpu_period: cpu_period.as_ref().map(Duration::as_micros),
            cpu_quota: cpu_quota.as_ref().map(Duration::as_micros),
            cpu_rt_runtime: cpu_rt_runtime.as_ref().map(Duration::as_micros),
            cpu_rt_period: cpu_rt_period.as_ref().map(Duration::as_micros),
            cpus: cpus.map(Into::into),
            cpuset_cpus: (!cpuset.is_empty()).then(|| cpuset.to_string()),
            cgroupns: cgroup.as_ref().map(ToString::to_string),
            cgroup_parent: cgroup_parent.map(Into::into),
            device_cgroup_rule: device_cgroup_rules
                .iter()
                .map(ToString::to_string)
                .collect(),
            add_host: extra_hosts
                .into_iter()
                .map(|(host, ip)| format!("{host}:{ip}"))
                .collect(),
            group_add: group_add.into_iter().map(Into::into).collect(),
            ipc: ipc
                .map(validate_ipc)
                .transpose()
                .wrap_err("`ipc` invalid")?,
            uts: uts.as_ref().map(ToString::to_string),
            log_opt: log_options
                .into_iter()
                .map(|(key, value)| {
                    let mut option = String::from(key);
                    if let Some(value) = value {
                        option.push('=');
                        option.push_str(&String::from(value));
                    }
                    option
                })
                .collect(),
            mac_address: mac_address.as_ref().map(ToString::to_string),
            memory: mem_limit.as_ref().map(ToString::to_string),
            memory_reservation: mem_reservation.as_ref().map(ToString::to_string),
            memory_swappiness: mem_swappiness.map(Into::into),
            oom_kill_disable,
            oom_score_adj: oom_score_adj.map(Into::into),
            pid,
            platform: platform.as_ref().map(ToString::to_string),
            privileged,
            attach: stdin_open
                .then(|| vec!["stdin".to_owned()])
                .unwrap_or_default(),
            stop_signal,
            tty,
            ..Self::default()
        })
    }
}

/// Convert a [`BpsLimit`] from a [`compose_spec::Service`]'s [`BlkioConfig`] into a [`String`]
/// suitable for the `device_read_bps` or `device_write_bps` field of [`PodmanArgs`].
fn bps_limit_into_short(BpsLimit { path, rate }: BpsLimit) -> String {
    format!("{}:{rate}", path.as_path().display())
}

/// Convert a [`IopsLimit`] from a [`compose_spec::Service`]'s [`BlkioConfig`] into a [`String`]
/// suitable for the `device_read_iops` or `device_write_iops` field of [`PodmanArgs`].
fn iops_limit_into_short(IopsLimit { path, rate }: IopsLimit) -> String {
    format!("{}:{rate}", path.as_path().display())
}

/// Validate a compose [`Service`](compose_spec::Service) [`Ipc`] for use in [`PodmanArgs`].
///
/// # Errors
///
/// Returns an error if the given `ipc` is not supported by `podman run --ipc`.
fn validate_ipc(ipc: Ipc) -> color_eyre::Result<String> {
    match ipc {
        Ipc::Shareable => Ok("shareable".to_owned()),
        Ipc::Service(_) => Err(eyre!("`service:` IPC namespace mode is not supported")
            .suggestion("try using the `container:` IPC namespace mode instead")),
        Ipc::Other(ipc) => {
            if ipc.is_empty()
                || ipc.starts_with("container:")
                || ipc == "host"
                || ipc == "none"
                || ipc.starts_with("ns:")
                || ipc == "private"
                || ipc == "shareable"
            {
                Ok(ipc)
            } else {
                Err(eyre!(
                    "`{ipc}` IPC namespace mode is not supported by podman"
                ))
            }
        }
    }
    .with_suggestion(|| {
        format!(
            "see the --ipc section of the {}(1) documentation for supported values: \
                https://docs.podman.io/en/stable/markdown/podman-run.1.html#ipc-ipc",
            "podman-run".bold()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_display_empty() {
        let args = PodmanArgs::default();
        assert!(args.to_string().is_empty());
    }
}
