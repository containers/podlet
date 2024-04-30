//! Types for splitting up a [`compose_spec::Service`] into parts and constructing a
//! [`Container`](super::Container).

use std::{net::IpAddr, time::Duration};

use color_eyre::eyre::ensure;
use compose_spec::{
    service::{
        build::Context, device::CgroupRule, AbsolutePath, BlkioConfig, Build, ByteValue, Cgroup,
        Command, ConfigOrSecret, CpuSet, Cpus, CredentialSpec, Deploy, Develop, Device, EnvFile,
        Expose, Extends, Healthcheck, Hostname, Image, Ipc, Limit, Link, Logging, MacAddress,
        NetworkConfig, OomScoreAdj, Percent, Platform, Ports, PullPolicy, Ulimits, UserOrGroup,
        Uts, Volumes, VolumesFrom,
    },
    Extensions, Identifier, ItemOrList, ListOrMap, MapKey, ShortOrLong, StringOrNumber,
};
use indexmap::{IndexMap, IndexSet};

/// A struct for splitting up a [`compose_spec::Service`] into parts used to construct a
/// [`Container`](super::Container).
pub struct Service {
    pub unsupported: Unsupported,
    pub quadlet: Quadlet,
    pub podman_args: PodmanArgs,
    pub container: Container,
}

impl From<compose_spec::Service> for Service {
    fn from(
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
            // Taken in `crate::cli::compose::service_try_into_quadlet_file()`.
            depends_on: _,
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
            // Taken for the `[Service]` section.
            restart: _,
            // Taken in `crate::cli::GlobalArgs::from_compose()`.
            runtime: _,
            scale,
            secrets,
            security_opt,
            shm_size,
            stdin_open,
            stop_grace_period,
            stop_signal,
            // Taken in `crate::cli::GlobalArgs::from_compose()`.
            storage_opt: _,
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
        let Logging {
            driver: log_driver,
            options: log_options,
            extensions: logging_extensions,
        } = logging.unwrap_or_default();

        Self {
            unsupported: Unsupported {
                attach,
                build,
                cpu_count,
                cpu_percent,
                configs,
                credential_spec,
                deploy,
                develop,
                domain_name,
                extends,
                external_links,
                isolation,
                links,
                logging_extensions,
                memswap_limit,
                profiles,
                scale,
                volumes_from,
                extensions,
            },
            quadlet: Quadlet {
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
                sysctls,
                tmpfs,
                ulimits,
                user,
                userns_mode,
                volumes,
                working_dir,
            },
            podman_args: PodmanArgs {
                blkio_config,
                cpu_shares,
                cpu_period,
                cpu_quota,
                cpu_rt_runtime,
                cpu_rt_period,
                cpus,
                cpuset,
                cgroup,
                cgroup_parent,
                device_cgroup_rules,
                extra_hosts,
                group_add,
                ipc,
                uts,
                log_options,
                mac_address,
                mem_limit,
                mem_reservation,
                mem_swappiness,
                oom_kill_disable,
                oom_score_adj,
                pid,
                platform,
                privileged,
                stdin_open,
                stop_grace_period,
                stop_signal,
                tty,
            },
            container: Container {
                command,
                image,
                security_opt,
            },
        }
    }
}

/// Fields taken from a [`compose_spec::Service`] which are not supported.
pub struct Unsupported {
    attach: bool,
    build: Option<ShortOrLong<Context, Build>>,
    cpu_count: Option<u64>,
    cpu_percent: Option<Percent>,
    configs: Vec<ShortOrLong<Identifier, ConfigOrSecret>>,
    credential_spec: Option<CredentialSpec>,
    deploy: Option<Deploy>,
    develop: Option<Develop>,
    domain_name: Option<Hostname>,
    extends: Option<Extends>,
    external_links: IndexSet<Link>,
    isolation: Option<String>,
    links: IndexSet<Link>,
    logging_extensions: Extensions,
    memswap_limit: Option<Limit<ByteValue>>,
    profiles: IndexSet<Identifier>,
    scale: Option<u64>,
    volumes_from: IndexSet<VolumesFrom>,
    extensions: Extensions,
}

