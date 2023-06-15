use std::net::IpAddr;

use clap::{Args, Subcommand};
use ipnet::IpNet;

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
    /// The range must be a complete subnet and in CIDR notation
    ///
    /// Converts to "IPRange=IP_RANGE"
    #[arg(long)]
    ip_range: Vec<IpNet>,

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

    /// The name of the network to create
    ///
    /// This will be used as the name of the generated file when used with
    /// the --file option without a filename
    name: String,
}

impl From<Create> for crate::quadlet::Network {
    fn from(value: Create) -> Self {
        Self {
            disable_dns: value.disable_dns,
            driver: value.driver,
            gateway: value.gateway,
            internal: value.internal,
            ipam_driver: value.ipam_driver,
            ip_range: value.ip_range,
            ipv6: value.ipv6,
            label: value.label,
            options: Some(value.opt.join(",")),
            subnet: value.subnet,
        }
    }
}
