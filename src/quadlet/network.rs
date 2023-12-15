use std::{
    fmt::{self, Display, Formatter},
    net::IpAddr,
    ops::Not,
};

use color_eyre::eyre::{self, Context};
use ipnet::IpNet;
use serde::Serialize;

use crate::serde::quadlet::quote_spaces_join_space;

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Network {
    /// If enabled, disables the DNS plugin for this network.
    #[serde(skip_serializing_if = "Not::not")]
    pub disable_dns: bool,

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
    pub ip_range: Vec<IpNet>,

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_default_empty() {
        let network = Network::default();
        assert_eq!(network.to_string(), "[Network]\n");
    }
}
