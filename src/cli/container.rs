use std::{
    borrow::Cow,
    fmt::Display,
    path::{Path, PathBuf},
};

use clap::{ArgAction, Args};

#[derive(Args, Debug, Clone, PartialEq)]
pub struct QuadletOptions {
    /// Add Linux capabilities
    ///
    /// Converts to "AddCapability=CAPABILITY"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CAPABILITY")]
    cap_add: Vec<String>,

    /// Add a device node from the host into the container
    ///
    /// Converts to "AddDevice=DEVICE"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]")]
    device: Vec<String>,

    /// Add an annotation to the container
    ///
    /// Converts to "Annotation=KEY=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    annotation: Vec<String>,

    /// The (optional) name of the container
    ///
    /// The default name is `systemd-%N`, where `%N` is the name of the service
    ///
    /// Converts to "ContainerName=NAME"
    #[arg(long)]
    name: Option<String>,

    /// Drop Linux capability from the default podman capability set
    ///
    /// If unspecified, the default is `all`
    ///
    /// Converts to "DropCapability=CAPABILITY"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CAPABILITY")]
    cap_drop: Vec<String>,

    /// Set environment variables in the container
    ///
    /// Converts to "Environment=ENV"
    ///
    /// Can be specified multiple times
    #[arg(short, long)]
    env: Vec<String>,

    /// Read in a line-delimited file of environment variables
    ///
    /// Converts to "EnvironmentFile=FILE"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "FILE")]
    env_file: Vec<PathBuf>,

    /// Use the host environment in the container
    ///
    /// Converts to "EnvironmentHost=true"
    #[arg(long)]
    env_host: bool,

    /// Exposes a port, or a range of ports, from the host to the container
    ///
    /// Converts to "ExposeHostPort=PORT"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "PORT")]
    expose: Vec<String>,

    /// Set or alter a healthcheck command for the container
    ///
    /// Converts to "HealthCmd=COMMAND"
    #[arg(long, value_name = "COMMAND")]
    health_cmd: Option<String>,

    /// Set an interval for the healthchecks
    ///
    /// Converts to "HealthInterval=INTERVAL"
    #[arg(long, value_name = "INTERVAL")]
    health_interval: Option<String>,

    /// Action to take once the container transitions to an unhealthy state
    ///
    /// Converts to "HealthOnFailure=ACTION"
    #[arg(long, value_name = "ACTION")]
    health_on_failure: Option<String>,

    /// The number of retries allowed before a healthcheck is considered unhealthy
    ///
    /// Converts to "HealthRetries=RETRIES"
    #[arg(long, value_name = "RETRIES")]
    health_retries: Option<u16>,

    /// The initialization time needed for the container to bootstrap
    ///
    /// Converts to "HealthStartPeriod=PERIOD"
    #[arg(long, value_name = "PERIOD")]
    health_start_period: Option<String>,

    /// Set a startup healthcheck command for the container
    ///
    /// Converts to "HealthStartupCmd=COMMAND"
    #[arg(long, value_name = "COMMAND")]
    health_startup_cmd: Option<String>,

    /// Set an interval for the startup healthcheck
    ///
    /// Converts to "HealthStartupInterval=INTERVAL"
    #[arg(long, value_name = "INTERVAL")]
    health_startup_interval: Option<String>,

    /// The number of retries allowed before the startup healthcheck restarts the container
    ///
    /// Converts to "HealthStartupRetries=RETRIES"
    #[arg(long, value_name = "RETRIES")]
    health_startup_retries: Option<u16>,

    /// The number of successful runs required before the startup healthcheck will succeed
    ///
    /// Converts to "HealthStartupSuccess=RETRIES"
    #[arg(long, value_name = "RETRIES")]
    health_startup_success: Option<u16>,

    /// The maximum time a startup healthcheck has to complete
    ///
    /// Converts to "HealthStartupTimeout=TIMEOUT"
    #[arg(long, value_name = "TIMEOUT")]
    health_startup_timeout: Option<String>,

    /// The maximum time a healthcheck has to complete
    ///
    /// Converts to "HealthTimeout=TIMEOUT"
    #[arg(long, value_name = "TIMEOUT")]
    health_timeout: Option<String>,

    /// Specify a static IPv4 address for the container
    ///
    /// Converts to "IP=IPV4"
    #[arg(long, value_name = "IPV4")]
    ip: Option<String>,

