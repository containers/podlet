//! Provides the `podlet generate` subcommand, see [`Generate`].
//!
//! `podlet generate` uses the podman `inspect` commands to get information on the selected
//! resource. The information is converted into a [`PodmanCommands`] which, in turn, is turned into
//! a [`crate::quadlet::File`].

use std::{
    fmt::{self, Display, Formatter},
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

use crate::quadlet::IpRange;

use super::{
    container::Container,
    network::{self, Network},
    service::Service,
    volume::{self, Volume},
    PodmanCommands,
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
}

impl TryFrom<Generate> for PodmanCommands {
    type Error = color_eyre::Report;

    fn try_from(value: Generate) -> Result<Self, Self::Error> {
        match value {
            Generate::Container { container } => {
                ContainerParser::from_container(&container).map(Into::into)
            }
            Generate::Network { network } => Ok(Self::Network {
                network: Box::new(NetworkInspect::from_network(&network)?.into()),
            }),
            Generate::Volume { volume } => Ok(Self::Volume {
                volume: VolumeInspect::from_volume(&volume)?.into(),
            }),
        }
    }
}

/// [`Parser`] for container creation CLI options.
#[derive(Parser, Debug)]
#[command(no_binary_name = true)]
struct ContainerParser {
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
                format!("error getting command used to create container: {container}")
            })?
            .config
            .create_command;

        Self::try_parse_from(strip_container_create_command_prefix(&create_command)).wrap_err_with(
            || {
                format!(
                    "error parsing podman command from: {}",
                    shlex::join(create_command.iter().map(String::as_str))
                )
            },
        )
    }
}

/// Remove the command part of `command`, leaving just the container creation options.
fn strip_container_create_command_prefix(command: &[String]) -> impl Iterator<Item = &String> {
    let mut iter = command.iter().peekable();

    // remove arg0, i.e. "podman" or "/usr/bin/podman"
    iter.next();

    // command could be `podman run`, `podman create`, or `podman container create`
    if iter.peek().is_some_and(|arg| *arg == "container") {
        iter.next();
    }
    if iter
        .peek()
        .is_some_and(|arg| *arg == "run" || *arg == "create")
    {
        iter.next();
    }

    iter
}

impl From<ContainerParser> for PodmanCommands {
    fn from(ContainerParser { container, service }: ContainerParser) -> Self {
        PodmanCommands::Run {
            container: Box::new(container),
            service,
        }
    }
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
        .note("ensure podman is installed and available on $PATH")?;

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
            format!(
                "error deserializing container create command \
                    from `podman {resource_kind} inspect {resource}` output"
            )
        })
        .with_section(|| stdout.trim().to_owned().header("Podman Stdout:"))?
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("no {resource_kind}s matching `{resource}`"))
}

/// Used by [`podman_inspect()`] to run the correct variant of `podman inspect`.
#[derive(Debug, Clone, Copy)]
enum ResourceKind {
    Container,
    Network,
    Volume,
}

impl ResourceKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Container => "container",
            Self::Network => "network",
            Self::Volume => "volume",
        }
    }
}

impl Display for ResourceKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
