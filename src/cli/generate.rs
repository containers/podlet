//! Provides the `podlet generate` subcommand, see [`Generate`].
//!
//! `podlet generate` uses the `podman inspect` commands to get information on the selected
//! resource. The information is converted into a [`PodmanCommands`] which, in turn, is turned into
//! a [`crate::quadlet::File`].

use std::{
    env,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    process::Command,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    Section, SectionExt,
    eyre::{WrapErr, eyre},
};
use indexmap::IndexMap;
use ipnet::IpNet;
use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor, value::MapAccessDeserializer},
};

use crate::quadlet::{self, Globals, Install, IpRange, ResourceKind};

use super::{
    Container, Image, Network, Pod, Volume, global_args::GlobalArgs, image, network,
    service::Service, unit::Unit, volume,
};

/// [`Subcommand`] for `podlet generate`
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Generate {
    /// Generate a Quadlet file from an existing container
    ///
    /// The command used to create the container is parsed to generate the Quadlet file.
    Container {
        /// Name or ID of the container
        ///
        /// Passed to `podman container inspect`.
        container: String,
    },

    /// Generate Quadlet files from an existing pod and its containers
    ///
    /// Creates a `.pod` Quadlet file and a `.container` Quadlet file for each container in the pod.
    ///
    /// Only supports pods created with `podman pod create`.
    /// The command used to create the pod is parsed to generate the Quadlet file.
    Pod {
        /// Ignore the `podman pod create --infra-conmon-pidfile` option if it is set.
        ///
        /// Quadlet sets the `--infra-conmon-pidfile` option when generating the systemd service
        /// unit file for the pod, and it cannot be set multiple times. Podlet will, by default,
        /// return an error if the option is used.
        #[arg(long)]
        ignore_infra_conmon_pidfile: bool,

        /// Ignore the `podman pod create --pod-id-file` option if it is set.
        ///
        /// Quadlet sets the `--pod-id-file` option when generating the systemd service unit file
        /// for the pod, and it cannot be set multiple times. Podlet will, by default, return an
        /// error if the option is used.
        #[arg(long)]
        ignore_pod_id_file: bool,

        /// Name or ID of the pod
        ///
        /// Passed to `podman pod inspect`.
        pod: String,
    },

    /// Generate a Quadlet file from an existing network
    ///
    /// The generated Quadlet file will be larger than strictly necessary.
    /// It is impossible to determine which CLI options were explicitly set when the network was
    /// created from the output of `podman network inspect`.
    ///
    /// You may wish to remove some of the generated Quadlet options for which you do not need a
    /// precise value.
    Network {
        /// Name of the network
        ///
        /// Passed to `podman network inspect`.
        network: String,
    },

    /// Generate a Quadlet file from an existing volume
    Volume {
        /// Name of the volume
        ///
        /// Passed to `podman volume inspect`.
        volume: String,
    },

    /// Generate a Quadlet file from an image in local storage
    Image {
        /// Name of the image
        ///
        /// Passed to `podman image inspect`.
        image: String,
    },
}

impl Generate {
    /// Inspect the given resource by running a Podman command, deserializing the output,
    /// and transforming it into one or more [`quadlet::File`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if there is a problem running the Podman command
    /// or its output could not be deserialized.
    pub fn try_into_quadlet_files(
        self,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> color_eyre::Result<Vec<quadlet::File>> {
        match self {
            Self::Container { container } => Ok(vec![
                ContainerParser::from_container(&container)?
                    .into_quadlet_file(None, name, unit, install),
            ]),
            Self::Pod {
                ignore_infra_conmon_pidfile,
                ignore_pod_id_file,
                pod,
            } => {
                let pod = PodParser::from_pod(&pod)?;

                if pod.infra_conmon_pidfile.is_some() && !ignore_infra_conmon_pidfile {
                    Err(eyre!(
                        "the `--infra-conmon-pidfile` option is not \
                        supported as it is set by Quadlet"
                    )
                    .suggestion(
                        "use `podlet generate pod --ignore-infra-conmon-pidfile` \
                        to remove the option",
                    ))
                } else if pod.pod_id_file.is_some() && !ignore_pod_id_file {
                    Err(eyre!(
                        "the `--pod-id-file` option is not supported as it is set by Quadlet"
                    )
                    .suggestion(
                        "use `podlet generate pod --ignore-pod-id-file` to remove the option",
                    ))
                } else {
                    Ok(pod.into_quadlet_files(name, unit, install))
                }
            }
            Self::Network { network } => Ok(vec![
                NetworkInspect::from_network(&network)?.into_quadlet_file(name, unit, install),
            ]),
            Self::Volume { volume } => Ok(vec![
                VolumeInspect::from_volume(&volume)?.into_quadlet_file(name, unit, install),
            ]),
            Self::Image { image } => Ok(vec![
                ImageInspect::from_image(&image)?.into_quadlet_file(name, unit, install),
            ]),
        }
    }
}

/// [`Parser`] for container creation CLI options.
#[derive(Parser, Debug)]
#[command(no_binary_name = true, disable_help_flag = true)]
struct ContainerParser {
    /// Podman global options
    #[command(flatten)]
    global_args: GlobalArgs,