    /// Specify a static IPv6 address for the container
    ///
    /// Converts to "IP6=IPV6"
    #[arg(long, value_name = "IPV6")]
    ip6: Option<String>,

    /// Set one or more OCI labels on the container
    ///
    /// Converts to "Label=KEY=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(short, long, value_name = "KEY=VALUE")]
    label: Vec<String>,

    /// Logging driver for the container
    ///
    /// The default is `passthrough`
    ///
    /// Converts to "LogDriver=DRIVER"
    #[arg(long, value_name = "DRIVER")]
    log_driver: Option<String>,

    /// Attach a filesystem mount to the container
    ///
    /// Converts to "Mount=MOUNT"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "type=TYPE,TYPE-SPECIFIC-OPTION[,...]")]
    mount: Vec<String>,

    /// Specify a custom network for the container
    ///
    /// Converts to "Network=MODE"
    ///
    /// Can be specified multiple times
    #[arg(long, visible_alias = "net", value_name = "MODE")]
    network: Vec<String>,

    /// The rootfs to use for the container
    ///
    /// Converts to "Rootfs=PATH"
    #[arg(long, value_name = "PATH[:OPTIONS]")]
    rootfs: Option<String>,

    /// Publish a container's port, or a range of ports, to the host
    ///
    /// Converts to "PublishPort=PORT"
    ///
    /// Can be specified multiple times
    #[arg(
        short,
        long,
        value_name = "[[IP:][HOST_PORT]:]CONTAINER_PORT[/PROTOCOL]"
    )]
    publish: Vec<String>,

    /// Mount the container's root filesystem as read-only
    ///
    /// Converts to "ReadOnly=true"
    #[arg(long)]
    read_only: bool,

    /// Run the container in a new user namespace using the supplied UID mapping
    ///
    /// Converts to ""RemapUsers=manual" and "RemapUid=UID_MAP""
    #[arg(long, value_name = "CONTAINER_UID:FROM_UID:AMOUNT")]
    uidmap: Option<String>,

    /// Run the container in a new user namespace using the supplied GID mapping
    ///
    /// Converts to "RemapUsers=manual" and "RemapGid=GID_MAP"
    #[arg(long, value_name = "CONTAINER_GID:HOST_GID:AMOUNT")]
    gidmap: Option<String>,

    /// Run an init inside the container
    ///
    /// Converts to "RunInit=true"
    #[arg(long)]
    init: bool,

    /// Set the timezone in the container
    ///
    /// Converts to "Timezone=TIMEZONE"
    #[arg(long, value_name = "TIMEZONE")]
    tz: Option<String>,

    /// Set the UID and, optionally, the GID used in the container
    ///
    /// Converts to "User=UID" and "Group=GID"
    #[arg(short, long, value_name = "UID[:GID]")]
    user: Option<String>,

    /// Mount a volume in the container
    ///
    /// Converts to "Volume=VOLUME"
    ///
    /// Can be specified multiple times
    #[arg(
        short,
        long,
        value_name = "[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]"
    )]
    volume: Vec<String>,
}

impl Display for QuadletOptions {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.cap_add.is_empty() {
            writeln!(f, "AddCapability={}", self.cap_add.join(" "))?;
        }

        for device in &self.device {
            writeln!(f, "AddDevice={device}")?;
        }

        if !self.annotation.is_empty() {
            writeln!(f, "Annotation={}", escape_spaces_join(&self.annotation))?;
        }

        if let Some(name) = &self.name {
            writeln!(f, "ContainerName={name}")?;
        }

        if !self.cap_drop.is_empty() {
            writeln!(f, "DropCapability={}", self.cap_drop.join(" "))?;
        }

        if !self.env.is_empty() {
            writeln!(f, "Environment={}", escape_spaces_join(&self.env))?;
        }

        for file in &self.env_file {
            writeln!(f, "EnvironmentFile={}", file.display())?;
        }

        if self.env_host {
            writeln!(f, "EnvironmentHost=true")?;
        }

        for port in &self.expose {
            writeln!(f, "ExposeHostPort={port}")?;
        }

        if let Some(command) = &self.health_cmd {
            writeln!(f, "HealthCmd={command}")?;
        }

        if let Some(interval) = &self.health_interval {
            writeln!(f, "HealthInterval={interval}")?;
        }

        if let Some(action) = &self.health_on_failure {
            writeln!(f, "HealthOnFailure={action}")?;
        }

