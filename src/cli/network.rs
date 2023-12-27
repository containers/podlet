use std::{
    fmt::{self, Display, Formatter},
    net::IpAddr,
};

use clap::{Args, Subcommand};
use ipnet::IpNet;
use serde::Serialize;

use crate::quadlet::IpRange;

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Network {
    /// Generate a podman quadlet `.network` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-network-create.1.html and
    /// https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#network-units-network
    #[group(skip)]
    Create {
        #[command(flatten)]
        create: Create,
    },
}

impl From<Network> for crate::quadlet::Network {
    fn from(value: Network) -> Self {
        let Network::Create { create } = value;
        create.into()
    }
}

impl From<Network> for crate::quadlet::Resource {
    fn from(value: Network) -> Self {
        crate::quadlet::Network::from(value).into()
    }
}

impl Network {
    pub fn name(&self) -> &str {
        let Self::Create { create } = self;
        &create.name
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Create {
    /// Disable the DNS plugin for the network
    ///
    /// Converts to "DisableDNS=true"
    #[arg(long)]
    disable_dns: bool,

    /// Set network-scoped DNS resolver/nameserver for containers in this network
    ///
    /// Converts to "DNS=IP"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "IP")]
    dns: Vec<String>,

    /// Driver to manage the network
    ///
    /// Converts to "Driver=DRIVER"
    #[arg(short, long)]
    driver: Option<String>,

    /// Define a gateway for the subnet
    ///
    /// Converts to "Gateway=GATEWAY"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    gateway: Vec<IpAddr>,

    /// Restrict external access of the network
    ///
    /// Converts to "Internal=true"
    #[arg(long)]
    internal: bool,

    /// Set the IPAM driver (IP Address Management Driver) for the network
    ///
    /// Converts to "IPAMDriver=DRIVER"
    #[arg(long, value_name = "DRIVER")]
    ipam_driver: Option<String>,

    /// Allocate container IP from a range
    ///
    /// The range must be a complete subnet in CIDR notation, or be in the `<startIP>-<endIP>`
    /// syntax which allows for a more flexible range compared to the CIDR subnet.
    ///
    /// Converts to "IPRange=IP_RANGE"
    #[arg(long)]
    ip_range: Vec<IpRange>,

    /// Enable IPv6 (Dual Stack) networking
    ///
    /// Converts to "IPv6=true"
    #[arg(long)]
    ipv6: bool,

    /// Set one or more OCI labels on the network
    ///
    /// Converts to "Label=KEY=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "KEY=VALUE")]
    label: Vec<String>,

    /// Set driver specific options
    ///
    /// Converts to "Options=OPTION[,...]"
    ///
    /// Can be specified multiple times
    #[arg(short, long, value_name = "OPTION", value_delimiter = ',')]
    opt: Vec<String>,

    /// The subnet in CIDR notation
    ///
    /// Converts to "Subnet=SUBNET"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    subnet: Vec<IpNet>,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: PodmanArgs,

    /// The name of the network to create
    ///
    /// This will be used as the name of the generated file when used with
    /// the --file option without a filename
    name: String,
}

impl From<Create> for crate::quadlet::Network {
    fn from(value: Create) -> Self {
        let podman_args = value.podman_args.to_string();
        Self {
            disable_dns: value.disable_dns,
            dns: value.dns,
            driver: value.driver,
            gateway: value.gateway,
            internal: value.internal,
            ipam_driver: value.ipam_driver,
            ip_range: value.ip_range,
            ipv6: value.ipv6,
            label: value.label,
            options: (!value.opt.is_empty()).then(|| value.opt.join(",")),
            podman_args: (!podman_args.is_empty()).then_some(podman_args),
            subnet: value.subnet,
        }
    }
}

#[derive(Args, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct PodmanArgs {
    /// Maps to the `network_interface` option in the network config
    #[arg(long, value_name = "NAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    interface_name: Option<String>,

    /// A static route to add to every container in this network
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    route: Vec<String>,
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let args = crate::serde::args::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&args)
    }
}
