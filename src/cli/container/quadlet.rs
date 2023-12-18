use std::{
    mem,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use clap::{Args, ValueEnum};
use color_eyre::eyre::{self, Context};
use docker_compose_types::{MapOrEmpty, Volumes};

use crate::{
    cli::ComposeService,
    quadlet::{AutoUpdate, PullPolicy},
};

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

    /// Set the host name that is available inside the container
    ///
    /// Converts to "HostName=NAME"
    #[arg(long, value_name = "NAME")]
    hostname: Option<String>,

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

    /// Pull image policy
    ///
    /// Converts to "Pull=POLICY"
    #[arg(long, value_name = "POLICY")]
    pull: Option<PullPolicy>,

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

    /// Configures namespaced kernel parameters for the container.
    ///
    /// Converts to "Sysctl=NAME=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "NAME=VALUE")]
    sysctl: Vec<String>,

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

    /// Working directory inside the container
    ///
    /// Converts to "WorkingDir=DIR"
    #[arg(short, long, value_name = "DIR")]
    workdir: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum Notify {
    Conmon,
    Container,
}

impl Notify {
    /// Returns `true` if the notify is [`Container`].
    ///
    /// [`Container`]: Notify::Container
    #[must_use]
    fn is_container(self) -> bool {
        matches!(self, Self::Container)
    }
}

impl Default for Notify {
    fn default() -> Self {
        Self::Conmon
    }
}

impl From<QuadletOptions> for crate::quadlet::Container {
    fn from(value: QuadletOptions) -> Self {
        let mut label = value.label;
        let auto_update = AutoUpdate::extract_from_labels(&mut label);

        // `--user` is in the format: `uid[:gid]`
        let (user, group) = value.user.map_or((None, None), |user| {
            if let Some((uid, gid)) = user.split_once(':') {
                (Some(uid.into()), Some(gid.into()))
            } else {
                (Some(user), None)
            }
        });

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
            auto_update,
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
            host_name: value.hostname,
            ip: value.ip,
            ip6: value.ip6,
            label,
            log_driver: value.log_driver,
            mount: value.mount,
            network: value.network,
            rootfs: value.rootfs,
            notify: value.sdnotify.is_container(),
            publish_port: value.publish,
            pull: value.pull,
            read_only: value.read_only,
            run_init: value.init,
            secret: value.secret,
            sysctl: value.sysctl,
            tmpfs,
            timezone: value.tz,
            user,
            user_ns: value.userns,
            volatile_tmp,
            volume: value.volume,
            working_dir: value.workdir,
            ..Self::default()
        }
    }
}

impl TryFrom<ComposeService> for QuadletOptions {
    type Error = color_eyre::Report;

    fn try_from(mut value: ComposeService) -> Result<Self, Self::Error> {
        (&mut value).try_into()
    }
}

impl TryFrom<&mut ComposeService> for QuadletOptions {
    type Error = color_eyre::Report;