        if let Some(retries) = &self.health_retries {
            writeln!(f, "HealthRetries={retries}")?;
        }

        if let Some(period) = &self.health_start_period {
            writeln!(f, "HealthStartPeriod={period}")?;
        }

        if let Some(command) = &self.health_startup_cmd {
            writeln!(f, "HealthStartupCmd={command}")?;
        }

        if let Some(interval) = &self.health_startup_interval {
            writeln!(f, "HealthStartupInterval={interval}")?;
        }

        if let Some(retries) = &self.health_startup_retries {
            writeln!(f, "HealthStartupRetries={retries}")?;
        }

        if let Some(retries) = &self.health_startup_success {
            writeln!(f, "HealthStartupSuccess={retries}")?;
        }

        if let Some(timeout) = &self.health_startup_timeout {
            writeln!(f, "HealthStartupTimeout={timeout}")?;
        }

        if let Some(timeout) = &self.health_timeout {
            writeln!(f, "HealthTimeout={timeout}")?;
        }

        if let Some(ip) = &self.ip {
            writeln!(f, "IP={ip}")?;
        }

        if let Some(ip6) = &self.ip6 {
            writeln!(f, "IP6={ip6}")?;
        }

        if !self.label.is_empty() {
            writeln!(f, "Label={}", escape_spaces_join(&self.label))?;
        }

        if let Some(log_driver) = &self.log_driver {
            writeln!(f, "LogDriver={log_driver}")?;
        }

        for mount in &self.mount {
            writeln!(f, "Mount={mount}")?;
        }

        for network in &self.network {
            writeln!(f, "Network={network}")?;
        }

        if let Some(rootfs) = &self.rootfs {
            writeln!(f, "Rootfs={rootfs}")?;
        }

        for port in &self.publish {
            writeln!(f, "PublishPort={port}")?;
        }

        if self.read_only {
            writeln!(f, "ReadOnly=true")?;
        }

        if self.uidmap.is_some() || self.gidmap.is_some() {
            writeln!(f, "RemapUsers=manual")?;
        }

        if let Some(uidmap) = &self.uidmap {
            writeln!(f, "RemapUid={uidmap}")?;
        }

        if let Some(gidmap) = &self.gidmap {
            writeln!(f, "RemapGid={gidmap}")?;
        }

        if self.init {
            writeln!(f, "RunInit=true")?;
        }

        if let Some(tz) = &self.tz {
            writeln!(f, "Timezone={tz}")?;
        }

        if let Some(user) = &self.user {
            if let Some((uid, gid)) = user.split_once(':') {
                writeln!(f, "User={uid}")?;
                writeln!(f, "Group={gid}")?;
            } else {
                writeln!(f, "User={user}")?;
            }
        }

        for volume in &self.volume {
            writeln!(f, "Volume={volume}")?;
        }

