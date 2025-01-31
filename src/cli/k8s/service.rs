//! [`Service`] is created from a [`compose_spec::Service`] and then added to a [`PodSpec`].

mod mount;

use std::{collections::BTreeMap, net::IpAddr, time::Duration};

use color_eyre::{
    eyre::{ensure, eyre, OptionExt, WrapErr},
    Section,
};
use compose_spec::{
    service::{
        build::Context,
        device::CgroupRule,
        healthcheck::{self, Test},
        ports::{self, Port, Protocol},
        AbsolutePath, BlkioConfig, Build, ByteValue, Cgroup, Command, ConfigOrSecret, CpuSet, Cpus,
        CredentialSpec, DependsOn, Deploy, Develop, Device, EnvFile, Expose, Extends, Healthcheck,
        Hostname, IdOrName, Image, Ipc, Limit, Link, Logging, MacAddress, NetworkConfig,
        OomScoreAdj, Percent, Platform, Ports, PullPolicy, Restart, Ulimits, User, Uts, Volumes,
        VolumesFrom,
    },
    Extensions, Identifier, ItemOrList, ListOrMap, Map, ShortOrLong,
};
use indexmap::{IndexMap, IndexSet};
use k8s_openapi::{
    api::core::v1::{
        Capabilities, Container, ContainerPort, EnvVar, ExecAction, PodSpec, Probe,
        ResourceRequirements, SELinuxOptions, SecurityContext,
    },
    apimachinery::pkg::api::resource::Quantity,
};

use crate::cli::{
    compose::command_try_into_vec,
    container::security_opt::{LabelOpt, SecurityOpt},
};

use self::mount::tmpfs_and_volumes_try_into_volume_mounts;

/// Fields from a [`compose_spec::Service`] which will be [added](Service::add_to_pod_spec()) to a
/// [`PodSpec`]'s [`Container`]s and [`Volume`](k8s_openapi::api::core::v1::Volume)s.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct Service {
    unsupported: Unsupported,
    build: Option<ShortOrLong<Context, Build>>,
    name: Identifier,
    resources: ContainerResources,
    security_context: ContainerSecurityContext,
    command: Option<Command>,
    entrypoint: Option<Command>,
    environment: ListOrMap,
    healthcheck: Option<Healthcheck>,
    image: Option<Image>,
    ports: Ports,
    pull_policy: Option<PullPolicy>,
    stdin_open: bool,
    tmpfs: Option<ItemOrList<AbsolutePath>>,
    tty: bool,
    volumes: Volumes,
    working_dir: Option<AbsolutePath>,
}

