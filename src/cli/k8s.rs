use std::collections::BTreeMap;

use color_eyre::{
    eyre::{self, Context},
    Help,
};
use docker_compose_types::{
    AdvancedVolumes, Command, Compose, ComposeVolume, Entrypoint, Environment, Healthcheck,
    HealthcheckTest, Labels, Ports, PublishedPort, Service, SingleValue, Tmpfs, Ulimit, Ulimits,
    Volumes as ComposeVolumes,
};
use indexmap::IndexMap;
use k8s_openapi::{
    api::core::v1::{
        Capabilities, Container, ContainerPort, EmptyDirVolumeSource, EnvVar, ExecAction,
        HostPathVolumeSource, PersistentVolumeClaim, PersistentVolumeClaimVolumeSource, Pod,
        PodSpec, Probe, ResourceRequirements, SELinuxOptions, SecurityContext, Volume, VolumeMount,
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};

use super::{
    compose,
    container::security_opt::{LabelOpt, SecurityOpt},
};

pub fn compose_try_into_pod(
    mut compose: Compose,
    name: String,
) -> color_eyre::Result<(Pod, Vec<PersistentVolumeClaim>)> {
    let mut volumes = Vec::new();

    let containers = compose::services(&mut compose)
        .map(|result| {
            result.and_then(|(name, service)| {
                let (container, container_volumes) =
                    service_try_into_container(name.clone(), service).wrap_err_with(|| {
                        format!("could not convert service `{name}` into k8s container spec")
                    })?;
                volumes.extend(container_volumes);
                Ok(container)
            })
        })
        .collect::<color_eyre::Result<_>>()?;

    let spec = PodSpec {
        containers,
        volumes: volumes.filter_empty(),
        ..PodSpec::default()
    };

    let pod = Pod {
        metadata: ObjectMeta {
            name: Some(name),
            ..ObjectMeta::default()
        },
        spec: Some(spec),
        status: None,
    };

    let persistent_volume_claims = compose
        .volumes
        .0
        .into_iter()
        .filter_map(|(name, volume)| {
            Option::<ComposeVolume>::from(volume).map(|volume| {
                compose_volume_try_into_persistent_volume_claim(volume, name.clone()).wrap_err_with(
                    || {
                        format!(
                            "could not convert volume `{name}` into \
                                k8s persistent volume claim spec"
                        )
                    },
                )
            })
        })
        .collect::<color_eyre::Result<_>>()?;

    Ok((pod, persistent_volume_claims))
}

fn service_try_into_container(
    name: String,
    service: Service,
) -> color_eyre::Result<(Container, Vec<Volume>)> {
    service_check_unsupported(&service)?;

    let name = service.container_name.unwrap_or(name);

    let liveness_probe = service
        .healthcheck
        .filter(|healthcheck| !healthcheck_is_disable(healthcheck))
        .map(healthcheck_try_into_probe)
        .transpose()
        .wrap_err("could not convert healthcheck into k8s liveness probe")?;

    let capabilities = Capabilities {
        add: service.cap_add.filter_empty(),
        drop: None,
    };
    let se_linux_options = security_opts_try_into_se_linux_options(service.security_opt)
        .wrap_err("unsupported security option")?;
    let security_context = SecurityContext {
        privileged: service.privileged.then_some(true),
        capabilities: capabilities.filter_default(),
        se_linux_options: se_linux_options.filter_default(),
        run_as_user: service
            .user
            .map(|user| user.parse())
            .transpose()
            .wrap_err("user must be specified as a UID")?,
        ..SecurityContext::default()
    };

    let ports = ports_try_into_container_ports(service.ports).wrap_err("could not parse ports")?;

    let env = environment_into_env_vars(service.environment);

    let (volume_mounts, volumes): (Vec<_>, _) = service
        .tmpfs
        .into_iter()
        .flat_map(|tmpfs| match tmpfs {
            Tmpfs::Simple(tmpfs) => vec![parse_tmpfs_volume_mount(&tmpfs, &name)],
            Tmpfs::List(tmpfs) => tmpfs
                .into_iter()
                .map(|tmpfs| parse_tmpfs_volume_mount(&tmpfs, &name))
                .collect(),
        })
        .chain(
            compose_volumes_try_into_volume_mounts(service.volumes, &name)
                .wrap_err("could not parse volumes")?,
        )
        .unzip();

    let args = service.command.map(|command| match command {
        Command::Simple(command) => vec![command],
        Command::Args(command) => command,
    });

    let command = service.entrypoint.map(|entrypoint| match entrypoint {
        Entrypoint::Simple(entrypoint) => vec![entrypoint],
        Entrypoint::List(entrypoint) => entrypoint,
    });

    let container = Container {
        name,
        image: service.image,
        command,
        args,
        liveness_probe,
        security_context: security_context.filter_default(),
        ports: ports.filter_empty(),
        env: env.filter_empty(),
        resources: ulimits_into_resources(service.ulimits),
        working_dir: service.working_dir,
        stdin: service.stdin_open.then_some(true),
        tty: service.tty.then_some(true),
        volume_mounts: volume_mounts.filter_empty(),
        ..Container::default()
    };

    Ok((container, volumes))
}

fn service_check_unsupported(service: &Service) -> color_eyre::Result<()> {
    let unsupported_options = [
        ("hostname", service.hostname.is_none()),
        ("deploy", service.deploy.is_none()),
        ("build", service.build_.is_none()),
        ("depends_on", service.depends_on.is_empty()),
        ("env_file", service.env_file.is_none()),
        ("profiles", service.profiles.is_empty()),
        ("links", service.links.is_empty()),
        ("net", service.net.is_none()),
        ("stop_signal", service.stop_signal.is_none()),
        ("expose", service.expose.is_empty()),
        ("volumes_from", service.volumes_from.is_empty()),
        ("extends", service.extends.is_empty()),
        ("scale", service.scale == 0),
        ("init", !service.init),
        ("shm_size", service.shm_size.is_none()),
        ("sysctls", service.sysctls.is_empty()),
    ];
    for (option, not_present) in unsupported_options {
        eyre::ensure!(not_present, "`{option}` is not supported for pods");
    }

    let unsupported_container_options = [
        ("pid", service.pid.is_none()),
        ("network_mode", service.network_mode.is_none()),
        ("restart", service.restart.is_none()),
        ("labels", service.labels.is_empty()),
        ("networks", service.networks.is_empty()),
        ("stop_grace_period", service.stop_grace_period.is_none()),
        ("dns", service.dns.is_empty()),
        ("ipc", service.ipc.is_none()),
        ("logging", service.logging.is_none()),
        ("extra_hosts", service.extra_hosts.is_empty()),
    ];
    for (option, not_present) in unsupported_container_options {
        eyre::ensure!(
            not_present,
            "pods do not support per container `{option}` options, \
                try setting the pod option instead",
        );
    }

    eyre::ensure!(
        service.devices.is_empty(),
        "pods do not directly support devices, try using a bind mount instead"
    );

    eyre::ensure!(
        service.extensions.is_empty(),
        "podman does not support docker extensions"
    );

    Ok(())
}

fn healthcheck_is_disable(healthcheck: &Healthcheck) -> bool {
    healthcheck.disable
        || healthcheck
            .test
            .as_ref()
            .map(|test| match test {
                HealthcheckTest::Single(_) => false,
                HealthcheckTest::Multiple(test) => test == &["NONE"],
            })
            .unwrap_or_default()
}

fn healthcheck_try_into_probe(healthcheck: Healthcheck) -> color_eyre::Result<Probe> {
    let Healthcheck {
        test,
        interval,
        timeout,
        retries,
        start_period,
        disable: _,
    } = healthcheck;

    let exec = test
        .map(|test| {
            match test {
                HealthcheckTest::Single(_) => None,
                HealthcheckTest::Multiple(mut test) => match test.first().map(String::as_str) {
                    Some("CMD") => {
                        test.remove(0); // can't panic, there is at least one element ("CMD")
                        Some(ExecAction {
                            command: Some(test),
                        })
                    }
                    _ => None,
                },
            }
            .ok_or(eyre::eyre!(
                "healthcheck implicitly using a shell is not supported for pods"
            ))
            .suggestion(r#"change healthcheck test to '["CMD", "/bin/sh", "-c", ...]'"#)
        })
        .transpose()?;

    let period_seconds = interval
        .map(|interval| parse_seconds(&interval))
        .transpose()
        .wrap_err("could not parse `interval`")?;

    let timeout_seconds = timeout
        .map(|timeout| parse_seconds(&timeout))
        .transpose()
        .wrap_err("could not parse `timeout`")?;

    let failure_threshold = (retries != 0)
        .then(|| retries.try_into())
        .transpose()
        .wrap_err_with(|| format!("`{retries}` retries is too large"))?;

    let initial_delay_seconds = start_period
        .map(|start_period| parse_seconds(&start_period))
        .transpose()
        .wrap_err("could not parse `start_period`")?;

    Ok(Probe {
        exec,
        failure_threshold,
        initial_delay_seconds,
        period_seconds,
        timeout_seconds,
        ..Probe::default()
    })
}

fn parse_seconds(duration: &str) -> color_eyre::Result<i32> {
    duration_str::parse(duration)
        .wrap_err_with(|| format!("could not parse `{duration}` as a valid duration"))
        .and_then(|period| {
            let seconds = period.as_secs();
            seconds
                .try_into()
                .wrap_err_with(|| format!("`{seconds}` seconds is too large"))
        })
}

fn security_opts_try_into_se_linux_options(
    security_opts: Vec<String>,
) -> color_eyre::Result<SELinuxOptions> {
    security_opts.into_iter().try_fold(
        SELinuxOptions::default(),
        |mut se_linux_options, security_opt| {
            let security_opt = if security_opt == "no-new-privileges:true" {
                SecurityOpt::NoNewPrivileges
            } else if security_opt == "no-new-privileges:false" {
                return Ok(se_linux_options);
            } else {
                security_opt.replacen(':', "=", 1).parse()?
            };

            match security_opt {
                SecurityOpt::Apparmor(_) => Err(eyre::eyre!(
                    "`apparmor` security_opt is not supported for pods"
                )),
                SecurityOpt::Label(label_opt) => match label_opt {
                    LabelOpt::User(user) => {
                        se_linux_options.user = Some(user);
                        Ok(se_linux_options)
                    }
                    LabelOpt::Role(role) => {
                        se_linux_options.role = Some(role);
                        Ok(se_linux_options)
                    }
                    LabelOpt::Type(kind) => {
                        se_linux_options.type_ = Some(kind);
                        Ok(se_linux_options)
                    }
                    LabelOpt::Level(level) => {
                        se_linux_options.level = Some(level);
                        Ok(se_linux_options)
                    }
                    LabelOpt::Filetype(_) => Err(eyre::eyre!(
                        "`label:filetype` security_opt is not supported for pods"
                    )),
                    LabelOpt::Disable => Err(eyre::eyre!(
                        "`label:disable` security_opt is not supported for pods"
                    )),
                    LabelOpt::Nested => Err(eyre::eyre!(
                        "`label:nested` security_opt is not supported for pods"
                    )),
                },
                SecurityOpt::Mask(_) => {
                    Err(eyre::eyre!("`mask` security_opt is not supported for pods"))
                }
                SecurityOpt::NoNewPrivileges => Err(eyre::eyre!(
                    "`no-new-privileges` security_opt is not supported for pods"
                )),
                SecurityOpt::Seccomp(_) => Err(eyre::eyre!(
                    "`seccomp` security_opt is not supported for pods"
                )),
                SecurityOpt::ProcOpts(_) => Err(eyre::eyre!(
                    "`proc-opts` security_opt is not supported for pods"
                )),
                SecurityOpt::Unmask(_) => Err(eyre::eyre!(
                    "`unmask` security_opt is not supported for pods"
                )),
            }
        },
    )
}

fn ports_try_into_container_ports(ports: Ports) -> color_eyre::Result<Vec<ContainerPort>> {
    match ports {
        Ports::Short(ports) => ports
            .into_iter()
            .map(|port| parse_container_port_from_short(&port))
            .collect(),
        Ports::Long(ports) => ports
            .into_iter()
            .map(|port| {
                eyre::ensure!(port.mode.is_none(), "port mode is not supported for pods");
                Ok(ContainerPort {
                    container_port: port.target.into(),
                    host_ip: port.host_ip,
                    host_port: match port.published {
                        Some(PublishedPort::Single(host_port)) => Some(host_port.into()),
                        Some(PublishedPort::Range(_)) => {
                            eyre::bail!("pods do not support published port ranges")
                        }
                        None => None,
                    },
                    name: None,
                    protocol: port.protocol,
                })
            })
            .collect(),
    }
}

fn parse_container_port_from_short(port: &str) -> color_eyre::Result<ContainerPort> {
    let (port, protocol) = port
        .split_once('/')
        .map_or((port, None), |(port, protocol)| {
            (port, Some(String::from(protocol)))
        });

    let (host, container_port) = port
        .rsplit_once(':')
        .map_or((None, port), |(host, container_port)| {
            (Some(host), container_port)
        });
    let container_port = container_port
        .parse()
        .wrap_err_with(|| format!("could not parse `{container_port}` as container_port"))?;

    let (host_ip, host_port) = host
        .map(|host| {
            host.split_once(':')
                .map_or((None, host), |(host_ip, host_port)| {
                    (Some(String::from(host_ip)), host_port)
                })
        })
        .unzip();
    let host_port = host_port
        .map(|host_port| {
            host_port
                .parse()
                .wrap_err_with(|| format!("could not parse `{host_port}` as host_port"))
        })
        .transpose()?;

    Ok(ContainerPort {
        container_port,
        host_ip: host_ip.flatten(),
        host_port,
        name: None,
        protocol,
    })
}

fn environment_into_env_vars(environment: Environment) -> Vec<EnvVar> {
    match environment {
        Environment::List(environment) => environment
            .into_iter()
            .map(|env_var| {
                let (name, value) = env_var
                    .split_once('=')
                    .map(|(name, value)| (String::from(name), String::from(value)))
                    .unzip();
                EnvVar {
                    name: name.unwrap_or(env_var),
                    value,
                    value_from: None,
                }
            })
            .collect(),
        Environment::KvPair(environment) => environment
            .into_iter()
            .map(|(name, value)| EnvVar {
                name,
                value: value.as_ref().map(ToString::to_string),
                value_from: None,
            })
            .collect(),
    }
}

fn ulimits_into_resources(ulimits: Ulimits) -> Option<ResourceRequirements> {
    (!ulimits.is_empty()).then(|| ResourceRequirements {
        claims: None,
        limits: Some(
            ulimits
                .0
                .into_iter()
                .map(|(name, ulimit)| {
                    let limit = match ulimit {
                        Ulimit::Single(limit) => limit,
                        Ulimit::SoftHard { soft: _, hard } => hard,
                    };
                    (name, Quantity(limit.to_string()))
                })
                .collect(),
        ),
        requests: None,
    })
}

fn parse_tmpfs_volume_mount(tmpfs: &str, container_name: &str) -> (VolumeMount, Volume) {
    let name = volume_name(container_name, tmpfs);
    let volume_mount = volume_mount(String::from(tmpfs), name.clone(), false);
    (volume_mount, tmpfs_volume(name, None))
}

fn compose_volumes_try_into_volume_mounts(
    volumes: ComposeVolumes,
    container_name: &str,
) -> color_eyre::Result<Vec<(VolumeMount, Volume)>> {
    match volumes {
        ComposeVolumes::Simple(volumes) => volumes
            .into_iter()
            .map(|volume| parse_short_volume(volume, container_name))
            .collect(),
        ComposeVolumes::Advanced(volumes) => volumes
            .into_iter()
            .map(|volume| advanced_volume_try_into_volume_mount(volume, container_name))
            .collect(),
    }
}

fn parse_short_volume(
    volume: String,
    container_name: &str,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    let mut split = volume.split(':');
    match split.clone().count() {
        // anonymous volume, no options
        1 => {
            let name = volume_name(container_name, &volume);
            Ok((
                volume_mount(volume, name.clone(), false),
                anonymous_volume(name),
            ))
        }

        // anonymous volume with options, named volume, or bind mount
        2 => {
            let source_or_target = split.next().expect("split has 2 elements");
            let target_or_options = split.next().expect("split has 2 elements");

            if target_or_options.contains('/') {
                // named volume or bind mount
                let source = source_or_target;
                let target = target_or_options;

                if source.starts_with(['.', '/', '~']) {
                    // bind mount
                    let name = volume_name(container_name, target);
                    Ok((
                        volume_mount(String::from(target), name.clone(), false),
                        bind_volume(name, String::from(source)),
                    ))
                } else {
                    // named volume
                    let name = String::from(source);
                    Ok((
                        volume_mount(String::from(target), name.clone(), false),
                        named_volume(name),
                    ))
                }
            } else {
                // anonymous volume with options
                let target = source_or_target;
                let options = target_or_options;

                let (target, read_only) = parse_target_and_read_only(target, options);

                let name = volume_name(container_name, &target);
                Ok((
                    volume_mount(target, name.clone(), read_only),
                    anonymous_volume(name),
                ))
            }
        }

        // named volume or bind mount with options
        3 => {
            let source = split.next().expect("split has 3 elements");
            let target = split.next().expect("split has 3 elements");
            let options = split.next().expect("split has 3 elements");

            let (target_with_options, read_only) = parse_target_and_read_only(target, options);

            if source.starts_with(['.', '/', '~']) {
                // bind mount with options
                let name = volume_name(container_name, target);
                Ok((
                    volume_mount(target_with_options, name.clone(), read_only),
                    bind_volume(name, String::from(source)),
                ))
            } else {
                // named volume with options
                let name = String::from(source);
                Ok((
                    volume_mount(target_with_options, name.clone(), read_only),
                    named_volume(name),
                ))
            }
        }

        _ => eyre::bail!("too many `:` in volume definition"),
    }
}

fn parse_target_and_read_only(target: &str, options: &str) -> (String, bool) {
    let mut read_only = false;
    let target = options
        .split(',')
        .fold(String::from(target), |target, option| {
            if option == "ro" {
                read_only = true;
                target
            } else if option == "rw" {
                target
            } else if target.contains(':') {
                target + "," + option
            } else {
                target + ":" + option
            }
        });
    (target, read_only)
}

fn advanced_volume_try_into_volume_mount(
    volume: AdvancedVolumes,
    container_name: &str,
) -> color_eyre::Result<(VolumeMount, Volume)> {
    let AdvancedVolumes {
        source,
        target,
        _type: kind,
        read_only,
        bind,
        volume,
        tmpfs,
    } = volume;

    let volume = match kind.as_str() {
        "bind" => {
            eyre::ensure!(
                bind.is_none(),
                "bind mount propagation is not supported by pods"
            );
            let source = source.ok_or(eyre::eyre!("cannot have a bind mount without a source"))?;
            let name = volume_name(container_name, &target);
            bind_volume(name, source)
        }
        "volume" => {
            eyre::ensure!(
                volume.is_none(),
                "volume nocopy option is not supported by pods"
            );
            source.map_or_else(
                || -> color_eyre::Result<_> {
                    let name = volume_name(container_name, &target);
                    Ok(anonymous_volume(name))
                },
                |source| Ok(named_volume(source)),
            )?
        }
        "tmpfs" => {
            let name = volume_name(container_name, &target);
            tmpfs_volume(name, tmpfs.map(|settings| settings.size))
        }
        _ => eyre::bail!("unsupported volume type: `{kind}`"),
    };

    Ok((volume_mount(target, volume.name.clone(), read_only), volume))
}

fn volume_name(container_name: &str, path: &str) -> String {
    format!("{container_name}{}", path.replace(['/', '\\'], "-"))
}

fn volume_mount(mount_path: String, name: String, read_only: bool) -> VolumeMount {
    VolumeMount {
        mount_path,
        mount_propagation: None,
        name,
        read_only: read_only.then_some(true),
        sub_path: None,
        sub_path_expr: None,
    }
}

fn tmpfs_volume(name: String, size_limit: Option<u64>) -> Volume {
    Volume {
        name,
        empty_dir: Some(EmptyDirVolumeSource {
            medium: Some(String::from("Memory")),
            size_limit: size_limit.map(|size_limit| Quantity(size_limit.to_string())),
        }),
        aws_elastic_block_store: None,
        azure_disk: None,
        azure_file: None,
        cephfs: None,
        cinder: None,
        config_map: None,
        csi: None,
        downward_api: None,
        ephemeral: None,
        fc: None,
        flex_volume: None,
        flocker: None,
        gce_persistent_disk: None,
        git_repo: None,
        glusterfs: None,
        host_path: None,
        iscsi: None,
        nfs: None,
        persistent_volume_claim: None,
        photon_persistent_disk: None,
        portworx_volume: None,
        projected: None,
        quobyte: None,
        rbd: None,
        scale_io: None,
        secret: None,
        storageos: None,
        vsphere_volume: None,
    }
}

fn anonymous_volume(name: String) -> Volume {
    Volume {
        name,
        empty_dir: Some(EmptyDirVolumeSource::default()),
        aws_elastic_block_store: None,
        azure_disk: None,
        azure_file: None,
        cephfs: None,
        cinder: None,
        config_map: None,
        csi: None,
        downward_api: None,
        ephemeral: None,
        fc: None,
        flex_volume: None,
        flocker: None,
        gce_persistent_disk: None,
        git_repo: None,
        glusterfs: None,
        host_path: None,
        iscsi: None,
        nfs: None,
        persistent_volume_claim: None,
        photon_persistent_disk: None,
        portworx_volume: None,
        projected: None,
        quobyte: None,
        rbd: None,
        scale_io: None,
        secret: None,
        storageos: None,
        vsphere_volume: None,
    }
}

fn bind_volume(name: String, path: String) -> Volume {
    Volume {
        name,
        host_path: Some(HostPathVolumeSource { path, type_: None }),
        aws_elastic_block_store: None,
        azure_disk: None,
        azure_file: None,
        cephfs: None,
        cinder: None,
        config_map: None,
        csi: None,
        downward_api: None,
        empty_dir: None,
        ephemeral: None,
        fc: None,
        flex_volume: None,
        flocker: None,
        gce_persistent_disk: None,
        git_repo: None,
        glusterfs: None,
        iscsi: None,
        nfs: None,
        persistent_volume_claim: None,
        photon_persistent_disk: None,
        portworx_volume: None,
        projected: None,
        quobyte: None,
        rbd: None,
        scale_io: None,
        secret: None,
        storageos: None,
        vsphere_volume: None,
    }
}

fn named_volume(name: String) -> Volume {
    Volume {
        name: name.clone(),
        persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
            claim_name: name,
            read_only: None,
        }),
        aws_elastic_block_store: None,
        azure_disk: None,
        azure_file: None,
        cephfs: None,
        cinder: None,
        config_map: None,
        csi: None,
        downward_api: None,
        empty_dir: None,
        ephemeral: None,
        fc: None,
        flex_volume: None,
        flocker: None,
        gce_persistent_disk: None,
        git_repo: None,
        glusterfs: None,
        host_path: None,
        iscsi: None,
        nfs: None,
        photon_persistent_disk: None,
        portworx_volume: None,
        projected: None,
        quobyte: None,
        rbd: None,
        scale_io: None,
        secret: None,
        storageos: None,
        vsphere_volume: None,
    }
}

