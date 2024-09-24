use std::{
    iter,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};

use clap::{builder::TypedValueParser, ArgAction, Args};
use color_eyre::{
    eyre::{ensure, eyre, Context, OptionExt},
    owo_colors::OwoColorize,
    Section,
};
use compose_spec::{
    duration,
    service::{
        self, env_file,
        network_config::{Network, NetworkMode},
        ports,
        volumes::{
            self,
            mount::{Common, Tmpfs, TmpfsOptions},
            ShortVolume,
        },
        Command, ConfigOrSecret, EnvFile, Limit, NetworkConfig, Ulimit,
    },
    Identifier, ItemOrList, ShortOrLong,
};
use smart_default::SmartDefault;

use crate::quadlet::{
    container::{Device, DnsEntry, Mount, Notify, PullPolicy, Rootfs, Volume},
    AutoUpdate,
};

use super::compose;

#[allow(
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::doc_markdown
)]
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
    // TODO: use `Dns` directly if clap ever supports custom collections (https://github.com/clap-rs/clap/issues/3114).
    dns: Vec<DnsEntry>,

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

    /// Drop Linux capability from the default Podman capability set
    ///
    /// If unspecified, the default is `all`
    ///
    /// Converts to "DropCapability=CAPABILITY"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CAPABILITY")]
    cap_drop: Vec<String>,

    /// Override the default entrypoint of the image
    ///
    /// Converts to "Entrypoint=ENTRYPOINT"
    #[arg(long, value_name = "\"COMMAND\" | '[\"COMMAND\", \"ARG1\", ...]'")]
    entrypoint: Option<String>,

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

    /// Assign additional groups to the primary user running within the container process
    ///
    /// Converts to "GroupAdd=GROUP"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "GROUP")]
    group_add: Vec<String>,

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
    health_retries: Option<u64>,

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
    ///
    /// If `healthy`, converts to "Notify=healthy"
    #[arg(long, value_enum, default_value_t)]
    sdnotify: Notify,

    /// Tune the container’s pids limit
    ///
    /// Converts to "PidsLimit=LIMIT"
    #[arg(
        long,
        value_name = "LIMIT",
        allow_negative_numbers = true,
        value_parser = pids_limit_parser()
    )]
    pids_limit: Option<Limit<u32>>,

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

    /// The rootfs to use for the container
    ///
    /// Converts to "Rootfs=PATH"
    #[arg(long, value_name = "PATH[:OPTIONS]")]
    rootfs: Option<Rootfs>,

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

    /// Timeout to stop a container
    ///
    /// Default is 10 seconds
    ///
    /// Converts to "StopTimeout=SECONDS"
    #[arg(long, value_name = "SECONDS")]
    stop_timeout: Option<u64>,

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

    /// Set the timezone in the container
    ///
    /// Converts to "Timezone=TIMEZONE"
    #[arg(long, value_name = "TIMEZONE")]
    tz: Option<String>,

    /// Create a tmpfs mount
    ///
    /// Converts to "Tmpfs=FS" or, if FS == /tmp, "VolatileTmp=true"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "FS")]
    tmpfs: Vec<String>,

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
    volume: Vec<Volume>,

    /// Working directory inside the container
    ///
    /// Converts to "WorkingDir=DIR"
    #[arg(short, long, value_name = "DIR")]
    workdir: Option<PathBuf>,
}

/// Create a [`TypedValueParser`] for parsing the `pids_limit` field of [`QuadletOptions`].
fn pids_limit_parser() -> impl TypedValueParser<Value = Limit<u32>> {
    clap::value_parser!(i64)
        .range(-1..=u32::MAX.into())
        .try_map(|pids_limit| {
            if pids_limit == -1 {
                Ok(Limit::Unlimited)
            } else if let Ok(pids_limit) = pids_limit.try_into() {
                Ok(Limit::Value(pids_limit))
            } else {
                Err("`--pids-limit` must be -1, or a u32")
            }
        })
}