impl Service {
    /// Create a [`Service`] from a `name` [`Identifier`] and a [`compose_spec::Service`].
    pub(super) fn from_compose(
        name: &Identifier,
        compose_spec::Service {
            attach,
            build,
            blkio_config,
            cpu_count,
            cpu_percent,
            cpu_shares,
            cpu_period,
            cpu_quota,
            cpu_rt_runtime,
            cpu_rt_period,
            cpus,
            cpuset,
            cap_add,
            cap_drop,
            cgroup,
            cgroup_parent,
            command,
            configs,
            container_name,
            credential_spec,
            depends_on,
            deploy,
            develop,
            device_cgroup_rules,
            devices,
            dns,
            dns_opt,
            dns_search,
            domain_name,
            entrypoint,
            env_file,
            environment,
            expose,
            extends,
            annotations,
            external_links,
            extra_hosts,
            group_add,
            healthcheck,
            hostname,
            image,
            init,
            ipc,
            uts,
            isolation,
            labels,
            links,
            logging,
            network_config,
            mac_address,
            mem_limit,
            mem_reservation,
            mem_swappiness,
            memswap_limit,
            oom_kill_disable,
            oom_score_adj,
            pid,
            pids_limit,
            platform,
            ports,
            privileged,
            profiles,
            pull_policy,
            read_only,
            restart,
            runtime,
            scale,
            secrets,
            security_opt,
            shm_size,
            stdin_open,
            stop_grace_period,
            stop_signal,
            storage_opt,
            sysctls,
            tmpfs,
            tty,
            ulimits,
            user,
            userns_mode,
            volumes,
            volumes_from,
            working_dir,
            extensions,
        }: compose_spec::Service,
    ) -> Self {
        Self {
            unsupported: Unsupported {
                attach,
                blkio_config,
                cpu_count,
                cpu_percent,
                cpu_shares,
                cpu_period,
                cpu_quota,
                cpu_rt_runtime,
                cpu_rt_period,
                cpuset,
                cgroup,
                cgroup_parent,
                configs,
                credential_spec,
                depends_on,
                deploy,
                develop,
                device_cgroup_rules,
                devices,
                dns,
                dns_opt,
                dns_search,
                domain_name,
                env_file,
                expose,
                extends,
                annotations,
                external_links,
                extra_hosts,
                group_add,
                hostname,
                init,
                ipc,
                uts,
                isolation,
                labels,
                links,
                logging,
                network_config,
                mac_address,
                mem_swappiness,
                memswap_limit,
                oom_kill_disable,
                oom_score_adj,
                pid,
                pids_limit,
                platform,
                profiles,
                restart,
                runtime,
                scale,
                secrets,
                shm_size,
                stop_grace_period,
                stop_signal,
                storage_opt,
                sysctls,
                ulimits,
                userns_mode,
                volumes_from,
                extensions,
            },
            name: container_name.unwrap_or_else(|| name.clone()),
            resources: ContainerResources {
                cpus,
                mem_limit,
                mem_reservation,
            },
            security_context: ContainerSecurityContext {
                cap_add,
                cap_drop,
                privileged,
                read_only,
                security_opt,
                user,
            },
            build,
            command,
            entrypoint,
            environment,
            healthcheck,
            image,
            ports,
            pull_policy,
            stdin_open,
            tmpfs,
            tty,
            volumes,
            working_dir,
        }
    }

    /// Add the service to a [`PodSpec`]'s [`Container`]s and [`Volume`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if an unsupported option was used or conversion of one of the fields fails.
    pub(super) fn add_to_pod_spec(self, spec: &mut PodSpec) -> color_eyre::Result<()> {
        let Self {
            unsupported,
            name,
            resources,
            security_context,
            command,
            entrypoint,
            environment,
            healthcheck,
            image,
            ports,
            pull_policy,
            stdin_open,
            tmpfs,
            tty,
            volumes,
            working_dir,
            build
        } = self;

        unsupported.ensure_empty()?;

        let volume_mounts =
            tmpfs_and_volumes_try_into_volume_mounts(tmpfs, volumes, &name, &mut spec.volumes)
                // converting `tmpfs` always succeeds
                .wrap_err("error converting `volumes`")?;

        let container_image: String;
        if build.is_some() {
            container_image = match build.unwrap() {
                ShortOrLong::Short(build) => Some(build),
                ShortOrLong::Long(build) => build.context,
            }.unwrap().into_string().unwrap();
        } else {
            container_image = image.ok_or_eyre("`image` is required")?.into_inner();
        }

        spec.containers.push(Container {
            name: name.into(),
            resources: resources.into_resource_requirements(),
            security_context: security_context.try_into_security_context()?,
            args: command
                .map(command_try_into_vec)
                .transpose()
                .wrap_err("error converting `command` to `args`")?,
            command: entrypoint.map(|entrypoint| match entrypoint {
                Command::String(entrypoint) => {
                    vec!["/bin/sh".to_owned(), "-c".to_owned(), entrypoint]
                }
                Command::List(entrypoint) => entrypoint,
            }),
            env: (!environment.is_empty())
                .then(|| {
                    environment.into_map().map(|environment| {
                        environment
                            .into_iter()
                            .map(|(name, value)| EnvVar {
                                name: name.into(),
                                value: value.map(Into::into),
                                value_from: None,
                            })
                            .collect()
                    })
                })
                .transpose()
                .wrap_err("error converting `environment`")?,
            liveness_probe: healthcheck
                .and_then(|healthcheck| match healthcheck {
                    Healthcheck::Command(command) => {
                        Some(healthcheck_command_try_into_probe(command))
                    }
                    // container image healthchecks are disabled by default in k8s
                    Healthcheck::Disable => None,
                })
                .transpose()
                .wrap_err("error converting `healthcheck`")?,
            image: Some(container_image),
            ports: (!ports.is_empty())
                .then(|| {
                    ports::into_long_iter(ports)
                        .map(port_try_into_container_port)
                        .collect()
                })
                .transpose()
                .wrap_err("error converting `ports`")?,
            image_pull_policy: pull_policy
                .map(|pull_policy| match pull_policy {
                    PullPolicy::Always => Ok("Always".to_owned()),
                    PullPolicy::Never => Ok("Never".to_owned()),
                    PullPolicy::Missing => Ok("IfNotPreset".to_owned()),
                    PullPolicy::Build => Err(eyre!("`build` is not supported")),
                })
                .transpose()
                .wrap_err("error converting `pull_policy`")?,
            stdin: stdin_open.then_some(true),
            tty: tty.then_some(true),
            volume_mounts: (!volume_mounts.is_empty()).then_some(volume_mounts),
            working_dir: working_dir
                .map(|path| {
                    path.into_inner()
                        .into_os_string()
                        .into_string()
                        .map_err(|_| eyre!("`working_dir` must contain only valid UTF-8"))
                })
                .transpose()?,
            ..Container::default()
        });

        Ok(())
    }
}

