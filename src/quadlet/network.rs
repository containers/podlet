use std::{
    fmt::{self, Display, Formatter},
    net::IpAddr,
};

use color_eyre::eyre::{self, Context};
use ipnet::IpNet;

use super::escape_spaces_join;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Network {
    pub disable_dns: bool,
    pub driver: Option<String>,
    pub gateway: Vec<IpAddr>,
    pub internal: bool,
    pub ipam_driver: Option<String>,
    pub ip_range: Vec<IpNet>,
    pub ipv6: bool,
    pub label: Vec<String>,
    pub options: Option<String>,
    pub subnet: Vec<IpNet>,
}

impl TryFrom<docker_compose_types::NetworkSettings> for Network {
    type Error = color_eyre::Report;

    fn try_from(value: docker_compose_types::NetworkSettings) -> Result<Self, Self::Error> {
        let unsupported_options = [
            ("attachable", value.attachable),
            ("internal", value.internal),
            ("external", value.external.is_some()),
            ("name", value.name.is_some()),
        ];
        for (option, exists) in unsupported_options {
            if exists {
                return Err(eyre::eyre!("`{option}` is not supported"));
            }
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
        writeln!(f, "[Network]")?;

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

        if let Some(options) = &self.options {
            writeln!(f, "Options={options}")?;
        }

        for subnet in &self.subnet {
            writeln!(f, "Subnet={subnet}")?;
        }

        Ok(())
    }
}