impl From<QuadletOptions> for crate::quadlet::Container {
    // Triggers on uid and gid options
    #[allow(clippy::similar_names)]
    fn from(
        QuadletOptions {
            cap_add: add_capability,
            device: add_device,
            annotation,
            name: container_name,
            dns,
            dns_option,
            dns_search,
            cap_drop: drop_capability,
            entrypoint,
            env,
            env_file: environment_file,
            env_host: environment_host,
            expose: expose_host_port,
            gidmap: gid_map,
            group_add,
            health_cmd,
            health_interval,
            health_on_failure,
            health_retries,
            health_start_period,
            health_startup_cmd,
            health_startup_interval,
            health_startup_retries,
            health_startup_success,
            health_startup_timeout,
            health_timeout,
            hostname: host_name,
            ip,
            ip6,
            mut label,
            log_driver,
            mount,
            network,
            sdnotify: notify,
            pids_limit,
            publish: publish_port,
            pull,
            read_only,
            read_only_tmpfs,
            rootfs,
            init: run_init,
            secret,
            shm_size,
            stop_timeout,
            subgidname: sub_gid_map,
            subuidname: sub_uid_map,
            sysctl,
            tz: timezone,
            tmpfs,
            uidmap: uid_map,
            ulimit,
            user,
            userns: user_ns,
            volume,
            workdir: working_dir,
        }: QuadletOptions,
    ) -> Self {
        let auto_update = AutoUpdate::extract_from_labels(&mut label);

        // `--user` is in the format: `uid[:gid]`
        let (user, group) = user.map_or((None, None), |user| {
            if let Some((uid, gid)) = user.split_once(':') {
                (Some(uid.into()), Some(gid.into()))
            } else {
                (Some(user), None)
            }
        });

        Self {
            add_capability,
            add_device,
            annotation,
            auto_update,
            container_name,
            dns: dns.into(),
            dns_option,
            dns_search,
            drop_capability,
            entrypoint,
            environment: env,
            environment_file,
            environment_host,
            expose_host_port,
            gid_map,
            group,
            group_add,
            health_cmd,
            health_interval,
            health_on_failure,
            health_retries,
            health_start_period,
            health_startup_cmd,
            health_startup_interval,
            health_startup_retries,
            health_startup_success,
            health_startup_timeout,
            health_timeout,
            host_name,
            ip,
            ip6,
            label,
            log_driver,
            mount,
            network,
            notify,
            pids_limit,
            publish_port,
            pull,
            read_only,
            read_only_tmpfs,
            rootfs,
            run_init,
            secret,
            shm_size,
            stop_timeout,
            sub_gid_map,
            sub_uid_map,
            sysctl,
            timezone,
            tmpfs,
            uid_map,
            ulimit,
            user,
            user_ns,
            volume,
            working_dir,
            ..Self::default()
        }
    }
}

impl TryFrom<compose::Quadlet> for QuadletOptions {
    type Error = color_eyre::Report;

