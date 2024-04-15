use std::{
    fmt::{self, Display, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::{Not, Range},
    str::FromStr,
};

use color_eyre::eyre::{ensure, eyre, Context};
use compose_spec::network::{Ipam, IpamConfig};
use ipnet::IpNet;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::serde::quadlet::quote_spaces_join_space;

use super::{Downgrade, DowngradeError, PodmanVersion};

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Network {
    /// If enabled, disables the DNS plugin for this network.
    #[serde(rename = "DisableDNS", skip_serializing_if = "Not::not")]
    pub disable_dns: bool,

    /// Set network-scoped DNS resolver/nameserver for containers in this network.
    #[serde(rename = "DNS")]
    pub dns: Vec<String>,

    /// Driver to manage the network.
    pub driver: Option<String>,

    /// Define a gateway for the subnet.
    pub gateway: Vec<IpAddr>,

    /// Restrict external access of this network.
    #[serde(skip_serializing_if = "Not::not")]
    pub internal: bool,

    /// Set the ipam driver (IP Address Management Driver) for the network.
    #[serde(rename = "IPAMDriver")]
    pub ipam_driver: Option<String>,

    /// Allocate container IP from a range.
    #[serde(rename = "IPRange")]
    pub ip_range: Vec<IpRange>,

    /// Enable IPv6 (Dual Stack) networking.
    #[serde(rename = "IPv6", skip_serializing_if = "Not::not")]
    pub ipv6: bool,

    /// Set one or more OCI labels on the network.
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub label: Vec<String>,

    /// Set driver specific options.
    pub options: Vec<String>,

    /// This key contains a list of arguments passed directly to the end of the `podman network create`
    /// command in the generated file, right before the name of the network in the command line.
    pub podman_args: Option<String>,

    /// The subnet in CIDR notation.
    pub subnet: Vec<IpNet>,
}

impl Network {
    /// Add `--{flag} {arg}` to `PodmanArgs=`.
    fn push_arg(&mut self, flag: &str, arg: &str) {
        let podman_args = self.podman_args.get_or_insert_with(String::new);
        if !podman_args.is_empty() {
            podman_args.push(' ');
        }
        podman_args.push_str("--");
        podman_args.push_str(flag);
        podman_args.push(' ');
        podman_args.push_str(arg);
    }
}

impl Downgrade for Network {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_7 {
            for dns in std::mem::take(&mut self.dns) {
                self.push_arg("dns", &dns);
            }
        }

        if version < PodmanVersion::V4_6 {
            if let Some(podman_args) = self.podman_args.take() {
                return Err(DowngradeError::Option {
                    quadlet_option: "PodmanArgs",
                    value: podman_args,
                    supported_version: PodmanVersion::V4_6,
                });
            }
        }

        Ok(())
    }
}

impl TryFrom<compose_spec::Network> for Network {
    type Error = color_eyre::Report;

    fn try_from(
        compose_spec::Network {
            driver,
            driver_opts,
            attachable,
            enable_ipv6: ipv6,
            ipam,
            internal,
            labels,
            name,
            extensions,
        }: compose_spec::Network,
    ) -> Result<Self, Self::Error> {
        let Ipam {
            driver: ipam_driver,
            config: ipam_config,
            options: ipam_options,
            extensions: ipam_extensions,
        } = ipam.unwrap_or_default();

        let unsupported_options = [
            ("attachable", !attachable),
            ("name", name.is_none()),
            ("ipam.options", ipam_options.is_empty()),
        ];
        for (option, not_present) in unsupported_options {
            ensure!(not_present, "`{option}` is not supported");
        }
        ensure!(
            extensions.is_empty() && ipam_extensions.is_empty(),
            "compose extensions are not supported"
        );

        let network = Self {
            driver: driver.map(Into::into),
            options: driver_opts
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
            ipv6,
            ipam_driver,
            internal,
            label: labels.into_list().into_iter().collect(),
            ..Self::default()
        };

        ipam_config.into_iter().enumerate().try_fold(
            network,
            |mut network,
             (
                index,
                IpamConfig {
                    subnet,
                    ip_range,
                    gateway,
                    aux_addresses,
                    extensions,
                },
            )| {
                if !aux_addresses.is_empty() {
                    Err(eyre!("`aux_addresses` is not supported"))
                } else if !extensions.is_empty() {
                    Err(eyre!("compose extensions are not supported"))
                } else {
                    network.subnet.extend(subnet);
                    network.ip_range.extend(ip_range.map(Into::into));
                    network.gateway.extend(gateway);
                    Ok(network)
                }
                .wrap_err_with(|| format!("error converting `ipam.config[{index}]`"))
            },
        )
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let network = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&network)
    }
}

/// Valid forms for `IPRange=` network quadlet option values.
#[derive(Debug, Clone, PartialEq)]
pub enum IpRange {
    Cidr(IpNet),
    Ipv4Range(Range<Ipv4Addr>),
    Ipv6Range(Range<Ipv6Addr>),
}

impl From<IpNet> for IpRange {
    fn from(value: IpNet) -> Self {
        Self::Cidr(value)
    }
}

impl Serialize for IpRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Cidr(subnet) => subnet.serialize(serializer),
            Self::Ipv4Range(Range { start, end }) => format!("{start}-{end}").serialize(serializer),
            Self::Ipv6Range(Range { start, end }) => format!("{start}-{end}").serialize(serializer),
        }
    }
}

impl FromStr for IpRange {
    type Err = ParseIpRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((start, end)) = s.split_once('-') {
            let start = start.parse().map_err(|source| ParseIpRangeError::IpAddr {
                source,
                ip_address: start.into(),
            })?;
            match start {
                IpAddr::V4(start) => {
                    let end = end.parse().map_err(|source| ParseIpRangeError::Ipv4Addr {
                        source,
                        ip_address: end.into(),
                    })?;
                    Ok(Self::Ipv4Range(start..end))
                }
                IpAddr::V6(start) => {
                    let end = end.parse().map_err(|source| ParseIpRangeError::Ipv6Addr {
                        source,
                        ip_address: end.into(),
                    })?;
                    Ok(Self::Ipv6Range(start..end))
                }
            }
        } else {
            Ok(Self::Cidr(s.parse()?))
        }
    }
}

/// Error which can be returned when parsing an [`IpRange`].
/// It must be in CIDR notation or in `<start-IP>-<end-IP>` syntax.
#[derive(Error, Debug)]
pub enum ParseIpRangeError {
    #[error("invalid subnet, must be in CIDR notation or in `<start-IP>-<end-IP>` syntax")]
    Cidr(#[from] ipnet::AddrParseError),
    #[error("invalid IP address: {ip_address}")]
    IpAddr {
        source: std::net::AddrParseError,
        ip_address: String,
    },
    #[error("invalid IPv4 address: {ip_address}")]
    Ipv4Addr {
        source: std::net::AddrParseError,
        ip_address: String,
    },
    #[error("invalid IPv6 address: {ip_address}")]
    Ipv6Addr {
        source: std::net::AddrParseError,
        ip_address: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_default_empty() {
        let network = Network::default();
        assert_eq!(network.to_string(), "[Network]\n");
    }
}
