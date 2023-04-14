use std::{fmt::Display, net::IpAddr};

use clap::{Args, Subcommand};
use ipnet::IpNet;

use crate::cli::escape_spaces_join;

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

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Create { create } = self;
        writeln!(f, "[Network]")?;
        write!(f, "{create}")
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

impl Display for Create {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.disable_dns {
            writeln!(f, "DisableDNS=true")?;
        }

        if let Some(driver) = &self.driver {
            writeln!(f, "Driver={driver}")?;
        }

        for gateway in &self.gateway {
            writeln!(f, "Gateway={gateway}")?;
        }

        if self.internal {
            writeln!(f, "Internal=true")?;
        }

        if let Some(driver) = &self.ipam_driver {
            writeln!(f, "IPAMDriver={driver}")?;
        }

        for ip_range in &self.ip_range {
            writeln!(f, "IPRange={ip_range}")?;
        }

        if self.ipv6 {
            writeln!(f, "IPv6=true")?;
        }

        if !self.label.is_empty() {
            writeln!(f, "Label={}", escape_spaces_join(&self.label))?;
        }

        if !self.opt.is_empty() {
            writeln!(f, "Options={}", self.opt.join(","))?;
        }

        for subnet in &self.subnet {
            writeln!(f, "Subnet={subnet}")?;
        }

        Ok(())
    }
}