fn compose_volume_try_into_persistent_volume_claim(
    compose_volume: ComposeVolume,
    name: String,
) -> color_eyre::Result<PersistentVolumeClaim> {
    eyre::ensure!(
        compose_volume.external.is_none() && compose_volume.name.is_none(),
        "external volumes are not supported"
    );

    let annotations: BTreeMap<_, _> = compose_volume
        .driver
        .map(|driver| Ok((String::from("volume.podman.io/driver"), driver)))
        .into_iter()
        .chain(driver_opts_try_into_annotations(compose_volume.driver_opts))
        .collect::<color_eyre::Result<_>>()?;

    let labels: BTreeMap<_, _> = match compose_volume.labels {
        Labels::List(labels) => labels
            .into_iter()
            .map(|label| {
                #[allow(clippy::map_unwrap_or)] // map_or_else forces clone of label
                label
                    .split_once('=')
                    .map(|(label, value)| (String::from(label), String::from(value)))
                    .unwrap_or_else(|| (label, String::new()))
            })
            .collect(),
        Labels::Map(labels) => labels.into_iter().collect(),
    };

    Ok(PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(name),
            annotations: annotations.filter_empty(),
            labels: labels.filter_empty(),
            ..ObjectMeta::default()
        },
        spec: None,
        status: None,
    })
}