    /// The \[Container\] section
    #[command(flatten)]
    container: Container,

    /// The \[Service\] section
    #[command(flatten)]
    service: Service,
}

impl ContainerParser {
    /// Runs `podman container inspect` on the container and parses the create command.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error getting the create command,
    /// or if it cannot be successfully parsed into container creation CLI options.
    fn from_container(container: &str) -> color_eyre::Result<Self> {
        let create_command = ContainerInspect::from_container(container)
            .wrap_err_with(|| {
                format!("error getting command used to create container `{container}`")
            })?
            .config
            .create_command;

        Self::try_parse_from(filter_container_create_command(&create_command)).wrap_err_with(|| {
            format!("error parsing Podman container command from `{create_command:?}`")
        })
    }

    /// Convert the parsed container command into a [`quadlet::File`].
    fn into_quadlet_file(
        self,
        pod: Option<&str>,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> quadlet::File {
        let Self {
            global_args,
            mut container,
            service,
        } = self;

        if pod.is_some() {
            container.set_pod(None);
        }

        let name = name.unwrap_or_else(|| container.name().to_owned());

        let mut container = quadlet::Container::from(container);
        if let Some(pod) = pod {
            container.pod = Some(format!("{pod}.pod"));
        }

        quadlet::File {
            name,
            unit,
            resource: container.into(),
            globals: global_args.into(),
            service,
            install,
        }
    }
}

/// Remove the command parts of `command`, leaving just the container creation options.
fn filter_container_create_command(command: &[String]) -> impl Iterator<Item = &String> {
    let mut iter = command.iter();

    // remove arg0, i.e. "podman" or "/usr/bin/podman"
    iter.next();

    // command could be `podman run`, `podman create`, or `podman container create`
    let mut command_seen = false;
    iter.filter(move |arg| match (command_seen, arg.as_str()) {
        (false, "container") => false,
        (false, "run" | "create") => {
            command_seen = true;
            false
        }
        // command_seen || arg != command
        (true | false, _) => true,
    })
}

/// Selected output of `podman container inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerInspect {
    config: ContainerConfig,
}

/// Part of `Config` object from the output of `podman container inspect`
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerConfig {
    create_command: Vec<String>,
}

impl ContainerInspect {
    /// Runs `podman container inspect` on the container and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman container inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_container(container: &str) -> color_eyre::Result<Self> {
        podman_inspect(ResourceKind::Container, container)
    }
}

/// [`Parser`] for pod creation CLI options.
#[derive(Parser, Debug)]
#[command(no_binary_name = true, disable_help_flag = true)]
struct PodParser {
    /// Podman global options
    #[command(flatten)]
    global_args: GlobalArgs,

    /// The \[Pod\] section
    #[command(subcommand)]
    pod: Pod,

    /// File to write the PID of the infra container's conmon process to.
    ///
    /// Not supported as Quadlet sets this when generating the pod's `.service` unit file.
    ///
    /// Ignored with the `podlet generate pod --ignore-infra-conmon-pidfile` option. Otherwise
    /// results in error if set.
    #[arg(long, global = true)]
    infra_conmon_pidfile: Option<PathBuf>,

    /// File to write the pod's ID to.
    ///
    /// Not supported as Quadlet sets this when generating the pod's `.service` unit file.
    ///
    /// Ignored with the `podlet generate pod --ignore-pod-id-file` option. Otherwise results in
    /// error if set.
    #[arg(long, global = true)]
    pod_id_file: Option<PathBuf>,

