use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Kube {
    pub config_map: Vec<PathBuf>,
    pub log_driver: Option<String>,
    pub network: Vec<String>,
    pub publish_port: Vec<String>,
    pub user_ns: Option<String>,
    pub yaml: String,
}

impl Display for Kube {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Kube]")?;

        writeln!(f, "Yaml={}", self.yaml)?;

        for config_map in &self.config_map {
            writeln!(f, "ConfigMap={}", config_map.display())?;
        }

        if let Some(log_driver) = &self.log_driver {
            writeln!(f, "LogDriver={log_driver}")?;
        }

        for network in &self.network {
            writeln!(f, "Network={network}")?;
        }

        for port in &self.publish_port {
            writeln!(f, "PublishPort={port}")?;
        }

        if let Some(user_ns) = &self.user_ns {
            writeln!(f, "UserNS={user_ns}")?;
        }

        Ok(())
    }
}
