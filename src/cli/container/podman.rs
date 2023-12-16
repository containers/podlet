use std::{
    fmt::{self, Display, Formatter},
    mem,
    ops::Not,
    path::PathBuf,
};

use clap::{ArgAction, Args};
use color_eyre::eyre::Context;
use serde::Serialize;

#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Args, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PodmanArgs {
    /// Add a custom host-to-IP mapping
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "HOST:IP")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    add_host: Vec<String>,

    /// Override the architecture of the image to be pulled
    ///
    /// Defaults to hosts architecture
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    arch: Option<String>,

    /// Attach to STDIN, STDOUT, or STDERR
    #[arg(short, long, value_name = "STDIN | STDOUT | STDERR")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attach: Vec<String>,

    /// Path of the authentication file
    ///
    /// Default is `${XDG_RUNTIME_DIR}/containers/auth.json`
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    authfile: Option<PathBuf>,

    /// Block IO relative weight, between 10 and 1000
    #[arg(long, value_name = "WEIGHT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    blkio_weight: Option<u16>,

    /// Block IO relative device weight
    #[arg(long, value_name = "DEVICE:WEIGHT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    blkio_weight_device: Option<String>,

    /// Specify the cgroup file to write to and its value
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cgroup_conf: Vec<String>,

    /// Path to cgroups under which the cgroup for the container will be created
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cgroup_parent: Option<PathBuf>,

    /// Set the cgroup namespace for the container
    #[arg(long, value_name = "MODE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cgroupns: Option<String>,

    /// Whether the container will create cgroups
    #[arg(long, value_name = "HOW")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cgroups: Option<String>,

    /// Chroot directories inside the container
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    chrootdirs: Option<String>,

    /// Write container ID to a file
    #[arg(long, value_name = "FILE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cidfile: Option<PathBuf>,

    /// Write the pid of the conmon process to a file
    #[arg(long, value_name = "FILE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    conmon_pidfile: Option<PathBuf>,

    /// Limit the CPU CFS (Completely Fair Scheduler) period
    #[arg(long, value_name = "LIMIT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_period: Option<usize>,

    /// Limit the CPU CFS (Completely Fair Scheduler) quota
    #[arg(long, value_name = "LIMIT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_quota: Option<usize>,

    /// Limit the CPU real-time period in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_rt_period: Option<usize>,

    /// Limit the CPU real-time runtime in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_rt_runtime: Option<usize>,

    /// CPU shares (relative weight)
    #[arg(short, long, value_name = "SHARES")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_shares: Option<u32>,

    /// Number of CPUs
    #[arg(long, value_name = "NUMBER")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpus: Option<f32>,

    /// CPUs in which to allow execution
    #[arg(long, value_name = "NUMBER")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpuset_cpus: Option<String>,

    /// Memory nodes (MEMs) in which to allow execution
    #[arg(long, value_name = "NODES")]
    #[serde(skip_serializing_if = "Option::is_none")]
    cpuset_mems: Option<String>,

    /// Key needed to decrypt the image
    #[arg(long, value_name = "KEY[:PASSPHRASE]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    decryption_key: Option<String>,

    /// Detached mode: run the container in the background
    ///
    /// Automatically set by quadlet
    #[arg(short, long)]
    #[serde(skip_serializing)]
    detach: bool,

    /// Key sequence for detaching a container
    #[arg(long, value_name = "SEQUENCE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    detach_keys: Option<String>,

    /// Add a rule to the cgroup allowed devices list
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "TYPE MAJOR:MINOR MODE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    device_cgroup_rule: Vec<String>,

    /// Limit read rate (in bytes per second) from a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    device_read_bps: Vec<String>,

    /// Limit read rate (in IO operations per second) from a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    device_read_iops: Vec<String>,

    /// Limit write rate (in bytes per second) to a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    device_write_bps: Vec<String>,

    /// Limit write rate (in IO operations per second) to a device
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PATH:RATE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    device_write_iops: Vec<String>,

    /// This is a Docker specific option and is a NOOP
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    disable_content_trust: bool,

    /// Set custom DNS search domains
    #[arg(long, value_name = "DOMAIN")]
    #[serde(skip_serializing_if = "Option::is_none")]
    dns_search: Option<String>,

    /// Override the default entrypoint of the image
    #[arg(long, value_name = "\"COMMAND\" | '[\"COMMAND\", \"ARG1\", ...]'")]
    #[serde(skip_serializing_if = "Option::is_none")]
    entrypoint: Option<String>,

    /// Preprocess default environment variables for the container
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "ENV")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    env_merge: Vec<String>,

    /// Run the container in a new user namespace using the supplied GID mapping
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CONTAINER_GID:HOST_GID:AMOUNT")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gidmap: Vec<String>,

    /// Assign additional groups to the primary user running within the container process
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "GROUP")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    group_add: Vec<String>,

    /// Customize the entry that is written to the /etc/group file within the container
    #[arg(long, value_name = "ENTRY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    group_entry: Option<String>,

    /// Add a user account to /etc/passwd from the host to the container
    #[arg(long, value_name = "NAME")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hostuser: Vec<String>,

    /// Set proxy environment variables in the container based on the host proxy vars
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    http_proxy: bool,

    /// How to handle the builtin image volumes
    #[arg(long, value_name = "bind | tmpfs | ignore")]
    #[serde(skip_serializing_if = "Option::is_none")]
    image_volume: Option<String>,

    /// Path to the container-init binary
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    init_path: Option<PathBuf>,

    /// keep stdin open even if not attached
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    interactive: bool,

    /// Set the IPC namespace mode for the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    ipc: Option<String>,

    /// Read in a line-delimited file of labels
    #[arg(long, value_name = "FILE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    label_file: Option<PathBuf>,

    /// Not implemented
    #[arg(long, value_name = "IP")]
    #[serde(skip_serializing_if = "Option::is_none")]
    link_local_ip: Option<String>,

    /// Logging driver specific options
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "NAME=VALUE")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    log_opt: Vec<String>,

    /// Container network interface MAC address
    #[arg(long, value_name = "ADDRESS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    mac_address: Option<String>,

    /// Memory limit
    #[arg(short, long, value_name = "NUMBER[UNIT]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<String>,

    /// Memory soft limit
    #[arg(long, value_name = "NUMBER[UNIT]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_reservation: Option<String>,

    /// Limit value equal to memory plus swap
    #[arg(long, value_name = "NUMBER[UNIT]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_swap: Option<String>,

    /// Tune the container’s memory swappiness behavior
    #[arg(long, value_name = "NUMBER")]
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_swappiness: Option<u8>,

    /// Add a network-scoped alias for the container
    #[arg(long, value_name = "ALIAS")]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    oom_score_adj: Option<i16>,

    /// Override the OS, defaults to hosts, of the image to be pulled
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<String>,

    /// Add entries to /etc/passwd and /etc/group when used with the --user option
    #[arg(long)]
    #[serde(skip_serializing_if = "Not::not")]
    passwd: bool,

    /// Entry to write to /etc/passwd
    #[arg(long, value_name = "ENTRY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    passwd_entry: Option<String>,

    /// Configure execution domain using personality
    #[arg(long, value_name = "PERSONA")]
    #[serde(skip_serializing_if = "Option::is_none")]
    personality: Option<String>,

    /// Set the PID namespace mode for the container
    #[arg(long, value_name = "MODE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<String>,

    /// Write the container process ID to the file
    #[arg(long, value_name = "PATH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pidfile: Option<PathBuf>,

    /// Tune the container’s pids limit
    #[arg(long, value_name = "LIMIT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pids_limit: Option<i16>,

    /// Specify the platform for selecting the image
    #[arg(long, value_name = "OS/ARCH")]
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,

    /// Run the container in an existing pod
    #[arg(long, value_name = "NAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pod: Option<String>,

    /// Read the pod ID from the file
    #[arg(long, value_name = "FILE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pod_id_file: Option<PathBuf>,

    /// Pass a number of additional file descriptors into the container
    #[arg(long, value_name = "N")]
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// When running containers in read-only mode mount a read-write tmpfs on /run, /tmp and /var/tmp
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    read_only_tmpfs: bool,

    /// If a container with the same name exists, replace it
    ///
    /// Automatically set by quadlet
    #[arg(long)]
    #[serde(skip_serializing)]
    replace: bool,

    /// Add one or more requirement containers
    #[arg(long, value_name = "CONTAINER[,...]")]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    seccomp_policy: Option<String>,

    /// Size of /dev/shm
    #[arg(long, value_name = "NUMBER[UNIT]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    shm_size: Option<String>,

    /// Size of systemd-specific tmpfs mounts: /run, /run/lock, /var/log/journal, and /tmp
    #[arg(long, value_name = "NUMBER[UNIT]")]
    #[serde(skip_serializing_if = "Option::is_none")]
    shm_size_systemd: Option<String>,

    /// Proxy received signals to the container process
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[serde(skip_serializing_if = "skip_true")]
    sig_proxy: bool,

    /// Signal to stop a container
    #[arg(long, value_name = "SIGNAL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<String>,

    /// Timeout to stop a container
    ///
    /// Default is 10
    #[arg(long, value_name = "SECONDS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_timeout: Option<u16>,

    /// Name of range listed in /etc/subgid for use in user namespace
    #[arg(long, value_name = "NAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    subgidname: Option<String>,

    /// Name of range listed in /etc/subuid for use in user namespace
    #[arg(long, value_name = "NAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    subuidname: Option<String>,

    /// Run container in systemd mode
    ///
    /// Default is true
    #[arg(long, value_name = "true | false | always")]
    #[serde(skip_serializing_if = "Option::is_none")]
    systemd: Option<String>,

    /// Maximum length of time a container is allowed to run
    #[arg(long, value_name = "SECONDS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u16>,

    /// Require HTTPS and verify certificates when contacting registries
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tls_verify: Option<bool>,

    /// Allocate a pseudo-TTY
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Not::not")]
    tty: bool,

    /// Run the container in a new user namespace using the supplied UID mapping
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CONTAINER_UID:FROM_UID:AMOUNT")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    uidmap: Vec<String>,

    /// Ulimit options
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "OPTION")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ulimit: Vec<String>,

    /// Set the umask inside the container
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    umask: Option<String>,

    /// Set variant to use instead of the default architecture variant of the container image
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<String>,

    /// Mount volumes from the specified container
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CONTAINER[:OPTIONS]")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volumes_from: Vec<String>,
}