    /// Containers associated with the pod.
    #[arg(skip)]
    containers: Vec<ContainerParser>,
}

impl PodParser {
    /// Runs `podman pod inspect` on the pod and parses the creation command and container list.
    /// For each of the pod's containers, `podman container inspect` is run and the container's
    /// creation command is parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error getting the creation command and containers,
    /// the creation command cannot be successfully parsed into pod CLI options,
    /// there is an error getting one of the pod's container's creation command,
    /// or a container creation command could not be parsed.
    fn from_pod(pod: &str) -> color_eyre::Result<Self> {
        let PodInspect {
            create_command,
            containers,
        } = PodInspect::from_pod(pod).wrap_err_with(|| format!("error inspecting pod `{pod}`"))?;

        // skip the `podman pod` prefix
        let iter = create_command.iter().skip(2);

        let mut pod = Self::try_parse_from(iter).wrap_err_with(|| {
            format!("error parsing `podman pod create` command from `{create_command:?}`")
        })?;

        let containers = containers
            .into_iter()
            .filter_map(|PodContainer { name }| {
                // skip infra containers
                (!name.ends_with("-infra")).then(|| ContainerParser::from_container(&name))
            })
            .collect::<Result<_, _>>()
            .wrap_err("error inspecting one of the pod's containers")?;

        pod.containers = containers;
        Ok(pod)
    }

    /// Convert the parsed pod and containers into [`quadlet::File`]s.
    fn into_quadlet_files(
        self,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> Vec<quadlet::File> {
        let Self {
            global_args,
            pod,
            containers,
            // Handled by Generate::try_into_quadlet_files()
            infra_conmon_pidfile: _,
            pod_id_file: _,
        } = self;

        let pod_name = pod.name();

        let mut files: Vec<_> = containers
            .into_iter()
            .map(|container| {
                container.into_quadlet_file(Some(pod_name), None, unit.clone(), install.clone())
            })
            .collect();

        let pod = quadlet::File {
            name: name.unwrap_or_else(|| pod_name.to_owned()),
            unit,
            resource: pod.into(),
            globals: global_args.into(),
            service: Service::default(),
            install,
        };

        files.push(pod);
        files
    }
}

/// Selected output of `podman pod inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PodInspect {
    /// Full command and arguments that created the pod.
    create_command: Vec<String>,
    /// All containers in the pod.
    containers: Vec<PodContainer>,
}

/// Container in output of `podman pod inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PodContainer {
    /// The name of the container.
    name: String,
}

impl PodInspect {
    /// Runs `podman pod inspect` on the pod and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman pod inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_pod(pod: &str) -> color_eyre::Result<Self> {
        podman_inspect(ResourceKind::Pod, pod)
    }
}

/// Output of `podman network inspect`.
#[derive(Deserialize, Debug)]
struct NetworkInspect {
    /// name
    name: String,

    /// --driver
    driver: String,

    /// --interface-name
    network_interface: String,

    /// --subnet, --gateway, --ip-range
    #[serde(default)]
    subnets: Vec<NetworkSubnet>,

    /// --route
    #[serde(default)]
    routes: Vec<NetworkRoute>,

    /// --ipv6
    ipv6_enabled: bool,

    /// --internal
    internal: bool,

    /// ! --disable-dns
    dns_enabled: bool,

    /// --dns
    #[serde(default)]
    network_dns_servers: Vec<IpAddr>,

    /// --label
    #[serde(default)]
    labels: IndexMap<String, String>,

    /// --opt
    #[serde(default)]
    options: IndexMap<String, String>,

    /// --ipam-driver
    ipam_options: NetworkIpamOptions,
}

#[derive(Deserialize, Debug)]
struct NetworkSubnet {
    /// --subnet
    subnet: IpNet,

    /// --gateway
    #[serde(default)]
    gateway: Option<IpAddr>,