impl Unsupported {
    /// Ensure that all unsupported fields are [`None`] or empty.
    ///
    /// # Errors
    ///
    /// Returns an error if a field is not empty.
    pub fn ensure_empty(&self) -> color_eyre::Result<()> {
        let Self {
            attach,
            build,
            cpu_count,
            cpu_percent,
            configs,
            credential_spec,
            deploy,
            develop,
            domain_name,
            extends,
            external_links,
            isolation,
            links,
            logging_extensions,
            memswap_limit,
            profiles,
            scale,
            volumes_from,
            extensions,
        } = self;

        let unsupported_options = [
            // `attach` default is `true`.
            ("attach", *attach),
            ("build", build.is_none()),
            ("cpu_count", cpu_count.is_none()),
            ("cpu_percent", cpu_percent.is_none()),
            ("configs", configs.is_empty()),
            ("credential_spec", credential_spec.is_none()),
            ("deploy", deploy.is_none()),
            ("develop", develop.is_none()),
            ("domain_name", domain_name.is_none()),
            ("extends", extends.is_none()),
            ("external_links", external_links.is_empty()),
            ("isolation", isolation.is_none()),
            ("links", links.is_empty()),
            ("memswap_limit", memswap_limit.is_none()),
            ("profiles", profiles.is_empty()),
            ("scale", scale.is_none()),
            ("volumes_from", volumes_from.is_empty()),
        ];

        for (option, not_present) in unsupported_options {
            ensure!(not_present, "`{option}` is not supported");
        }

        ensure!(
            logging_extensions.is_empty() && extensions.is_empty(),
            "compose extensions are not supported"
        );

        Ok(())
    }
}

/// Fields taken from a [`compose_spec::Service`] for constructing a [`super::QuadletOptions`].
pub struct Quadlet {
    pub cap_add: IndexSet<String>,
    pub cap_drop: IndexSet<String>,
    pub container_name: Option<Identifier>,
    pub devices: IndexSet<Device>,
    pub dns: Option<ItemOrList<IpAddr>>,
    pub dns_opt: IndexSet<String>,
    pub dns_search: Option<ItemOrList<Hostname>>,
    pub entrypoint: Option<Command>,
    pub env_file: Option<EnvFile>,
    pub environment: ListOrMap,
    pub expose: IndexSet<Expose>,
    pub annotations: ListOrMap,
    pub healthcheck: Option<Healthcheck>,
    pub hostname: Option<Hostname>,
    pub init: bool,
    pub labels: ListOrMap,
    pub log_driver: Option<String>,
    pub network_config: Option<NetworkConfig>,
    pub pids_limit: Option<Limit<u32>>,
    pub ports: Ports,
    pub pull_policy: Option<PullPolicy>,
    pub read_only: bool,
    pub secrets: Vec<ShortOrLong<Identifier, ConfigOrSecret>>,
    pub shm_size: Option<ByteValue>,
    pub sysctls: ListOrMap,
    pub tmpfs: Option<ItemOrList<AbsolutePath>>,
    pub ulimits: Ulimits,
    pub user: Option<UserOrGroup>,
    pub userns_mode: Option<String>,
    pub volumes: Volumes,
    pub working_dir: Option<AbsolutePath>,
}

/// Fields taken from a [`compose_spec::Service`] for constructing a [`super::PodmanArgs`].
#[allow(clippy::struct_excessive_bools)]
pub struct PodmanArgs {
    pub blkio_config: Option<BlkioConfig>,
    pub cpu_shares: Option<u64>,
    pub cpu_period: Option<Duration>,
    pub cpu_quota: Option<Duration>,
    pub cpu_rt_runtime: Option<Duration>,
    pub cpu_rt_period: Option<Duration>,
    pub cpus: Option<Cpus>,
    pub cpuset: CpuSet,
    pub cgroup: Option<Cgroup>,
    pub cgroup_parent: Option<String>,
    pub device_cgroup_rules: IndexSet<CgroupRule>,
    pub extra_hosts: IndexMap<Hostname, IpAddr>,
    pub group_add: IndexSet<UserOrGroup>,
    pub ipc: Option<Ipc>,
    pub uts: Option<Uts>,
    pub log_options: IndexMap<MapKey, Option<StringOrNumber>>,
    pub mac_address: Option<MacAddress>,
    pub mem_limit: Option<ByteValue>,
    pub mem_reservation: Option<ByteValue>,
    pub mem_swappiness: Option<Percent>,
    pub oom_kill_disable: bool,
    pub oom_score_adj: Option<OomScoreAdj>,
    pub pid: Option<String>,
    pub platform: Option<Platform>,
    pub privileged: bool,
    pub stdin_open: bool,
    pub stop_grace_period: Option<Duration>,
    pub stop_signal: Option<String>,
    pub tty: bool,
}

/// Fields taken from a [`compose_spec::Service`] for constructing the top-level fields in
/// [`super::Container`].
pub struct Container {
    pub command: Option<Command>,
    pub image: Option<Image>,
    pub security_opt: IndexSet<String>,
}