/// Attempt to convert a [`compose_spec::Service`]'s [`healthcheck::Command`] into a Kubernetes
/// [`Probe`] for use in the `liveness_probe` field of [`Container`].
///
/// # Errors
///
/// Returns an error if extensions are present or there was an error converting one of the
/// [`Duration`]s into seconds.
fn healthcheck_command_try_into_probe(
    healthcheck::Command {
        test,
        interval,
        timeout,
        retries,
        start_period,
        // k8s doesn't run probes during startup
        start_interval: _,
        extensions,
    }: healthcheck::Command,
) -> color_eyre::Result<Probe> {
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    Ok(Probe {
        exec: test.map(|test| ExecAction {
            command: Some(match test {
                Test::Command(test) => test,
                Test::ShellCommand(test) => vec!["/bin/sh".to_owned(), "-c".to_owned(), test],
            }),
        }),
        period_seconds: interval
            .map(duration_round_seconds)
            .map(TryInto::try_into)
            .transpose()
            .wrap_err("error converting `interval`")?,
        timeout_seconds: Some(
            timeout
                .map(duration_round_seconds)
                .map(TryInto::try_into)
                .transpose()
                .wrap_err("error converting `timeout`")?
                // default timeout for compose is 30 seconds, for k8s its 1 second
                .unwrap_or(30),
        ),
        failure_threshold: retries
            .map(TryInto::try_into)
            .transpose()
            .wrap_err("error converting `retries`")?,
        initial_delay_seconds: start_period
            .map(duration_round_seconds)
            .map(TryInto::try_into)
            .transpose()
            .wrap_err("error converting `start_period`")?,
        ..Probe::default()
    })
}

/// Round a [`Duration`] to the nearest whole seconds with a minimum of 1 second.
fn duration_round_seconds(duration: Duration) -> u64 {
    let mut secs = duration.as_secs();
    // rounding
    if duration.subsec_micros() >= 500_000 {
        secs += 1;
    }
    // floor of 1
    secs.max(1)
}