// ref required for serde's skip_serializing_if
#[allow(clippy::trivially_copy_pass_by_ref)]
fn skip_true(bool: &bool) -> bool {
    *bool
}

impl Default for PodmanArgs {
    fn default() -> Self {
        Self {
            add_host: Vec::new(),
            arch: None,
            attach: Vec::new(),
            authfile: None,
            blkio_weight: None,
            blkio_weight_device: None,
            cgroup_conf: Vec::new(),
            cgroup_parent: None,
            cgroupns: None,
            cgroups: None,
            chrootdirs: None,
            cidfile: None,
            conmon_pidfile: None,
            cpu_period: None,
            cpu_quota: None,
            cpu_rt_period: None,
            cpu_rt_runtime: None,
            cpu_shares: None,
            cpus: None,
            cpuset_cpus: None,
            cpuset_mems: None,
            decryption_key: None,
            detach: false,
            detach_keys: None,
            device_cgroup_rule: Vec::new(),
            device_read_bps: Vec::new(),
            device_read_iops: Vec::new(),
            device_write_bps: Vec::new(),
            device_write_iops: Vec::new(),
            disable_content_trust: false,
            dns_search: None,
            entrypoint: None,
            env_merge: Vec::new(),
            gidmap: Vec::new(),
            group_add: Vec::new(),
            group_entry: None,
            hostuser: Vec::new(),
            http_proxy: true,
            image_volume: None,
            init_path: None,
            interactive: false,
            ipc: None,
            label_file: None,
            link_local_ip: None,
            log_opt: Vec::new(),
            mac_address: None,
            memory: None,
            memory_reservation: None,
            memory_swap: None,
            memory_swappiness: None,
            network_alias: None,
            no_healthcheck: false,
            no_hosts: false,
            oom_kill_disable: false,
            oom_score_adj: None,
            os: None,
            passwd: false,
            passwd_entry: None,
            personality: None,
            pid: None,
            pidfile: None,
            pids_limit: None,
            platform: None,
            pod: None,
            pod_id_file: None,
            preserve_fds: None,
            privileged: false,
            publish_all: false,
            quiet: false,
            read_only_tmpfs: true,
            replace: false,
            requires: None,
            rm: false,
            rmi: false,
            seccomp_policy: None,
            shm_size: None,
            shm_size_systemd: None,
            sig_proxy: true,
            stop_signal: None,
            stop_timeout: None,
            subgidname: None,
            subuidname: None,
            systemd: None,
            timeout: None,
            tls_verify: None,
            tty: false,
            uidmap: Vec::new(),
            ulimit: Vec::new(),
            umask: None,
            variant: None,
            volumes_from: Vec::new(),
        }
    }
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}