        Ok(())
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Debug, Clone, PartialEq)]
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
    #[arg(long, value_name = "WEIGHT")]
    blkio_weight: Option<u16>,

    /// Block IO relative device weight
    #[arg(long, value_name = "DEVICE:WEIGHT")]
    blkio_weight_device: Option<String>,

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
    cpu_period: Option<usize>,

    /// Limit the CPU CFS (Completely Fair Scheduler) quota
    #[arg(long, value_name = "LIMIT")]
    cpu_quota: Option<usize>,

    /// Limit the CPU real-time period in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    cpu_rt_period: Option<usize>,

    /// Limit the CPU real-time runtime in microseconds
    #[arg(long, value_name = "MICROSECONDS")]
    cpu_rt_runtime: Option<usize>,

    /// CPU shares (relative weight)
    #[arg(short, long, value_name = "SHARES")]
    cpu_shares: Option<u32>,

    /// Number of CPUs
    #[arg(long, value_name = "NUMBER")]
    cpus: Option<f32>,

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
    disable_content_trust: bool,

    /// Set custom DNS servers
    #[arg(long, value_name = "IP_ADDRESS")]
    dns: Option<String>,

    /// Set custom DNS options
    #[arg(long, value_name = "OPTION")]
    dns_option: Option<String>,

    /// Set custom DNS search domains
    #[arg(long, value_name = "DOMAIN")]
    dns_search: Option<String>,

    /// Override the default entrypoint of the image
    #[arg(long, value_name = "\"COMMAND\" | '[\"COMMAND\", \"ARG1\", ...]'")]
    entrypoint: Option<String>,

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

    /// Set container hostname
    #[arg(long, value_name = "NAME")]
    hostname: Option<String>,

    /// Add a user account to /etc/passwd from the host to the container
    #[arg(long, value_name = "NAME")]
    hostuser: Vec<String>,

    /// Set proxy environment variables in the container based on the host proxy vars
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    http_proxy: bool,

    /// How to handle the builtin image volumes
    #[arg(long, value_name = "bind | tmpfs | ignore")]
    image_volume: Option<String>,

    /// Path to the container-init binary
    #[arg(long, value_name = "PATH")]
    init_path: Option<PathBuf>,

    /// keep stdin open even if not attached
    #[arg(short, long)]
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
    no_healthcheck: bool,

    /// Do not create /etc/hosts for the container
    #[arg(long)]
    no_hosts: bool,

    /// Disable OOM Killer for the container
    #[arg(long)]
    oom_kill_disable: bool,

    /// Tune the host’s OOM preferences for the container
    #[arg(long, value_name = "NUM")]
    oom_score_adj: Option<i16>,

    /// Override the OS, defaults to hosts, of the image to be pulled
    #[arg(long)]
    os: Option<String>,

    /// Add entries to /etc/passwd and /etc/group when used with the --user option
    #[arg(long)]
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

    /// Tune the container’s pids limit
    #[arg(long, value_name = "LIMIT")]
    pids_limit: Option<i16>,

    /// Specify the platform for selecting the image
    #[arg(long, value_name = "OS/ARCH")]
    platform: Option<String>,

    /// Run the container in an existing pod
    #[arg(long, value_name = "NAME")]
    pod: Option<String>,

    /// Read the pod ID from the file
    #[arg(long, value_name = "FILE")]
    pod_id_file: Option<PathBuf>,

    /// Pass a number of additional file descriptors into the container
    #[arg(long, value_name = "N")]
    preserve_fds: Option<u16>,

    /// Give extended privileges to the container
    #[arg(long)]
    privileged: bool,

    /// Publish all exposed ports to random ports on the host interfaces
    #[arg(short = 'P', long)]
    publish_all: bool,

    /// Pull image policy
    #[arg(long, value_name = "POLICY")]
    pull: Option<String>,

    /// Suppress output information when pulling images
    #[arg(short, long)]
    quiet: bool,

    /// When running containers in read-only mode mount a read-write tmpfs on /run, /tmp and /var/tmp
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    read_only_tmpfs: bool,

    /// If a container with the same name exists, replace it
    ///
    /// Automatically set by quadlet
    #[arg(long)]
    replace: bool,

    /// Add one or more requirement containers
    #[arg(long, value_name = "CONTAINER[,...]")]
    requires: Option<String>,

    /// Remove container (and pod if created) after exit
    ///
    /// Automatically set by quadlet
    #[arg(long)]
    rm: bool,

    /// After the container exits, remove the container image unless it is used by other containers
    #[arg(long)]
    rmi: bool,
}