    #[allow(clippy::too_many_lines)]
    fn try_from(
        compose::Quadlet {
            cap_add,
            cap_drop,
            container_name,
            devices,
            dns,
            dns_opt,
            dns_search,
            entrypoint,
            env_file,
            environment,
            expose,
            annotations,
            group_add,
            healthcheck,
            hostname,
            init,
            labels,
            log_driver,
            network_config,
            pids_limit,
            ports,
            pull_policy,
            read_only,
            secrets,
            shm_size,
            stop_grace_period,
            sysctls,
            tmpfs,
            ulimits,
            user,
            userns_mode: userns,
            volumes,
            working_dir,
        }: compose::Quadlet,
    ) -> Result<Self, Self::Error> {
        let Healthcheck {
            health_cmd,
            health_interval,
            health_timeout,
            health_retries,
            health_start_period,
            health_startup_interval,
        } = healthcheck
            .unwrap_or_default()
            .try_into()
            .wrap_err("error converting `healthcheck`")?;

        let mut tmpfs = tmpfs
            .into_iter()
            .flat_map(ItemOrList::into_list)
            .map(|tmpfs| tmpfs.as_path().display().to_string())
            .collect();

        let volume = volumes
            .into_iter()
            .filter_map(|volume| volume_try_into_short(volume, &mut tmpfs).transpose())
            .collect::<Result<_, _>>()
            .wrap_err("error converting `volumes`")?;

        Ok(Self {
            cap_add: cap_add.into_iter().collect(),
            cap_drop: cap_drop.into_iter().collect(),
            name: container_name.map(Into::into),
            device: devices.into_iter().map(Into::into).collect(),
            dns: dns
                .into_iter()
                .flat_map(ItemOrList::into_list)
                .map(Into::into)
                .collect(),
            dns_option: dns_opt.into_iter().collect(),
            dns_search: dns_search
                .into_iter()
                .flat_map(ItemOrList::into_list)
                .map(Into::into)
                .collect(),
            entrypoint: entrypoint
                .map(|entrypoint| match entrypoint {
                    Command::String(entrypoint) => Ok(entrypoint),
                    Command::List(entrypoint) => serde_json::to_string(&entrypoint)
                        .wrap_err("error serializing `entrypoint` command as JSON"),
                })
                .transpose()?,
            env_file: env_file
                .into_iter()
                .flat_map(EnvFile::into_list)
                .map(ShortOrLong::into_long)
                .map(|env_file::Config { path, required }| {
                    required
                        .then_some(path)
                        .ok_or_eyre("optional environment files are not supported")
                })
                .collect::<color_eyre::Result<_>>()
                .wrap_err("error converting `env_file`")?,
            env: environment.into_list().into_iter().collect(),
            expose: expose.iter().map(ToString::to_string).collect(),
            annotation: annotations.into_list().into_iter().collect(),
            group_add: group_add.into_iter().map(Into::into).collect(),
            health_cmd,
            health_interval,
            health_timeout,
            health_retries,
            health_start_period,
            health_startup_interval,
            hostname: hostname.map(Into::into),
            init,
            label: labels.into_list().into_iter().collect(),
            log_driver,
            network: network_config
                .map(network_config_try_into_network_options)
                .transpose()
                .wrap_err("error converting network configuration")?
                .unwrap_or_default(),
            pids_limit,
            publish: ports::into_short_iter(ports)
                .map(|port| {
                    port.as_ref().map(ToString::to_string).map_err(|port| {
                        eyre!("could not convert port to short syntax, port = {port:#?}")
                    })
                })
                .collect::<Result<_, _>>()
                .wrap_err("error converting `ports`")?,
            pull: pull_policy
                .map(TryInto::try_into)
                .transpose()
                .wrap_err("error converting `pull_policy`")?,
            read_only,
            secret: secrets
                .into_iter()
                .map(secret_try_into_short)
                .collect::<Result<_, _>>()
                .wrap_err("error converting `secrets`")?,
            shm_size: shm_size.as_ref().map(ToString::to_string),
            stop_timeout: stop_grace_period.as_ref().map(Duration::as_secs),
            sysctl: sysctls.into_list().into_iter().collect(),
            tmpfs,
            ulimit: ulimits
                .into_iter()
                .map(ulimit_try_into_short)
                .collect::<Result<_, _>>()
                .wrap_err("error converting `ulimits`")?,
            user: user.map(Into::into),
            userns,
            volume,
            workdir: working_dir.map(Into::into),
            ..Self::default()
        })
    }
}

/// Healthcheck options used in [`QuadletOptions`].
///
/// Used for converting from [`compose_spec::service::Healthcheck`].
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Clone, PartialEq)]
struct Healthcheck {
    health_cmd: Option<String>,
    health_interval: Option<String>,
    health_timeout: Option<String>,
    health_retries: Option<u64>,
    health_start_period: Option<String>,
    health_startup_interval: Option<String>,
}

impl TryFrom<service::Healthcheck> for Healthcheck {
    type Error = color_eyre::Report;

