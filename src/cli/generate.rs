//! Provides the `podlet generate` subcommand, see [`Generate`].
//!
//! `podlet generate` uses the podman `inspect` commands to get information on the selected
//! resource. The information is converted into a [`PodmanCommands`] which, in turn, is turned into
//! a [`crate::quadlet::File`].

use std::{
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    process::Command,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{eyre, WrapErr},
    Section, SectionExt,
};
use indexmap::IndexMap;
use ipnet::IpNet;
use serde::{de::DeserializeOwned, Deserialize};

use crate::quadlet::{self, Globals, Install, IpRange, ResourceKind};

use super::{
    global_args::GlobalArgs, image, network, service::Service, unit::Unit, volume, Container,
    Image, Network, Pod, Volume,
};

/// [`Subcommand`] for `podlet generate`
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Generate {
    /// Generate a quadlet file from an existing container
    ///
    /// The command used to create the container is parsed to generate the quadlet file.
    Container {
        /// Name or ID of the container
        ///
        /// Passed to `podman container inspect`.
        container: String,
    },

    /// Generate quadlet files from an existing pod and its containers
    ///
    /// Creates a `.pod` quadlet file and a `.container` quadlet file for each container in the pod.
    ///
    /// Only supports pods created with `podman pod create`.
    /// The command used to create the pod is parsed to generate the quadlet file.
    Pod {
        /// Name or ID of the pod
        ///
        /// Passed to `podman pod inspect`.
        pod: String,
    },

    /// Generate a quadlet file from an existing network
    ///
    /// The generated quadlet file will be larger than strictly necessary.
    /// It is impossible to determine which CLI options were explicitly set when the network was
    /// created from the output of `podman network inspect`.
    ///
    /// You may wish to remove some of the generated quadlet options for which you do not need a
    /// precise value.
    Network {
        /// Name of the network
        ///
        /// Passed to `podman network inspect`.
        network: String,
    },

    /// Generate a quadlet file from an existing volume
    Volume {
        /// Name of the volume
        ///
        /// Passed to `podman volume inspect`.
        volume: String,
    },

    /// Generate a quadlet file from an image in local storage
    Image {
        /// Name of the image
        ///
        /// Passed to `podman image inspect`.
        image: String,
    },
}

impl Generate {
    /// Inspect the given resource by running a podman command, deserializing the output,
    /// and transforming it into one or more [`quadlet::File`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if there is a problem running the podman command
    /// or its output could not be deserialized.
    pub fn try_into_quadlet_files(
        self,
        name: Option<String>,
        unit: Option<Unit>,
        install: Option<Install>,
    ) -> color_eyre::Result<Vec<quadlet::File>> {
        match self {
            Self::Container { container } => Ok(vec![ContainerParser::from_container(&container)?
                .into_quadlet_file(None, name, unit, install)]),
            Self::Pod { pod } => {
                Ok(PodParser::from_pod(&pod)?.into_quadlet_files(name, unit, install))
            }
            Self::Network { network } => Ok(vec![
                NetworkInspect::from_network(&network)?.into_quadlet_file(name, unit, install)
            ]),
            Self::Volume { volume } => Ok(vec![
                VolumeInspect::from_volume(&volume)?.into_quadlet_file(name, unit, install)
            ]),
            Self::Image { image } => Ok(vec![
                ImageInspect::from_image(&image)?.into_quadlet_file(name, unit, install)
            ]),
        }
    }
}

/// [`Parser`] for container creation CLI options.
#[derive(Parser, Debug)]
#[command(no_binary_name = true)]
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
            format!("error parsing podman container command from `{create_command:?}`")
        })
    }

    /// Convert the parsed container command into a [`quadlet::File`].
    fn into_quadlet_file(
        self,
        pod: Option<&str>,
        name: Option<String>,
        unit: Option<Unit>,
        install: Option<Install>,
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
            service: (!service.is_empty()).then_some(service),
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
#[command(no_binary_name = true)]
struct PodParser {
    /// Podman global options
    #[command(flatten)]
    global_args: GlobalArgs,

    /// The \[Pod\] section
    #[command(subcommand)]
    pod: Pod,

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
        unit: Option<Unit>,
        install: Option<Install>,
    ) -> Vec<quadlet::File> {
        let Self {
            global_args,
            pod,
            containers,
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
            service: None,
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
        unit: Option<Unit>,
        install: Option<Install>,
    ) -> quadlet::File {
        let network = Network::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| network.name().to_owned()),
            unit,
            resource: network.into(),
            globals: Globals::default(),
            service: None,
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
        unit: Option<Unit>,
        install: Option<Install>,
    ) -> quadlet::File {
        let volume = Volume::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| volume.name().to_owned()),
            unit,
            resource: volume.into(),
            globals: Globals::default(),
            service: None,
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
        unit: Option<Unit>,
        install: Option<Install>,
    ) -> quadlet::File {
        let image = Image::from(self);
        quadlet::File {
            name: name.unwrap_or_else(|| image.name().to_owned()),
            unit,
            resource: image.into(),
            globals: Globals::default(),
            service: None,
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
        .note("ensure podman is installed and available on $PATH")
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

    // `podman inspect` returns a JSON array which is also valid YAML so serde_yaml can be reused.
    // There should only be a single object in the array, so the first one is returned.
    serde_yaml::from_str::<Vec<T>>(&stdout)
        .wrap_err_with(|| {
            format!("error deserializing from `podman {resource_kind} inspect {resource}` output")
        })
        .with_section(|| stdout.trim().to_owned().header("Podman Stdout:"))?
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("no {resource_kind}s matching `{resource}`"))
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_container_parser_cli() {
        ContainerParser::command().debug_assert();
    }
}