    /// --ip-range
    #[serde(default)]
    lease_range: Option<NetworkLeaseRange>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum NetworkLeaseRange {
    Ipv4 {
        start_ip: Ipv4Addr,
        end_ip: Ipv4Addr,
    },
    Ipv6 {
        start_ip: Ipv6Addr,
        end_ip: Ipv6Addr,
    },
}

impl From<NetworkLeaseRange> for IpRange {
    fn from(value: NetworkLeaseRange) -> Self {
        match value {
            NetworkLeaseRange::Ipv4 { start_ip, end_ip } => Self::Ipv4Range(start_ip..end_ip),
            NetworkLeaseRange::Ipv6 { start_ip, end_ip } => Self::Ipv6Range(start_ip..end_ip),
        }
    }
}

#[derive(Deserialize, Debug)]
struct NetworkRoute {
    destination: IpNet,
    gateway: IpAddr,
    #[serde(default)]
    metric: Option<u32>,
}

impl NetworkRoute {
    /// Format as value suitable for `podman network create --route`:
    /// "<destination in CIDR notation>,<gateway>,<route metric (optional)>".
    fn to_route_value(&self) -> String {
        let Self {
            destination,
            gateway,
            metric,
        } = self;

        if let Some(metric) = metric {
            format!("{destination},{gateway},{metric}")
        } else {
            format!("{destination},{gateway}")
        }
    }
}

#[derive(Deserialize, Debug)]
struct NetworkIpamOptions {
    /// --ipam-driver
    driver: String,
}

impl NetworkInspect {
    /// Runs `podman network inspect` on the network and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman network inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_network(network: &str) -> color_eyre::Result<Self> {
        podman_inspect(ResourceKind::Network, network)
    }

    /// Convert the inspected network into a [`quadlet::File`].
    fn into_quadlet_file(
        self,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> quadlet::File {
        let network = Network::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| network.name().to_owned()),
            unit,
            resource: network.into(),
            globals: Globals::default(),
            service: Service::default(),
            install,
        }
    }
}

impl From<NetworkInspect> for Network {
    fn from(
        NetworkInspect {
            name,
            driver,
            network_interface,
            subnets,
            routes,
            ipv6_enabled: ipv6,
            internal,
            dns_enabled,
            network_dns_servers,
            labels,
            options,
            ipam_options: NetworkIpamOptions {
                driver: ipam_driver,
            },
        }: NetworkInspect,
    ) -> Self {
        let subnets_len = subnets.len();
        let (subnet, gateway, ip_range) = subnets.into_iter().fold(
            (Vec::with_capacity(subnets_len), Vec::new(), Vec::new()),
            |(mut subnets, mut gateways, mut ip_ranges),
             NetworkSubnet {
                 subnet,
                 gateway,
                 lease_range,
             }| {
                subnets.push(subnet);
                gateways.extend(gateway);
                ip_ranges.extend(lease_range.map(Into::into));
                (subnets, gateways, ip_ranges)
            },
        );

        Network::Create {
            create: network::Create {
                disable_dns: !dns_enabled,
                dns: network_dns_servers
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                driver: Some(driver),
                gateway,
                internal,
                ipam_driver: Some(ipam_driver),
                ip_range,
                ipv6,
                label: labels
                    .into_iter()
                    .map(|(label, value)| format!("{label}={value}"))
                    .collect(),
                opt: options
                    .into_iter()
                    .map(|(opt, value)| format!("{opt}={value}"))
                    .collect(),
                subnet,
                podman_args: network::PodmanArgs {
                    interface_name: Some(network_interface),
                    route: routes.iter().map(NetworkRoute::to_route_value).collect(),
                },
                name,
            },
        }
    }
}

/// Output of `podman volume inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct VolumeInspect {
    /// name
    name: String,

    /// --driver
    driver: String,

    /// --label
    labels: IndexMap<String, String>,

    /// --opt
    options: IndexMap<String, String>,
}

impl VolumeInspect {
    /// Runs `podman volume inspect` on the volume and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman volume inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_volume(volume: &str) -> color_eyre::Result<Self> {
        podman_inspect(ResourceKind::Volume, volume)
    }

    /// Convert the inspected volume into a [`quadlet::File`].
    fn into_quadlet_file(
        self,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> quadlet::File {
        let volume = Volume::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| volume.name().to_owned()),
            unit,
            resource: volume.into(),
            globals: Globals::default(),
            service: Service::default(),
            install,
        }
    }
}