/// Attempt to convert a [`compose_spec::Service`]'s [`Port`] into a Kubernetes [`ContainerPort`].
///
/// # Errors
///
/// Returns an error if an unsupported option is used.
fn port_try_into_container_port(
    Port {
        name,
        target,
        published,
        host_ip,
        protocol,
        app_protocol,
        mode,
        extensions,
    }: Port,
) -> color_eyre::Result<ContainerPort> {
    ensure!(app_protocol.is_none(), "`app_protocol` is not supported");
    ensure!(mode.is_none(), "`mode` is not supported");
    ensure!(
        extensions.is_empty(),
        "compose extensions are not supported"
    );

    Ok(ContainerPort {
        name,
        container_port: target.into(),
        host_port: published
            .map(|range| {
                ensure!(
                    range.size() == 1,
                    "Kubernetes only supports publishing to a single, specific, host port, \
                        as apposed to one of a range of ports"
                );
                Ok(range.start().into())
            })
            .transpose()
            .wrap_err("error converting `published`")?,
        host_ip: host_ip.as_ref().map(ToString::to_string),
        protocol: protocol
            .map(|protocol| match protocol {
                Protocol::Tcp => Ok("TCP".to_owned()),
                Protocol::Udp => Ok("UDP".to_owned()),
                Protocol::Other(mut protocol) => {
                    protocol.make_ascii_uppercase();
                    ensure!(
                        protocol == "SCTP",
                        "only `UDP`, `TCP`, and `SCTP` are supported"
                    );
                    Ok(protocol)
                }
            })
            .transpose()
            .wrap_err("error converting `protocol`")?,
    })
}

/// Fields from a [`compose_spec::Service`] which are converted into a [`Container`]'s
/// [`ResourceRequirements`].
struct ContainerResources {
    cpus: Option<Cpus>,
    mem_limit: Option<ByteValue>,
    mem_reservation: Option<ByteValue>,
}

impl ContainerResources {
    /// Convert into [`ResourceRequirements`] for a Kubernetes [`Container`].
    ///
    /// Returns [`None`] if no resource options are set.
    fn into_resource_requirements(self) -> Option<ResourceRequirements> {
        let Self {
            cpus,
            mem_limit,
            mem_reservation,
        } = self;

        let mut resources = None;

        if let Some(cpus) = cpus {
            resources
                .get_or_insert_with(ResourceRequirements::default)
                .limits
                .get_or_insert_with(BTreeMap::default)
                .insert("cpu".to_owned(), Quantity(cpus.into_inner().to_string()));
        }

        if let Some(mem_limit) = mem_limit {
            resources
                .get_or_insert_with(ResourceRequirements::default)
                .limits
                .get_or_insert_with(BTreeMap::default)
                .insert("memory".to_owned(), Quantity(mem_limit.to_string()));
        }

        if let Some(mem_reservation) = mem_reservation {
            resources
                .get_or_insert_with(ResourceRequirements::default)
                .requests
                .get_or_insert_with(BTreeMap::default)
                .insert("memory".to_owned(), Quantity(mem_reservation.to_string()));
        }

        resources
    }
}

/// Fields from a [`compose_spec::Service`] which are converted into a [`Container`]'s
/// [`SecurityContext`].
struct ContainerSecurityContext {
    cap_add: IndexSet<String>,
    cap_drop: IndexSet<String>,
    privileged: bool,
    read_only: bool,
    security_opt: IndexSet<String>,
    user: Option<User>,
}