    fn try_from(value: service::Healthcheck) -> Result<Self, Self::Error> {
        use service::healthcheck::{Command, Healthcheck, Test};

        match value {
            Healthcheck::Command(Command {
                test,
                interval,
                timeout,
                retries,
                start_period,
                start_interval,
                extensions,
            }) => {
                ensure!(
                    extensions.is_empty(),
                    "compose extensions are not supported"
                );
                Ok(Self {
                    health_cmd: test
                        .map(|test| match test {
                            Test::Command(command) => serde_json::to_string(&command)
                                .wrap_err("error serializing healthcheck test command as JSON"),
                            Test::ShellCommand(command) => Ok(command),
                        })
                        .transpose()?,
                    health_interval: interval.map(duration::to_string),
                    health_timeout: timeout.map(duration::to_string),
                    health_retries: retries,
                    health_start_period: start_period.map(duration::to_string),
                    health_startup_interval: start_interval.map(duration::to_string),
                })
            }
            Healthcheck::Disable => Ok(Self {
                health_cmd: Some("none".to_owned()),
                ..Self::default()
            }),
        }
    }
}

/// Attempt to convert a volume from a [`compose_spec::Service`] into a form suitable for
/// `podman run --volume` or `podman run --tmpfs`.
///
/// [`Tmpfs`] volumes will be converted, added to `tmpfs`, and [`None`] is returned.
///
/// # Errors
///
/// Returns an error if the volume is not compatible with `podman run --volume` or
/// `podman run --tmpfs`.
fn volume_try_into_short(
    volume: ShortOrLong<ShortVolume, volumes::Mount>,
    tmpfs: &mut Vec<String>,
) -> color_eyre::Result<Option<Volume>> {
    match volume {
        ShortOrLong::Short(volume) => Ok(Some(volume.into())),
        ShortOrLong::Long(volumes::Mount::Volume(mount)) => mount.try_into().map(Some),
        ShortOrLong::Long(volumes::Mount::Bind(mount)) => mount.try_into().map(Some),
        ShortOrLong::Long(volumes::Mount::Tmpfs(mount)) => {
            let mount = tmpfs_try_into_short(mount).wrap_err("error converting tmpfs volume")?;
            tmpfs.push(mount);
            Ok(None)
        }
        ShortOrLong::Long(volumes::Mount::NamedPipe(_)) => {
            Err(eyre!("`npipe` type volumes are not supported"))
        }
        ShortOrLong::Long(volumes::Mount::Cluster(_)) => {
            Err(eyre!("`cluster` type volumes are not supported"))
        }
    }
}

/// Attempt to convert a [`Tmpfs`] volume from a [`compose_spec::Service`] into a form suitable for
/// `podman run --tmpfs`.
///
/// # Errors
///
/// Returns an error if the [`Tmpfs`] is not compatible with `podman run --tmpfs`.
fn tmpfs_try_into_short(
    Tmpfs {
        tmpfs,
        common:
            Common {
                target,
                read_only,
                consistency,
                extensions,
            },
    }: Tmpfs,
) -> color_eyre::Result<String> {
    ensure!(
        consistency.is_none(),
        "`consistency` volume option is not supported"
    );
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let TmpfsOptions {
        size,
        mode,
        extensions,
    } = tmpfs.unwrap_or_default();

    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let mut tmpfs = target.as_path().display().to_string();

    let options = read_only
        .then(|| "ro".to_owned())
        .into_iter()
        .chain(size.map(|size| format!("size={size}")))
        .chain(mode.map(|mode| format!("mode={mode:o}")));

    let mut first = true;
    for option in options {
        let separator = if first {
            first = false;
            ':'
        } else {
            ','
        };
        tmpfs.push(separator);
        tmpfs.push_str(&option);
    }

    Ok(tmpfs)
}

/// Attempt to convert a compose service [`NetworkConfig`] into network options for the `network`
/// field of [`QuadletOptions`].
///
/// # Errors
///
/// Returns an error if an option is not supported by `podman run --network`.
fn network_config_try_into_network_options(
    network_config: NetworkConfig,
) -> color_eyre::Result<Vec<String>> {
    match network_config {
        NetworkConfig::NetworkMode(network_mode) => {
            validate_network_mode(network_mode).map(|network_mode| vec![network_mode])
        }
        NetworkConfig::Networks(networks) => networks
            .into_long()
            .into_iter()
            .map(|(identifier, options)| {
                let mut network = String::from(identifier.clone());
                network.push_str(".network");
                if let Some(options) = options {
                    let options = network_options(options).wrap_err_with(|| {
                        format!("error converting `{identifier}` network options")
                    })?;
                    network.push(':');
                    network.push_str(&options);
                }
                Ok(network)
            })
            .collect(),
    }
}