fn driver_opts_try_into_annotations(
    driver_opts: IndexMap<String, Option<SingleValue>>,
) -> impl Iterator<Item = color_eyre::Result<(String, String)>> {
    driver_opts
        .into_iter()
        .flat_map(|(option, value)| match option.as_str() {
            "type" => vec![Ok((
                String::from("volume.podman.io/type"),
                value.as_ref().map(ToString::to_string).unwrap_or_default(),
            ))],
            "device" => vec![Ok((
                String::from("volume.podman.io/device"),
                value.as_ref().map(ToString::to_string).unwrap_or_default(),
            ))],
            "o" => value
                .map(|value| MountOptions::from(value.to_string()))
                .unwrap_or_default()
                .into_annotations()
                .map(Ok)
                .collect(),
            _ => vec![Err(eyre::eyre!(
                "unsupported volume driver_opt: `{option}`"
            ))],
        })
}

#[derive(Debug, Default)]
struct MountOptions {
    uid: Option<String>,
    gid: Option<String>,
    options: Option<String>,
}

impl From<String> for MountOptions {
    fn from(value: String) -> Self {
        value
            .split(',')
            .fold(Self::default(), |mut mount_options, option| {
                if option.starts_with("uid=") {
                    let (_, uid) = option
                        .split_once('=')
                        .expect("delimiter is in if expression");
                    mount_options.uid = Some(String::from(uid));
                    mount_options
                } else if option.starts_with("gid=") {
                    let (_, gid) = option
                        .split_once('=')
                        .expect("delimiter is in if expression");
                    mount_options.gid = Some(String::from(gid));
                    mount_options
                } else if let Some(options) = mount_options.options {
                    mount_options.options = Some(options + "," + option);
                    mount_options
                } else {
                    mount_options.options = Some(String::from(option));
                    mount_options
                }
            })
    }
}

impl MountOptions {
    fn into_annotations(self) -> impl Iterator<Item = (String, String)> {
        self.uid
            .map(|uid| (String::from("volume.podman.io/uid"), uid))
            .into_iter()
            .chain(
                self.gid
                    .map(|gid| (String::from("volume.podman.io/gid"), gid)),
            )
            .chain(
                self.options
                    .map(|options| (String::from("volume.podman.io/mount-options"), options)),
            )
    }
}

trait FilterDefault {
    fn filter_default(self) -> Option<Self>
    where
        Self: Sized;
}

impl<T: Default + PartialEq<T>> FilterDefault for T {
    fn filter_default(self) -> Option<Self> {
        (self != Self::default()).then_some(self)
    }
}

trait FilterEmpty {
    fn filter_empty(self) -> Option<Self>
    where
        Self: Sized;
}

impl<T> FilterEmpty for Vec<T> {
    fn filter_empty(self) -> Option<Self> {
        (!self.is_empty()).then_some(self)
    }
}

impl<K, V> FilterEmpty for BTreeMap<K, V> {
    fn filter_empty(self) -> Option<Self> {
        (!self.is_empty()).then_some(self)
    }
}