impl ContainerSecurityContext {
    /// Attempt to convert into [`SecurityContext`] for a Kubernetes [`Container`].
    ///
    /// Returns [`None`] if no security context options are set.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion of one of the fields fails.
    fn try_into_security_context(self) -> color_eyre::Result<Option<SecurityContext>> {
        let Self {
            cap_add,
            cap_drop,
            privileged,
            read_only,
            security_opt,
            user,
        } = self;

        let mut security_context = None;

        if !cap_add.is_empty() {
            security_context
                .get_or_insert_with(SecurityContext::default)
                .capabilities
                .get_or_insert_with(Capabilities::default)
                .add = Some(cap_add.into_iter().collect());
        }

        if !cap_drop.is_empty() {
            security_context
                .get_or_insert_with(SecurityContext::default)
                .capabilities
                .get_or_insert_with(Capabilities::default)
                .drop = Some(cap_drop.into_iter().collect());
        }

        if privileged {
            security_context
                .get_or_insert_with(SecurityContext::default)
                .privileged = Some(true);
        }

        if read_only {
            security_context
                .get_or_insert_with(SecurityContext::default)
                .read_only_root_filesystem = Some(true);
        }

        if !security_opt.is_empty() {
            let se_linux_options = security_opt_try_into_selinux_options(security_opt)
                .wrap_err("error converting `security_opt`")?;
            security_context
                .get_or_insert_with(SecurityContext::default)
                .se_linux_options = Some(se_linux_options);
        }

        if let Some(User { user, group }) = user {
            let user = user
                .as_id()
                .ok_or_eyre("only numeric UIDs are supported for `user`")?
                .into();
            let group = group
                .map(|group| {
                    group
                        .as_id()
                        .ok_or_eyre("only numeric GIDs are supported for `user`")
                })
                .transpose()?
                .map(Into::into);

            let security_context = security_context.get_or_insert_with(SecurityContext::default);
            security_context.run_as_user = Some(user);
            security_context.run_as_group = group;
        }

        Ok(security_context)
    }
}

/// Attempt to convert a [`compose_spec::Service`]'s `security_opt` field into [`SELinuxOptions`].
///
/// # Errors
///
/// Returns an error if an unknown or unsupported security opt is given.
fn security_opt_try_into_selinux_options(
    security_opt: IndexSet<String>,
) -> color_eyre::Result<SELinuxOptions> {
    security_opt.into_iter().try_fold(
        SELinuxOptions::default(),
        |mut selinux_options, security_opt| {
            let security_opt = if security_opt == "no-new-privileges:true" {
                SecurityOpt::NoNewPrivileges
            } else if security_opt == "no-new-privileges:false" {
                return Ok(selinux_options);
            } else {
                security_opt.replacen(':', "=", 1).parse()?
            };

            match security_opt {
                SecurityOpt::Apparmor(_) => Err(eyre!("`apparmor` security_opt is not supported")),
                SecurityOpt::Label(label_opt) => match label_opt {
                    LabelOpt::User(user) => {
                        selinux_options.user = Some(user);
                        Ok(selinux_options)
                    }
                    LabelOpt::Role(role) => {
                        selinux_options.role = Some(role);
                        Ok(selinux_options)
                    }
                    LabelOpt::Type(kind) => {
                        selinux_options.type_ = Some(kind);
                        Ok(selinux_options)
                    }
                    LabelOpt::Level(level) => {
                        selinux_options.level = Some(level);
                        Ok(selinux_options)
                    }
                    LabelOpt::Filetype(_) => {
                        Err(eyre!("`label:filetype` security_opt is not supported"))
                    }
                    LabelOpt::Disable => {
                        Err(eyre!("`label:disable` security_opt is not supported"))
                    }
                    LabelOpt::Nested => Err(eyre!("`label:nested` security_opt is not supported")),
                },
                SecurityOpt::Mask(_) => Err(eyre!("`mask` security_opt is not supported")),
                SecurityOpt::NoNewPrivileges => {
                    Err(eyre!("`no-new-privileges` security_opt is not supported"))
                }
                SecurityOpt::Seccomp(_) => Err(eyre!("`seccomp` security_opt is not supported")),
                SecurityOpt::ProcOpts(_) => Err(eyre!("`proc-opts` security_opt is not supported")),
                SecurityOpt::Unmask(_) => Err(eyre!("`unmask` security_opt is not supported")),
            }
        },
    )
}