impl TryFrom<docker_compose_types::Service> for PodmanArgs {
    type Error = color_eyre::Report;

    fn try_from(mut value: docker_compose_types::Service) -> Result<Self, Self::Error> {
        (&mut value).try_into()
    }
}

impl TryFrom<&mut docker_compose_types::Service> for PodmanArgs {
    type Error = color_eyre::Report;

    fn try_from(value: &mut docker_compose_types::Service) -> Result<Self, Self::Error> {
        let ulimit = mem::take(&mut value.ulimits)
            .0
            .into_iter()
            .map(|(kind, ulimit)| match ulimit {
                docker_compose_types::Ulimit::Single(soft) => format!("{kind}={soft}"),
                docker_compose_types::Ulimit::SoftHard { soft, hard } => {
                    if hard == 0 {
                        format!("{kind}={soft}")
                    } else {
                        format!("{kind}={soft}:{hard}")
                    }
                }
            })
            .collect();

        let entrypoint = value.entrypoint.take().map(|entrypoint| match entrypoint {
            docker_compose_types::Entrypoint::Simple(entrypoint) => entrypoint,
            docker_compose_types::Entrypoint::List(list) => format!("{list:?}"),
        });

        let stop_timeout = value
            .stop_grace_period
            .take()
            .map(|timeout| {
                duration_str::parse(&timeout)
                    .map(|duration| duration.as_secs().try_into().unwrap_or(u16::MAX))
                    .wrap_err_with(|| {
                        format!(
                            "could not parse `stop_grace_period` value `{timeout}` as a duration"
                        )
                    })
            })
            .transpose()?;

        let log_opt = value
            .logging
            .as_mut()
            .and_then(|logging| logging.options.take())
            .unwrap_or_default()
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();

        Ok(Self {
            privileged: value.privileged,
            pid: value.pid.take(),
            ulimit,
            entrypoint,
            group_add: mem::take(&mut value.group_add),
            stop_signal: value.stop_signal.take(),
            stop_timeout,
            ipc: value.ipc.take(),
            interactive: value.stdin_open,
            shm_size: value.shm_size.take(),
            log_opt,
            add_host: mem::take(&mut value.extra_hosts),
            tty: value.tty,
            ..Self::default()
        })
    }
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