impl PodmanArgs {
    /// The total resulting number of arguments
    fn args_len(&self) -> usize {
        (self.add_host.len()
            + self.arch.iter().len()
            + self.attach.len()
            + self.authfile.iter().len()
            + self.blkio_weight.iter().len()
            + self.blkio_weight_device.iter().len()
            + self.cgroup_conf.len()
            + self.cgroup_parent.iter().len()
            + self.cgroupns.iter().len()
            + self.cgroups.iter().len()
            + self.chrootdirs.iter().len()
            + self.cidfile.iter().len()
            + self.conmon_pidfile.iter().len()
            + self.cpu_period.iter().len()
            + self.cpu_quota.iter().len()
            + self.cpu_rt_period.iter().len()
            + self.cpu_rt_runtime.iter().len()
            + self.cpu_shares.iter().len()
            + self.cpus.iter().len()
            + self.cpuset_cpus.iter().len()
            + self.cpuset_mems.iter().len()
            + self.decryption_key.iter().len()
            + self.detach_keys.iter().len()
            + self.device_cgroup_rule.len()
            + self.device_read_bps.len()
            + self.device_read_iops.len()
            + self.device_write_bps.len()
            + self.device_write_iops.len()
            + self.dns.iter().len()
            + self.dns_option.iter().len()
            + self.dns_search.iter().len()
            + self.entrypoint.iter().len()
            + self.env_merge.len()
            + self.group_add.len()
            + self.group_entry.iter().len()
            + self.hostname.iter().len()
            + self.hostuser.len()
            + usize::from(!self.http_proxy)
            + self.image_volume.iter().len()
            + self.init_path.iter().len()
            + self.ipc.iter().len()
            + self.label_file.iter().len()
            + self.link_local_ip.iter().len()
            + self.log_opt.len()
            + self.mac_address.iter().len()
            + self.memory.iter().len()
            + self.memory_reservation.iter().len()
            + self.memory_swap.iter().len()
            + self.memory_swappiness.iter().len()
            + self.network_alias.iter().len()
            + self.oom_score_adj.iter().len()
            + self.os.iter().len()
            + self.passwd_entry.iter().len()
            + self.personality.iter().len()
            + self.pid.iter().len()
            + self.pidfile.iter().len()
            + self.pids_limit.iter().len()
            + self.platform.iter().len()
            + self.pod.iter().len()
            + self.pod_id_file.iter().len()
            + self.preserve_fds.iter().len()
            + self.pull.iter().len()
            + usize::from(!self.read_only_tmpfs)
            + self.requires.iter().len())
            * 2
            + usize::from(self.interactive)
            + usize::from(self.no_healthcheck)
            + usize::from(self.no_hosts)
            + usize::from(self.oom_kill_disable)
            + usize::from(self.passwd)
            + usize::from(self.privileged)
            + usize::from(self.publish_all)
            + usize::from(self.quiet)
            + usize::from(self.rmi)
    }
}

fn extend_args<'a, T, U>(args: &mut Vec<&'a str>, arg: &'a str, values: T)
where
    T: IntoIterator<Item = &'a U>,
    U: 'a + AsRef<str>,
{
    args.extend(values.into_iter().flat_map(|value| [arg, value.as_ref()]));
}

impl Display for PodmanArgs {
    #[allow(clippy::similar_names, clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = Vec::with_capacity(self.args_len());

        extend_args(&mut args, "--add-host", &self.add_host);

        extend_args(&mut args, "--arch", &self.arch);

        extend_args(&mut args, "--attach", &self.attach);

        let authfile = self.authfile.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--authfile", &authfile);

        let blkio_weight = self.blkio_weight.map(|weight| weight.to_string());
        extend_args(&mut args, "--blkio-weight", &blkio_weight);

        extend_args(
            &mut args,
            "--blkio-weight-device",
            &self.blkio_weight_device,
        );

        extend_args(&mut args, "--cgroup-conf", &self.cgroup_conf);

        let cgroup_parent = self.cgroup_parent.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--cgroup-parent", &cgroup_parent);

        extend_args(&mut args, "--cgroupns", &self.cgroupns);

        extend_args(&mut args, "--cgroups", &self.cgroups);

        extend_args(&mut args, "--chrootdirs", &self.chrootdirs);

        let cidfile = self.cidfile.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--cidfile", &cidfile);

        let conmon_pidfile = self.conmon_pidfile.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--conmon-pidfile", &conmon_pidfile);

        let cpu_period = self.cpu_period.map(|period| period.to_string());
        extend_args(&mut args, "--cpu-period", &cpu_period);

        let cpu_quota = self.cpu_quota.map(|quota| quota.to_string());
        extend_args(&mut args, "--cpu-quota", &cpu_quota);

        let cpu_rt_period = self.cpu_rt_period.map(|period| period.to_string());
        extend_args(&mut args, "--cpu-rt-period", &cpu_rt_period);

        let cpu_rt_runtime = self.cpu_rt_runtime.map(|runtime| runtime.to_string());
        extend_args(&mut args, "--cpu-rt-runtime", &cpu_rt_runtime);

        let cpu_shares = self.cpu_shares.map(|shares| shares.to_string());
        extend_args(&mut args, "--cpu-shares", &cpu_shares);

        let cpus = self.cpus.map(|cpus| cpus.to_string());
        extend_args(&mut args, "--cpus", &cpus);

        extend_args(&mut args, "--cpuset-cpus", &self.cpuset_cpus);

        extend_args(&mut args, "--cpuset-mems", &self.cpuset_mems);

        extend_args(&mut args, "--decryption-key", &self.decryption_key);

        extend_args(&mut args, "--detach-keys", &self.detach_keys);

