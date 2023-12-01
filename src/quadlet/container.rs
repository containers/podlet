use std::{
    fmt::{self, Display, Formatter},
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use super::writeln_escape_spaces;

#[derive(Debug, Default, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Container {
    pub add_capability: Vec<String>,
    pub add_device: Vec<String>,
    pub annotation: Vec<String>,
    pub container_name: Option<String>,
    pub drop_capability: Vec<String>,
    pub environment: Vec<String>,
    pub environment_file: Vec<PathBuf>,
    pub environment_host: bool,
    pub exec: Option<String>,
    pub expose_host_port: Vec<String>,
    pub group: Option<String>,
    pub health_cmd: Option<String>,
    pub health_interval: Option<String>,
    pub health_on_failure: Option<String>,
    pub health_retries: Option<u32>,
    pub health_start_period: Option<String>,
    pub health_startup_cmd: Option<String>,
    pub health_startup_interval: Option<String>,
    pub health_startup_retries: Option<u16>,
    pub health_startup_success: Option<u16>,
    pub health_startup_timeout: Option<String>,
    pub health_timeout: Option<String>,
    pub image: String,
    pub ip: Option<Ipv4Addr>,
    pub ip6: Option<Ipv6Addr>,
    pub label: Vec<String>,
    pub log_driver: Option<String>,
    pub mount: Vec<String>,
    pub network: Vec<String>,
    pub no_new_privileges: bool,
    pub rootfs: Option<String>,
    pub notify: bool,
    pub podman_args: Option<String>,
    pub publish_port: Vec<String>,
    pub read_only: bool,
    pub run_init: bool,
    pub seccomp_profile: Option<String>,
    pub security_label_disable: bool,
    pub security_label_file_type: Option<String>,
    pub security_label_level: Option<String>,
    pub security_label_type: Option<String>,
    pub secret: Vec<String>,
    pub sysctl: Vec<String>,
    pub tmpfs: Vec<String>,
    pub timezone: Option<String>,
    pub user: Option<String>,
    pub user_ns: Option<String>,
    pub volatile_tmp: bool,
    pub volume: Vec<String>,
}

impl Display for Container {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Container]")?;

        writeln!(f, "Image={}", self.image)?;

        if !self.add_capability.is_empty() {
            writeln!(f, "AddCapability={}", self.add_capability.join(" "))?;
        }

        for device in &self.add_device {
            writeln!(f, "AddDevice={device}")?;
        }

        if !self.annotation.is_empty() {
            writeln_escape_spaces(f, "Annotation", &self.annotation)?;
        }

        if let Some(name) = &self.container_name {
            writeln!(f, "ContainerName={name}")?;
        }

        if !self.drop_capability.is_empty() {
            writeln!(f, "DropCapability={}", self.drop_capability.join(" "))?;
        }

        if !self.environment.is_empty() {
            writeln_escape_spaces(f, "Environment", &self.environment)?;
        }

        for file in &self.environment_file {
            writeln!(f, "EnvironmentFile={}", file.display())?;
        }

        if self.environment_host {
            writeln!(f, "EnvironmentHost=true")?;
        }

        for port in &self.expose_host_port {
            writeln!(f, "ExposeHostPort={port}")?;
        }

        if let Some(group) = &self.group {
            writeln!(f, "Group={group}")?;
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
            writeln_escape_spaces(f, "Label", &self.label)?;
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

        if self.no_new_privileges {
            writeln!(f, "NoNewPrivileges=true")?;
        }

        if let Some(rootfs) = &self.rootfs {
            writeln!(f, "Rootfs={rootfs}")?;
        }

        if self.notify {
            writeln!(f, "Notify=true")?;
        }

        for port in &self.publish_port {
            writeln!(f, "PublishPort={port}")?;
        }

        if self.read_only {
            writeln!(f, "ReadOnly=true")?;
        }

        if self.run_init {
            writeln!(f, "RunInit=true")?;
        }

        if let Some(profile) = &self.seccomp_profile {
            writeln!(f, "SeccompProfile={profile}")?;
        }

        if self.security_label_disable {
            writeln!(f, "SecurityLabelDisable=true")?;
        }

        if let Some(file_type) = &self.security_label_file_type {
            writeln!(f, "SecurityLabelFileType={file_type}")?;
        }

        if let Some(level) = &self.security_label_level {
            writeln!(f, "SecurityLabelLevel={level}")?;
        }

        if let Some(label_type) = &self.security_label_type {
            writeln!(f, "SecurityLabelType={label_type}")?;
        }

        for secret in &self.secret {
            writeln!(f, "Secret={secret}")?;
        }

        if !self.sysctl.is_empty() {
            writeln_escape_spaces(f, "Sysctl", &self.sysctl)?;
        }

        for tmpfs in &self.tmpfs {
            writeln!(f, "Tmpfs={tmpfs}")?;
        }

        if let Some(timezone) = &self.timezone {
            writeln!(f, "Timezone={timezone}")?;
        }

        if let Some(user) = &self.user {
            writeln!(f, "User={user}")?;
        }

        if let Some(user_ns) = &self.user_ns {
            writeln!(f, "UserNS={user_ns}")?;
        }

        if self.volatile_tmp {
            writeln!(f, "VolatileTmp=true")?;
        }

        for volume in &self.volume {
            writeln!(f, "Volume={volume}")?;
        }

        if let Some(podman_args) = &self.podman_args {
            writeln!(f, "PodmanArgs={podman_args}")?;
        }

        if let Some(exec) = &self.exec {
            writeln!(f, "Exec={exec}")?;
        }

        Ok(())
    }
}
