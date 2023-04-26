use std::{
    fmt::{self, Display, Formatter},
    net::IpAddr,
};

use ipnet::IpNet;

use super::escape_spaces_join;

#[derive(Debug, Clone, PartialEq)]
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