/// Validate a compose service [`NetworkMode`] for use in [`QuadletOptions`].
///
/// # Errors
///
/// Returns an error if the given `network_mode` is not supported by `podman run --network`.
fn validate_network_mode(network_mode: NetworkMode) -> color_eyre::Result<String> {
    match network_mode {
        NetworkMode::None | NetworkMode::Host => Ok(network_mode.to_string()),
        NetworkMode::Service(_) => Err(eyre!("network_mode `service:` is not supported")
            .suggestion("try using the `container:` network_mode instead")),
        NetworkMode::Other(s) => {
            if s.starts_with("bridge")
                || s.starts_with("container")
                || s.starts_with("ns:")
                || s == "private"
                || s.starts_with("slirp4netns")
                || s.starts_with("pasta")
            {
                Ok(s)
            } else {
                Err(eyre!("network_mode `{s}` is not supported by Podman"))
            }
        }
    }
    .with_suggestion(|| {
        format!(
            "see the --network section of the {}(1) documentation for supported values: \
                https://docs.podman.io/en/stable/markdown/podman-run.1.html#network-mode-net",
            "podman-run".bold()
        )
    })
}

/// Convert compose service [`Network`] options into a comma (,) separated list of key value pairs.
///
/// # Errors
///
/// Returns an error if an option not supported by `podman run --network` is used.
fn network_options(
    Network {
        aliases,
        ipv4_address,
        ipv6_address,
        link_local_ips,
        mac_address,
        priority,
        extensions,
    }: Network,
) -> color_eyre::Result<String> {
    ensure!(
        link_local_ips.is_empty(),
        "network `link_local_ips` option is not supported"
    );
    ensure!(
        priority.is_none(),
        "network `priority` option is not supported"
    );
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    let ip_addrs = ipv4_address
        .map(IpAddr::from)
        .into_iter()
        .chain(ipv6_address.map(IpAddr::from))
        .map(|ip_addr| format!("ip={ip_addr}"));

    let options: Vec<_> = aliases
        .into_iter()
        .map(|alias| format!("alias={alias}"))
        .chain(ip_addrs)
        .chain(mac_address.map(|mac| format!("mac={mac}")))
        .collect();

    Ok(options.join(","))
}

/// Attempt to convert a secret from a [`compose_spec::Service`] into a form suitable for
/// `podman run --secret`.
///
/// # Errors
///
/// Returns an error if the secret has extensions.
fn secret_try_into_short(
    secret: ShortOrLong<Identifier, ConfigOrSecret>,
) -> color_eyre::Result<String> {
    match secret {
        ShortOrLong::Short(secret) => Ok(secret.into()),
        ShortOrLong::Long(ConfigOrSecret {
            source,
            target,
            uid,
            gid,
            mode,
            extensions,
        }) => {
            ensure!(
                extensions.is_empty(),
                "compose extensions are not supported"
            );

            Ok(iter::once(source.into())
                .chain(target.map(|target| format!("target={}", target.display())))
                .chain(uid.map(|uid| format!("uid={uid}")))
                .chain(gid.map(|gid| format!("gid={gid}")))
                .chain(mode.map(|mode| format!("mode={mode:o}")))
                .collect::<Vec<_>>()
                .join(","))
        }
    }
}

/// Attempt to convert a ulimit from a [`compose_spec::Service`] into a form suitable for
/// `podman run --ulimit`.
///
/// # Errors
///
/// Returns an error if the [`Ulimit`] has extensions.
fn ulimit_try_into_short(
    (resource, ulimit): (service::Resource, ShortOrLong<u64, Ulimit>),
) -> color_eyre::Result<String> {
    match ulimit {
        ShortOrLong::Short(ulimit) => Ok(format!("{resource}={ulimit}")),
        ShortOrLong::Long(Ulimit {
            soft,
            hard,
            extensions,
        }) => {
            ensure!(
                extensions.is_empty(),
                "compose extensions are not supported"
            );
            Ok(format!("{resource}={soft}:{hard}"))
        }
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