impl From<VolumeInspect> for Volume {
    fn from(
        VolumeInspect {
            name,
            driver,
            labels,
            options,
        }: VolumeInspect,
    ) -> Self {
        Volume::Create {
            create: volume::Create {
                driver: Some(driver),
                opt: options
                    .into_iter()
                    .filter_map(|(option, value)| {
                        volume::Opt::parse(&option, (!value.is_empty()).then_some(value)).ok()
                    })
                    .collect(),
                label: labels
                    .into_iter()
                    .map(|(label, value)| format!("{label}={value}"))
                    .collect(),
                name,
            },
        }
    }
}

/// Selected output of `podman image inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ImageInspect {
    repo_tags: Vec<String>,
    #[serde(default)]
    architecture: Option<String>,
    #[serde(default)]
    os: Option<String>,
}

impl ImageInspect {
    /// Runs `podman image inspect` on the image and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman image inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_image(image: &str) -> color_eyre::Result<Self> {
        podman_inspect(ResourceKind::Image, image)
    }

    /// Convert the inspected image into a [`quadlet::File`].
    fn into_quadlet_file(
        self,
        name: Option<String>,
        unit: Unit,
        install: Install,
    ) -> quadlet::File {
        let image = Image::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| image.name().to_owned()),
            unit,
            resource: image.into(),
            globals: Globals::default(),
            service: Service::default(),
            install,
        }
    }
}

impl From<ImageInspect> for Image {
    fn from(
        ImageInspect {
            mut repo_tags,
            architecture: arch,
            os,
        }: ImageInspect,
    ) -> Self {
        let source = repo_tags
            .pop()
            .expect("RepoTags should have at least one value");
        Self::Pull {
            pull: image::Pull {
                arch,
                os,
                source,
                ..Default::default()
            },
        }
    }
}

/// Runs `podman {resource_kind} inspect` on the resource and deserializes the output.
///
/// # Errors
///
/// Returns an error if there is problem running `podman {resource_kind} inspect`,
/// it doesn't complete successfully,
/// or if the output cannot be properly deserialized.
fn podman_inspect<T: DeserializeOwned>(
    resource_kind: ResourceKind,
    resource: &str,
) -> color_eyre::Result<T> {
    let output = Command::new("podman")
        .args([resource_kind.as_str(), "inspect", resource])
        .output()
        .wrap_err_with(|| format!("error running `podman {resource_kind} inspect {resource}`"))
        .note("ensure Podman is installed and available on $PATH")
        .with_section(|| env::var("PATH").unwrap_or_default().header("PATH:"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return if let Some(code) = output.status.code() {
            Err(eyre!(
                "`podman {resource_kind} inspect {resource}` \
                    exited unsuccessfully with status code: {code}"
            ))
        } else {
            Err(eyre!(
                "`podman {resource_kind} inspect {resource}` \
                    was terminated by a signal"
            ))
        }
        .section(stdout.trim().to_owned().header("Podman Stdout:"))
        .section(stderr.trim().to_owned().header("Podman Stderr:"));
    }

    serde_json::Deserializer::from_str(&stdout)
        .deserialize_any(PodmanInspectVisitor {
            resource_kind,
            resource,
            value: PhantomData,
        })
        .wrap_err_with(|| {
            format!("error deserializing from `podman {resource_kind} inspect {resource}` output")
        })
        .with_section(|| stdout.trim().to_owned().header("Podman Stdout:"))
}

/// A [`Visitor`] for deserializing the output of `podman inspect`.
///
/// Podman v5.0.0 and newer always returns an array from `podman inspect`. Older versions may return
/// a single JSON object if there is only one result, notably for `podman pod inspect`.
///
/// If an array is encountered, the first object is returned.
struct PodmanInspectVisitor<'a, T> {
    resource_kind: ResourceKind,
    resource: &'a str,
    value: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for PodmanInspectVisitor<'_, T> {
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "the output of `podman {} inspect`, an object or array",
            self.resource_kind
        )
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(MapAccessDeserializer::new(map))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let Self {
            resource_kind,
            resource,
            ..
        } = self;

        seq.next_element()?.ok_or_else(|| {
            de::Error::custom(format_args!("no {resource_kind}s matching `{resource}`"))
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_container_parser_cli() {
        ContainerParser::command().debug_assert();
    }

    #[test]
    fn verify_pod_parser_cli() {
        PodParser::command().debug_assert();
    }
}
