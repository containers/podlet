use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use clap::{Args, ValueEnum};

use super::unsupported_option;

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
    health_retries: Option<u32>,

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

impl TryFrom<docker_compose_types::Service> for QuadletOptions {
    type Error = color_eyre::Report;

    fn try_from(mut value: docker_compose_types::Service) -> Result<Self, Self::Error> {
        (&mut value).try_into()
    }
}

impl TryFrom<&mut docker_compose_types::Service> for QuadletOptions {
    type Error = color_eyre::Report;

    fn try_from(value: &mut docker_compose_types::Service) -> Result<Self, Self::Error> {
        let Healthcheck {
            health_cmd,
            health_interval,
            health_timeout,
            health_retries,
            health_start_period,
        } = value
            .healthcheck
            .take()
            .map(Healthcheck::from)
            .unwrap_or_default();

        let env = value
            .environment
            .take()
            .map(|env| match env {
                docker_compose_types::Environment::List(list) => list,
                docker_compose_types::Environment::KvPair(map) => map
                    .into_iter()
                    .map(|(key, value)| {
                        let value = value
                            .map(|value| match value {
                                docker_compose_types::EnvTypes::String(string) => string,
                                docker_compose_types::EnvTypes::Number(num) => num.to_string(),
                                docker_compose_types::EnvTypes::Bool(bool) => bool.to_string(),
                                docker_compose_types::EnvTypes::Null => String::new(),
                            })
                            .unwrap_or_default();
                        format!("{key}={value}")
                    })
                    .collect(),
            })
            .unwrap_or_default();

        let network = value
            .network_mode
            .take()
            .map(|mode| match mode.as_str() {
                "bridge" | "host" | "none" => Ok(mode),
                s if s.starts_with("container") => Ok(mode),
                _ => Err(unsupported_option(&format!("network_mode: {mode}"))),
            })
            .into_iter()
            .chain(
                value
                    .networks
                    .take()
                    .map(map_networks)
                    .unwrap_or_default()
                    .into_iter()
                    .map(Ok),
            )
            .collect::<Result<_, _>>()?;

        let label = value
            .labels
            .take()
            .map(|labels| {
                labels
                    .0
                    .into_iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect()
            })
            .unwrap_or_default();

        let mut tmpfs = Vec::new();

        let volume = value
            .volumes
            .take()
            .map(|volumes| volumes_to_short(volumes, &mut tmpfs))
            .unwrap_or_default();

        let env_file = value
            .env_file
            .take()
            .map(|env_file| match env_file {
                docker_compose_types::EnvFile::Simple(s) => vec![s.into()],
                docker_compose_types::EnvFile::List(list) => {
                    list.into_iter().map(Into::into).collect()
                }
            })
            .unwrap_or_default();

        Ok(Self {
            cap_add: value.cap_add.take().unwrap_or_default(),
            name: value.container_name.take(),
            publish: value.ports.take().unwrap_or_default(),
            env,
            env_file,
            network,
            device: value.devices.take().unwrap_or_default(),
            label,
            health_cmd,
            health_interval,
            health_retries,
            health_start_period,
            health_timeout,
            tmpfs,
            user: value.user.take(),
            expose: value.expose.drain(..).collect(),
            log_driver: value.logging.as_ref().map(|logging| logging.driver.clone()),
            init: value.init,
            volume,
            ..Self::default()
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Healthcheck {
    health_cmd: Option<String>,
    health_interval: Option<String>,
    health_timeout: Option<String>,
    health_retries: Option<u32>,
    health_start_period: Option<String>,
}

impl From<docker_compose_types::Healthcheck> for Healthcheck {
    fn from(value: docker_compose_types::Healthcheck) -> Self {
        let docker_compose_types::Healthcheck {
            test,
            interval,
            timeout,
            retries,
            start_period,
            mut disable,
        } = value;

        let mut command = test.and_then(|test| match test {
            docker_compose_types::HealthcheckTest::Single(s) => Some(s),
            docker_compose_types::HealthcheckTest::Multiple(test) => {
                match test.first().map(String::as_str) {
                    Some("NONE") => {
                        disable = true;
                        None
                    }
                    Some("CMD") => Some(format!("{:?}", &test[1..])),
                    Some("CMD-SHELL") => Some(shlex::join(test[1..].iter().map(String::as_str))),
                    _ => None,
                }
            }
        });

        if disable {
            command = Some(String::from("none"));
        }

        let retries = (retries > 0).then(|| u32::try_from(retries).unwrap_or_default());
        Self {
            health_cmd: command,
            health_interval: interval,
            health_timeout: timeout,
            health_retries: retries,
            health_start_period: start_period,
        }
    }
}

fn volumes_to_short(
    volumes: docker_compose_types::Volumes,
    tmpfs: &mut Vec<String>,
) -> Vec<String> {
    match volumes {
        docker_compose_types::Volumes::Simple(volumes) => volumes
            .into_iter()
            .map(|volume| match volume.split_once(':') {
                Some((source, target)) if !source.starts_with(['.', '/', '~']) => {
                    format!("{source}.volume:{target}")
                }
                _ => volume,
            })
            .collect(),
        docker_compose_types::Volumes::Advanced(volumes) => volumes
            .into_iter()
            .filter_map(|volume| {
                let docker_compose_types::AdvancedVolumes {
                    source,
                    target,
                    _type: kind,
                    read_only,
                    volume,
                } = volume;

                if kind == "tmpfs" {
                    tmpfs.push(target);
                    None
                } else {
                    let source = source
                        .map(|source| {
                            if kind == "bind" {
                                source + ":"
                            } else {
                                source + ".volume:"
                            }
                        })
                        .unwrap_or_default();

                    let mut options = Vec::new();
                    if read_only {
                        options.push("ro");
                    }
                    if let Some(docker_compose_types::Volume { nocopy: true }) = volume {
                        options.push("nocopy");
                    }
                    let options = if options.is_empty() {
                        String::new()
                    } else {
                        format!(":{}", options.join(","))
                    };

                    Some(format!("{source}{target}{options}"))
                }
            })
            .collect(),
    }
}

fn map_networks(networks: docker_compose_types::Networks) -> Vec<String> {
    match networks {
        docker_compose_types::Networks::Simple(networks) => networks
            .into_iter()
            .map(|network| network + ".network")
            .collect(),
        docker_compose_types::Networks::Advanced(networks) => networks
            .0
            .into_iter()
            .map(|(network, settings)| {
                let options = settings
                    .map(|settings| format!(":ip={}", settings.ipv4_address))
                    .unwrap_or_default();
                format!("{network}.network{options}")
            })
            .collect(),
    }
}
