use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use clap::{ArgAction, Args};

#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
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

        write!(f, "{}", shlex::join(args))
    }
}