        extend_args(&mut args, "--device-cgroup-rule", &self.device_cgroup_rule);

        extend_args(&mut args, "--device-read-bps", &self.device_read_bps);

        extend_args(&mut args, "--device-read-iops", &self.device_read_iops);

        extend_args(&mut args, "--device-write-bps", &self.device_write_bps);

        extend_args(&mut args, "--device-write-iops", &self.device_write_iops);

        extend_args(&mut args, "--dns", &self.dns);

        extend_args(&mut args, "--dns-option", &self.dns_option);

        extend_args(&mut args, "--dns-search", &self.dns_search);

        extend_args(&mut args, "--entrypoint", &self.entrypoint);

        extend_args(&mut args, "--env-merge", &self.env_merge);

        extend_args(&mut args, "--group-add", &self.group_add);

        extend_args(&mut args, "--group-entry", &self.group_entry);

        extend_args(&mut args, "--hostname", &self.hostname);

        extend_args(&mut args, "--hostuser", &self.hostuser);

        if !self.http_proxy {
            args.extend(["--http-proxy", "false"]);
        }

        extend_args(&mut args, "--image-volume", &self.image_volume);

        let init_path = self.init_path.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--init-path", &init_path);

        if self.interactive {
            args.push("--interactive");
        }

        extend_args(&mut args, "--ipc", &self.ipc);

        let label_file = self.label_file.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--label-file", &label_file);

        extend_args(&mut args, "--link-local-ip", &self.link_local_ip);

        extend_args(&mut args, "--log-opt", &self.log_opt);

        extend_args(&mut args, "--mac-address", &self.mac_address);

        extend_args(&mut args, "--memory", &self.memory);

        extend_args(&mut args, "--memory-reservation", &self.memory_reservation);

        extend_args(&mut args, "--memory-swap", &self.memory_swap);

        let memory_swappiness = self
            .memory_swappiness
            .map(|swappiness| swappiness.to_string());
        extend_args(&mut args, "--memory-swappiness", &memory_swappiness);

        extend_args(&mut args, "--network-alias", &self.network_alias);

        if self.no_healthcheck {
            args.push("--no-healthcheck");
        }

        if self.no_hosts {
            args.push("--no-hosts");
        }

        if self.oom_kill_disable {
            args.push("--oom-kill-disable");
        }

        let oom_score_adj = self.oom_score_adj.map(|score| score.to_string());
        extend_args(&mut args, "--oom-score-adj", &oom_score_adj);

        extend_args(&mut args, "--os", &self.os);

        if self.passwd {
            args.push("--passwd");
        }

        extend_args(&mut args, "--passwd-entry", &self.passwd_entry);

        extend_args(&mut args, "--personality", &self.personality);

        extend_args(&mut args, "--pid", &self.pid);

        let pidfile = self.pidfile.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--pidfile", &pidfile);

        let pids_limit = self.pids_limit.map(|limit| limit.to_string());
        extend_args(&mut args, "--pids-limit", &pids_limit);

        extend_args(&mut args, "--platform", &self.platform);

        extend_args(&mut args, "--pod", &self.pod);

        let pod_id_file = self.pod_id_file.as_deref().map(Path::to_string_lossy);
        extend_args(&mut args, "--pod-id-file", &pod_id_file);

        let preserve_fds = self.preserve_fds.map(|n| n.to_string());
        extend_args(&mut args, "--preserve-fds", &preserve_fds);

        if self.privileged {
            args.push("--privileged");
        }

        if self.publish_all {
            args.push("--publish-all");
        }

        extend_args(&mut args, "--pull", &self.pull);

        if self.quiet {
            args.push("--quiet");
        }

        if !self.read_only_tmpfs {
            args.extend(["--read-only-tmpfs", "false"]);
        }

        extend_args(&mut args, "--requires", &self.requires);

        if self.rmi {
            args.push("--rmi");
        }

        // ----------

        debug_assert_eq!(args.len(), self.args_len());

        let args = shlex::join(args);

        if args.is_empty() {
            Ok(())
        } else {
            writeln!(f, "PodmanArgs={args}")
        }
    }
}

fn escape_spaces_join<'a>(words: impl IntoIterator<Item = &'a String>) -> String {
    words
        .into_iter()
        .map(|word| {
            if word.contains(' ') {
                format!("\"{word}\"").into()
            } else {
                word.into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join(" ")
}