/// Fields taken from a [`compose_spec::Service`] which are not supported for Kubernetes pod
/// [`Container`]s.
struct Unsupported {
    attach: bool,
    blkio_config: Option<BlkioConfig>,
    cpu_count: Option<u64>,
    cpu_percent: Option<Percent>,
    cpu_shares: Option<u64>,
    cpu_period: Option<Duration>,
    cpu_quota: Option<Duration>,
    cpu_rt_runtime: Option<Duration>,
    cpu_rt_period: Option<Duration>,
    cpuset: CpuSet,
    cgroup: Option<Cgroup>,
    cgroup_parent: Option<String>,
    configs: Vec<ShortOrLong<Identifier, ConfigOrSecret>>,
    credential_spec: Option<CredentialSpec>,
    depends_on: DependsOn,
    deploy: Option<Deploy>,
    develop: Option<Develop>,
    device_cgroup_rules: IndexSet<CgroupRule>,
    devices: IndexSet<Device>,
    dns: Option<ItemOrList<IpAddr>>,
    dns_opt: IndexSet<String>,
    dns_search: Option<ItemOrList<Hostname>>,
    domain_name: Option<Hostname>,
    env_file: Option<EnvFile>,
    expose: IndexSet<Expose>,
    extends: Option<Extends>,
    annotations: ListOrMap,
    external_links: IndexSet<Link>,
    extra_hosts: IndexMap<Hostname, IpAddr>,
    group_add: IndexSet<IdOrName>,
    hostname: Option<Hostname>,
    init: bool,
    ipc: Option<Ipc>,
    uts: Option<Uts>,
    isolation: Option<String>,
    labels: ListOrMap,
    links: IndexSet<Link>,
    logging: Option<Logging>,
    network_config: Option<NetworkConfig>,
    mac_address: Option<MacAddress>,
    mem_swappiness: Option<Percent>,
    memswap_limit: Option<Limit<ByteValue>>,
    oom_kill_disable: bool,
    oom_score_adj: Option<OomScoreAdj>,
    pid: Option<String>,
    pids_limit: Option<Limit<u32>>,
    platform: Option<Platform>,
    profiles: IndexSet<Identifier>,
    restart: Option<Restart>,
    runtime: Option<String>,
    scale: Option<u64>,
    secrets: Vec<ShortOrLong<Identifier, ConfigOrSecret>>,
    shm_size: Option<ByteValue>,
    stop_grace_period: Option<Duration>,
    stop_signal: Option<String>,
    storage_opt: Map,
    sysctls: ListOrMap,
    ulimits: Ulimits,
    userns_mode: Option<String>,
    volumes_from: IndexSet<VolumesFrom>,
    extensions: Extensions,
}

