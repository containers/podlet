use std::{
    fmt::{self, Display, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::{Not, Range},
    str::FromStr,
};

use color_eyre::eyre::{self, Context};
use ipnet::IpNet;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::serde::quadlet::quote_spaces_join_space;

use super::{DowngradeError, PodmanVersion};

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
    pub options: Option<String>,

    /// This key contains a list of arguments passed directly to the end of the `podman network create`
    /// command in the generated file, right before the name of the network in the command line.
    pub podman_args: Option<String>,

    /// The subnet in CIDR notation.
    pub subnet: Vec<IpNet>,
}

impl Network {
    /// Downgrade compatibility to `version`.
    ///
    /// This is a one-way transformation, calling downgrade a second time with a higher version
    /// will not increase the quadlet options used.
    ///
    /// # Errors
    ///
    /// Returns an error if a used quadlet option is incompatible with the given [`PodmanVersion`].
    pub fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_7 {
            for dns in std::mem::take(&mut self.dns) {
                self.push_arg("dns", &dns);
            }
        }

        if version < PodmanVersion::V4_6 {
            if let Some(podman_args) = self.podman_args.take() {
                return Err(DowngradeError {
                    quadlet_option: String::from("PodmanArgs"),
                    value: podman_args,
                    supported_version: PodmanVersion::V4_6,
                });
            }
        }

        Ok(())
    }

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

impl TryFrom<docker_compose_types::NetworkSettings> for Network {
    type Error = color_eyre::Report;

    fn try_from(value: docker_compose_types::NetworkSettings) -> Result<Self, Self::Error> {
        let unsupported_options = [
            ("attachable", !value.attachable),
            ("internal", !value.internal),
            ("external", value.external.is_none()),
            ("name", value.name.is_none()),
        ];
        for (option, not_present) in unsupported_options {
            eyre::ensure!(not_present, "`{option}` is not supported");
        }

        let options: Vec<String> = value
            .driver_opts
            .into_iter()
            .map(|(key, value)| {
                let value = value.as_ref().map(ToString::to_string).unwrap_or_default();
                format!("{key}={value}")
            })
            .collect();

        let mut gateway = Vec::new();
        let mut subnet = Vec::new();
        let ipam_driver = value
            .ipam
            .map(|ipam| -> color_eyre::Result<_> {
                for config in ipam.config {
                    if let Some(ip) = config.gateway {
                        gateway.push(ip.parse().wrap_err_with(|| {
                            format!("could not parse `{ip}` as a valid IP address")
                        })?);
                    }
                    subnet.push(config.subnet.parse().wrap_err_with(|| {
                        format!("could not parse `{}` as a valid IP subnet", config.subnet)
                    })?);
                }
                Ok(ipam.driver)
            })
            .transpose()
            .wrap_err("invalid ipam config")?
            .flatten();

        let label = match value.labels {
            docker_compose_types::Labels::List(labels) => labels,
            docker_compose_types::Labels::Map(labels) => labels
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        };

        Ok(Self {
            driver: value.driver,
            options: (!options.is_empty()).then(|| options.join(",")),
            ipv6: value.enable_ipv6,
            gateway,
            subnet,
            ipam_driver,
            label,
            ..Self::default()
        })
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