    #[allow(clippy::too_many_lines)]
    fn try_from(value: &mut ComposeService) -> Result<Self, Self::Error> {
        let service = &mut value.service;

        let Healthcheck {
            health_cmd,
            health_interval,
            health_timeout,
            health_retries,
            health_start_period,
        } = service
            .healthcheck
            .take()
            .map(Healthcheck::from)
            .unwrap_or_default();

        let publish =
            ports_try_into_publish(mem::take(&mut service.ports)).wrap_err("invalid port")?;

        let env = match mem::take(&mut service.environment) {
            docker_compose_types::Environment::List(list) => list,
            docker_compose_types::Environment::KvPair(map) => map
                .into_iter()
                .map(|(key, value)| {
                    let value = value.as_ref().map(ToString::to_string).unwrap_or_default();
                    format!("{key}={value}")
                })
                .collect(),
        };

        let network = service
            .network_mode
            .take()
            .map(|mode| match mode.as_str() {
                "bridge" | "host" | "none" => Ok(mode),
                s if s.starts_with("container") => Ok(mode),
                s if s.starts_with("service") => Ok(mode),
                _ => Err(eyre::eyre!("network_mode `{mode}` is unsupported")),
            })
            .transpose()?
            .into_iter()
            .chain(map_networks(mem::take(&mut service.networks)))
            .collect();

        let label = match mem::take(&mut service.labels) {
            docker_compose_types::Labels::List(vec) => vec,
            docker_compose_types::Labels::Map(map) => map
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        };

        let sysctl = match mem::take(&mut service.sysctls) {
            docker_compose_types::SysCtls::List(vec) => vec,
            docker_compose_types::SysCtls::Map(map) => map
                .into_iter()
                .map(|(key, value)| {
                    if let Some(value) = value {
                        format!("{key}={value}")
                    } else {
                        key + "=null"
                    }
                })
                .collect(),
        };

        let mut tmpfs = service
            .tmpfs
            .take()
            .map(|tmpfs| match tmpfs {
                docker_compose_types::Tmpfs::Simple(tmpfs) => vec![tmpfs],
                docker_compose_types::Tmpfs::List(tmpfs) => tmpfs,
            })
            .unwrap_or_default();

        let mut mount = Vec::new();

        let volume =
            volumes_try_into_short(value, &mut tmpfs, &mut mount).wrap_err("invalid volume")?;

        let service = &mut value.service;

        let env_file = service
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
            cap_add: mem::take(&mut service.cap_add),
            name: service.container_name.take(),
            cap_drop: mem::take(&mut service.cap_drop),
            publish,
            env,
            env_file,
            network,
            device: mem::take(&mut service.devices),
            label,
            health_cmd,
            health_interval,
            health_retries,
            health_start_period,
            health_timeout,
            hostname: service.hostname.take(),
            sysctl,
            tmpfs,
            mount,
            user: service.user.take(),
            userns: service.userns_mode.take(),
            expose: mem::take(&mut service.expose),
            log_driver: service
                .logging
                .as_mut()
                .map(|logging| mem::take(&mut logging.driver)),
            init: service.init,
            volume,
            workdir: service.working_dir.take().map(Into::into),
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
                #[allow(clippy::indexing_slicing)]
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

fn ports_try_into_publish(ports: docker_compose_types::Ports) -> color_eyre::Result<Vec<String>> {
    match ports {
        docker_compose_types::Ports::Short(ports) => Ok(ports),
        docker_compose_types::Ports::Long(ports) => ports
            .into_iter()
            .map(|port| {
                let docker_compose_types::Port {
                    target,
                    host_ip,
                    published,
                    protocol,
                    mode,
                } = port;
                if let Some(mode) = mode {
                    eyre::ensure!(mode == "host", "unsupported port mode: {mode}");
                }

                let host_ip = host_ip.map(|host_ip| host_ip + ":").unwrap_or_default();

                let host_port = published
                        .map(|port| match port {
                            docker_compose_types::PublishedPort::Single(port) => port.to_string(),
                            docker_compose_types::PublishedPort::Range(range) => range,
                        } + ":")
                        .unwrap_or_default();

                let protocol = protocol
                    .map(|protocol| format!("/{protocol}"))
                    .unwrap_or_default();

                Ok(format!("{host_ip}{host_port}{target}{protocol}"))
            })
            .collect(),
    }
}

/// Takes the [`Volumes`] from a service and converts them to short form if possible, or adds
/// them to the `tmpfs` or `mount` options if not.
fn volumes_try_into_short(
    service: &mut ComposeService,
    tmpfs: &mut Vec<String>,
    mount: &mut Vec<String>,
) -> color_eyre::Result<Vec<String>> {
    mem::take(&mut service.service.volumes)
        .into_iter()
        .filter_map(|volume| match volume {
            Volumes::Simple(volume) => match volume.split_once(':') {
                Some((source, target))
                    if !source.starts_with(['.', '/', '~'])
                        && service.volume_has_options(source) =>
                {
                    // not bind mount or anonymous volume which has options which require a
                    // separate volume unit to define
                    Some(Ok(format!("{source}.volume:{target}")))
                }
                _ => Some(Ok(volume)),
            },
            Volumes::Advanced(volume) => {
                let docker_compose_types::AdvancedVolumes {
                    source,
                    target,
                    _type: kind,
                    read_only,
                    bind,
                    volume,
                    tmpfs: tmpfs_settings,
                } = volume;

                match kind.as_str() {
                    // volume or bind mount without extra options
                    "bind" | "volume" if bind.is_none() => {
                        let Some(mut source) = source else {
                            return Some(Err(eyre::eyre!("{kind} mount without a source")));
                        };
                        if kind == "volume" && service.volume_has_options(&source) {
                            source += ".volume";
                        }
                        source += ":";

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

                        Some(Ok(format!("{source}{target}{options}")))
                    }
                    // bind mount with extra options
                    "bind" => {
                        let Some(source) = source else {
                            return Some(Err(eyre::eyre!("bind mount without a source")));
                        };
                        let read_only = if read_only { ",ro" } else { "" };
                        let propagation = bind
                            .map(|bind| format!(",bind-propagation={}", bind.propagation))
                            .unwrap_or_default();
                        mount.push(format!(
                            "type=bind,source={source},destination={target}{read_only}{propagation}"
                        ));
                        None
                    }
                    "tmpfs" => {
                        let mut options = Vec::new();
                        if read_only {
                            options.push(String::from("ro"));
                        }
                        if let Some(docker_compose_types::TmpfsSettings { size }) = tmpfs_settings {
                            options.push(format!("size={size}"));
                        }
                        let options = if options.is_empty() {
                            String::new()
                        } else {
                            format!(":{}", options.join(","))
                        };
                        tmpfs.push(format!("{target}{options}"));
                        None
                    }
                    _ => Some(Err(eyre::eyre!("unsupported volume type: {kind}"))),
                }
            }
        })
        .collect()
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
                let options =
                    if let MapOrEmpty::Map(docker_compose_types::AdvancedNetworkSettings {
                        ipv4_address,
                        ipv6_address,
                        aliases,
                    }) = settings
                    {
                        let mut options = Vec::new();
                        for ip in ipv4_address.into_iter().chain(ipv6_address) {
                            options.push(format!("ip={ip}"));
                        }
                        for alias in aliases {
                            options.push(format!("alias={alias}"));
                        }
                        if options.is_empty() {
                            String::new()
                        } else {
                            format!(":{}", options.join(","))
                        }
                    } else {
                        String::new()
                    };
                format!("{network}.network{options}")
            })
            .collect(),
    }
}