impl Unsupported {
    /// Ensure that all unsupported fields are [`None`] or empty.
    ///
    /// # Errors
    ///
    /// Returns an error if a field is not empty.
    #[allow(clippy::too_many_lines)]
    fn ensure_empty(&self) -> color_eyre::Result<()> {
        let Self {
            attach,
            blkio_config,
            cpu_count,
            cpu_percent,
            cpu_shares,
            cpu_period,
            cpu_quota,
            cpu_rt_runtime,
            cpu_rt_period,
            cpuset,
            cgroup,
            cgroup_parent,
            configs,
            credential_spec,
            depends_on,
            deploy,
            develop,
            device_cgroup_rules,
            devices,
            dns,
            dns_opt,
            dns_search,
            domain_name,
            env_file,
            expose,
            extends,
            annotations,
            external_links,
            extra_hosts,
            group_add,
            hostname,
            init,
            ipc,
            uts,
            isolation,
            labels,
            links,
            logging,
            network_config,
            mac_address,
            mem_swappiness,
            memswap_limit,
            oom_kill_disable,
            oom_score_adj,
            pid,
            pids_limit,
            platform,
            profiles,
            restart,
            runtime,
            scale,
            secrets,
            shm_size,
            stop_grace_period,
            stop_signal,
            storage_opt,
            sysctls,
            ulimits,
            userns_mode,
            volumes_from,
            extensions,
        } = self;

        let unsupported_options = [
            ("attach", *attach),
            ("blkio_config", blkio_config.is_none()),
            ("cpu_count", cpu_count.is_none()),
            ("cpu_percent", cpu_percent.is_none()),
            ("cpu_shares", cpu_shares.is_none()),
            ("cpu_period", cpu_period.is_none()),
            ("cpu_quota", cpu_quota.is_none()),
            ("cpu_rt_runtime", cpu_rt_runtime.is_none()),
            ("cpu_rt_period", cpu_rt_period.is_none()),
            ("cpuset", cpuset.is_empty()),
            ("cgroup", cgroup.is_none()),
            ("cgroup_parent", cgroup_parent.is_none()),
            ("configs", configs.is_empty()),
            ("credential_spec", credential_spec.is_none()),
            ("depends_on", depends_on_is_empty(depends_on)),
            ("deploy", deploy.is_none()),
            ("develop", develop.is_none()),
            ("device_cgroup_rules", device_cgroup_rules.is_empty()),
            ("domainname", domain_name.is_none()),
            ("env_file", env_file.is_none()),
            ("expose", expose.is_empty()),
            ("extends", extends.is_none()),
            ("external_links", external_links.is_empty()),
            ("group_add", group_add.is_empty()),
            ("uts", uts.is_none()),
            ("isolation", isolation.is_none()),
            ("links", links.is_empty()),
            ("logging", logging.is_none()),
            (
                "network_mode",
                !matches!(network_config, Some(NetworkConfig::NetworkMode(_))),
            ),
            (
                "networks",
                !matches!(network_config, Some(NetworkConfig::Networks(_))),
            ),
            ("mac_address", mac_address.is_none()),
            ("mem_swappiness", mem_swappiness.is_none()),
            ("memswap_limit", memswap_limit.is_none()),
            ("oom_kill_disable", !oom_kill_disable),
            ("oom_score_adj", oom_score_adj.is_none()),
            ("pids_limit", pids_limit.is_none()),
            ("platform", platform.is_none()),
            ("profiles", profiles.is_empty()),
            ("runtime", runtime.is_none()),
            ("scale", scale.is_none()),
            ("secrets", secrets.is_empty()),
            ("shm_size", shm_size.is_none()),
            ("stop_signal", stop_signal.is_none()),
            ("storage_opt", storage_opt.is_empty()),
            ("ulimits", ulimits.is_empty()),
            ("userns_mode", userns_mode.is_none()),
            ("volumes_from", volumes_from.is_empty()),
        ];
        for (option, not_present) in unsupported_options {
            ensure!(
                not_present,
                "`{option}` is not supported for Kubernetes pod containers"
            );
        }

        let pod_spec_options = [
            ("dns", dns.is_none()),
            ("dns_opt", dns_opt.is_empty()),
            ("dns_search", dns_search.is_none()),
            ("extra_hosts", extra_hosts.is_empty()),
            ("hostname", hostname.is_none()),
            ("init", !init),
            ("ipc", ipc.is_none()),
            ("pid", pid.is_none()),
            ("restart", restart.is_none()),
            ("stop_grace_period", stop_grace_period.is_none()),
            ("sysctls", sysctls.is_empty()),
        ];
        for (option, not_present) in pod_spec_options {
            if !not_present {
                return Err(eyre!(
                    "Kubernetes pods do not support per container `{option}` options",
                )
                .suggestion("try using setting the option in the pod spec instead"));
            }
        }

        let pod_metadata_options = [
            ("annotations", annotations.is_empty()),
            ("labels", labels.is_empty()),
        ];
        for (option, not_present) in pod_metadata_options {
            if !not_present {
                return Err(eyre!(
                    "Kubernetes pods do not support per container `{option}` options",
                )
                .suggestion("try using setting the option in the pod metadata instead"));
            }
        }

        if !devices.is_empty() {
            return Err(
                eyre!("Kubernetes pod containers do not directly support devices")
                    .suggestion("try using a bind mount instead"),
            );
        };

        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        Ok(())
    }
}

/// Return `true` if the [`DependsOn`] is empty.
fn depends_on_is_empty(depends_on: &DependsOn) -> bool {
    match depends_on {
        ShortOrLong::Short(depends_on) => depends_on.is_empty(),
        ShortOrLong::Long(depends_on) => depends_on.is_empty(),
    }
}
