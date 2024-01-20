use std::{
    mem,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

use clap::{ArgAction, Args, ValueEnum};
use color_eyre::{
    eyre::{self, Context},
    owo_colors::OwoColorize,
    Section,
};
use docker_compose_types::{MapOrEmpty, Volumes};
use smart_default::SmartDefault;

use crate::{
    cli::ComposeService,
    quadlet::{
        container::{Device, Mount, PullPolicy, Rootfs},
        AutoUpdate,
    },
};

#[allow(clippy::module_name_repetitions, clippy::struct_excessive_bools)]
#[derive(Args, SmartDefault, Debug, Clone, PartialEq)]
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
    device: Vec<Device>,

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

    /// Set custom DNS servers
    ///
    /// Converts to "DNS=IP_ADDRESS"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "IP_ADDRESS")]
    dns: Vec<String>,

    /// Set custom DNS options
    ///
    /// Converts to "DNSOption=OPTION"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "OPTION")]
    dns_option: Vec<String>,

    /// Set custom DNS search domains
    ///
    /// Converts to "DNSSearch=DOMAIN"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "DOMAIN")]
    dns_search: Vec<String>,

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

    /// Run the container in a new user namespace using the supplied GID mapping
    ///
    /// Converts to "GIDMap=[FLAGS]CONTAINER_GID:FROM_GID[:AMOUNT]"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "[FLAGS]CONTAINER_GID:FROM_GID[:AMOUNT]")]
    gidmap: Vec<String>,

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
    mount: Vec<Mount>,

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

    /// Tune the container’s pids limit
    ///
    /// Converts to "PidsLimit=LIMIT"
    #[arg(long, value_name = "LIMIT")]
    pids_limit: Option<i16>,

    /// The rootfs to use for the container
    ///
    /// Converts to "Rootfs=PATH"
    #[arg(long, value_name = "PATH[:OPTIONS]")]
    rootfs: Option<Rootfs>,

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

    /// When running containers in read-only mode mount a read-write tmpfs on
    /// `/dev`, `/dev/shm`, `/run`, `/tmp`, and `/var/tmp`
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    #[default = true]
    read_only_tmpfs: bool,

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

    /// Size of /dev/shm
    ///
    /// Converts to "ShmSize=NUMBER[UNIT]"
    #[arg(long, value_name = "NUMBER[UNIT]")]
    shm_size: Option<String>,

    /// Name of range listed in /etc/subgid for use in user namespace
    ///
    /// Converts to "SubGIDMap=NAME"
    #[arg(long, value_name = "NAME")]
    subgidname: Option<String>,

    /// Name of range listed in /etc/subuid for use in user namespace
    ///
    /// Converts to "SubUIDMap=NAME"
    #[arg(long, value_name = "NAME")]
    subuidname: Option<String>,

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

    /// Run the container in a new user namespace using the supplied UID mapping
    ///
    /// Converts to "UIDMap=[FLAGS]CONTAINER_UID:FROM_UID[:AMOUNT]"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "[FLAGS]CONTAINER_UID:FROM_UID[:AMOUNT]")]
    uidmap: Vec<String>,

    /// Ulimit options; set the ulimit values inside of the container
    ///
    /// Converts to "Ulimit=OPTION"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "OPTION")]
    ulimit: Vec<String>,

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

        Self {
            add_capability: value.cap_add,
            add_device: value.device,
            annotation: value.annotation,
            auto_update,
            container_name: value.name,
            dns: value.dns,
            dns_option: value.dns_option,
            dns_search: value.dns_search,
            drop_capability: value.cap_drop,
            environment: value.env,
            environment_file: value.env_file,
            environment_host: value.env_host,
            expose_host_port: value.expose,
            gid_map: value.gidmap,
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
            pids_limit: value.pids_limit,
            publish_port: value.publish,
            pull: value.pull,
            read_only: value.read_only,
            read_only_tmpfs: value.read_only_tmpfs,
            run_init: value.init,
            secret: value.secret,
            shm_size: value.shm_size,
            sub_gid_map: value.subgidname,
            sub_uid_map: value.subuidname,
            sysctl: value.sysctl,
            tmpfs: value.tmpfs,
            timezone: value.tz,
            uid_map: value.uidmap,
            ulimit: value.ulimit,
            user,
            user_ns: value.userns,
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

        let device = mem::take(&mut service.devices)
            .into_iter()
            .map(|device| {
                Device::from_str(&device).wrap_err_with(|| format!("invalid device: {device}"))
            })
            .collect::<Result<_, _>>()?;

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
            .map(filter_network_mode)
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

        let ulimit = mem::take(&mut service.ulimits)
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

        let mut tmpfs = service
            .tmpfs
            .take()
            .map(|tmpfs| match tmpfs {
                docker_compose_types::Tmpfs::Simple(tmpfs) => vec![tmpfs],
                docker_compose_types::Tmpfs::List(tmpfs) => tmpfs,
            })
            .unwrap_or_default();

        let volume = volumes_try_into_short(value, &mut tmpfs).wrap_err("invalid volume")?;

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
            dns: mem::take(&mut service.dns),
            cap_drop: mem::take(&mut service.cap_drop),
            publish,
            env,
            env_file,
            network,
            device,
            label,
            health_cmd,
            health_interval,
            health_retries,
            health_start_period,
            health_timeout,
            hostname: service.hostname.take(),
            shm_size: service.shm_size.take(),
            sysctl,
            tmpfs,
            ulimit,
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

#[allow(clippy::struct_field_names)]
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

/// Takes the [`Volumes`] from a service and converts them to short form, or adds them to the
/// `tmpfs` options if a tmpfs mount.
fn volumes_try_into_short(
    service: &mut ComposeService,
    tmpfs: &mut Vec<String>,
) -> color_eyre::Result<Vec<String>> {
    mem::take(&mut service.service.volumes)
        .into_iter()
        .filter_map(|volume| match volume {
            Volumes::Simple(volume) => match volume.split_once(':') {
                Some((source, target))
                    if !source.starts_with(['.', '/', '~'])
                        && service.volume_has_options(source) =>
                {
                    // source is a volume which has options which require a separate volume unit
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
                    volume: volume_options,
                    tmpfs: tmpfs_settings,
                } = volume;

                match kind.as_str() {
                    "bind" | "volume" => {
                        // Format is "[volume|host-dir:]container-dir[:options]"
                        let Some(mut volume) = source else {
                            return Some(Err(eyre::eyre!("{kind} mount without a source")));
                        };

                        if kind == "volume" && service.volume_has_options(&volume) {
                            // source is a volume which has options requiring a separate volume unit
                            volume.push_str(".volume");
                        }

                        volume.push(':');
                        volume.push_str(&target);

                        let mut options = Vec::new();

                        if read_only {
                            options.push("ro");
                        }

                        // Bind propagation is not a valid option for short syntax in compose,
                        // but it is for podman.
                        if let Some(bind) = &bind {
                            options.push(&bind.propagation);
                        }

                        if volume_options.is_some_and(|options| options.nocopy) {
                            options.push("nocopy");
                        }

                        if !options.is_empty() {
                            volume.push(':');
                            volume.push_str(&options.join(","));
                        }

                        Some(Ok(volume))
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

/// Filters out unsupported compose service `network_mode`s.
///
/// # Errors
///
/// Returns an error if the given `mode` is not supported by `podman run --network`.
fn filter_network_mode(mode: String) -> color_eyre::Result<String> {
    match mode.as_str() {
        "host" | "none" | "private" => Ok(mode),
        s if s.starts_with("bridge")
            || s.starts_with("container")
            || s.starts_with("slirp4netns")
            || s.starts_with("pasta") =>
        {
            Ok(mode)
        }
        s if s.starts_with("service") => Err(eyre::eyre!(
            "network_mode `service:` is not supported by podman"
        ))
        .suggestion("try using the `container:` network_mode instead"),
        _ => Err(eyre::eyre!("network_mode `{mode}` is not supported")),
    }
    .with_suggestion(|| {
        format!(
            "see the --network section of the {} documentation for supported values: \
                https://docs.podman.io/en/stable/markdown/podman-run.1.html#network-mode-net",
            "podman-run(1)".bold()
        )
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_convert() {
        assert_eq!(
            crate::quadlet::Container::default(),
            QuadletOptions::default().into(),
        );
    }
}
