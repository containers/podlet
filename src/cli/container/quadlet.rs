use std::{borrow::Cow, fmt::Display, path::PathBuf};

use clap::Args;

#[allow(clippy::module_name_repetitions)]
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
