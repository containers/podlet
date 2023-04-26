use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use clap::{Args, ValueEnum};

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Default, Debug, Clone, PartialEq)]
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
    pub name: Option<String>,

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
    ip: Option<Ipv4Addr>,

    /// Specify a static IPv6 address for the container
    ///
    /// Converts to "IP6=IPV6"
    #[arg(long, value_name = "IPV6")]
    ip6: Option<Ipv6Addr>,

    /// Set one or more OCI labels on the container
    ///
    /// Converts to "Label=KEY=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(short, long, value_name = "KEY=VALUE")]
    label: Vec<String>,

    /// Logging driver for the container
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

    /// Control sd-notify behavior
    ///
    /// If `container`, converts to "Notify=true"
    #[arg(long, value_enum, default_value_t)]
    sdnotify: Notify,

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

    /// Run an init inside the container
    ///
    /// Converts to "RunInit=true"
    #[arg(long)]
    init: bool,

    /// Give the container access to a secret
    ///
    /// Converts to "Secret=SECRET[,OPT=OPT,...]"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "SECRET[,OPT=OPT,...]")]
    secret: Vec<String>,

    /// Create a tmpfs mount
    ///
    /// Converts to "Tmpfs=FS" or, if FS == /tmp, "VolatileTmp=true"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "FS")]
    tmpfs: Vec<String>,

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

    /// Set the user namespace mode for the container
    ///
    /// Converts to "UserNS=MODE"
    #[arg(long, value_name = "MODE")]
    userns: Option<String>,

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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum Notify {
    Conmon,
    Container,
}

impl Default for Notify {
    fn default() -> Self {
        Self::Conmon
    }
}

impl From<QuadletOptions> for crate::quadlet::Container {
    fn from(value: QuadletOptions) -> Self {
        let (user, group) = if let Some(user) = value.user {
            if let Some((uid, gid)) = user.split_once(':') {
                (Some(String::from(uid)), Some(String::from(gid)))
            } else {
                (Some(user), None)
            }
        } else {
            (None, None)
        };

        let mut tmpfs = value.tmpfs;
        let mut volatile_tmp = false;
        tmpfs.retain(|tmpfs| {
            if tmpfs == "/tmp" {
                volatile_tmp = true;
                false
            } else {
                true
            }
        });

        Self {
            add_capability: value.cap_add,
            add_device: value.device,
            annotation: value.annotation,
            container_name: value.name,
            drop_capability: value.cap_drop,
            environment: value.env,
            environment_file: value.env_file,
            environment_host: value.env_host,
            expose_host_port: value.expose,
            group,
            health_cmd: value.health_cmd,
            health_interval: value.health_interval,
            health_on_failure: value.health_on_failure,
            health_retries: value.health_retries,
            health_start_period: value.health_start_period,
            health_startup_cmd: value.health_startup_cmd,
            health_startup_interval: value.health_startup_interval,
            health_startup_retries: value.health_startup_retries,
            health_startup_success: value.health_startup_success,
            health_startup_timeout: value.health_startup_timeout,
            health_timeout: value.health_timeout,
            ip: value.ip,
            ip6: value.ip6,
            label: value.label,
            log_driver: value.log_driver,
            mount: value.mount,
            network: value.network,
            rootfs: value.rootfs,
            notify: match value.sdnotify {
                Notify::Conmon => false,
                Notify::Container => true,
            },
            publish_port: value.publish,
            read_only: value.read_only,
            run_init: value.init,
            secret: value.secret,
            tmpfs,
            timezone: value.tz,
            user,
            user_ns: value.userns,
            volatile_tmp,
            volume: value.volume,
            ..Self::default()
        }
    }
}
