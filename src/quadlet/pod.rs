use std::path::PathBuf;

use serde::Serialize;

use super::{
    Downgrade, DowngradeError, HostPaths, PodmanVersion, ResourceKind,
    container::{Dns, Volume},
};

/// Options for the \[Pod\] section of a `.pod` Quadlet file.
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Pod {
    /// Add host-to-IP mapping to `/etc/hosts`.
    pub add_host: Vec<String>,

    /// Set network-scoped DNS resolver/nameserver for containers in this pod.
    #[serde(rename = "DNS")]
    pub dns: Dns,

    /// Set custom DNS options.
    #[serde(rename = "DNSOption")]
    pub dns_option: Vec<String>,

    /// Set custom DNS search domains.
    #[serde(rename = "DNSSearch")]
    pub dns_search: Vec<String>,

    /// Specify a custom network for the pod.
    pub network: Vec<String>,

    /// Add a network-scoped alias for the pod.
    pub network_alias: Vec<String>,

    /// A list of arguments passed directly to the end of the `podman pod create` command in the
    /// generated file.
    pub podman_args: Option<String>,

    /// The name of the Podman pod.
    ///
    /// If not set, the default value is `systemd-%N`.
    #[allow(clippy::struct_field_names)]
    pub pod_name: Option<String>,

    /// Exposes a port, or a range of ports, from the pod to the host.
    pub publish_port: Vec<String>,

    /// Mount a volume in the pod.
    pub volume: Vec<Volume>,
}

impl HostPaths for Pod {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.volume.host_paths()
    }
}

impl Downgrade for Pod {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V5_3 {
            self.remove_v5_3_options();
        }

        if version < PodmanVersion::V5_2 {
            for network_alias in std::mem::take(&mut self.network_alias) {
                self.push_arg("network-alias", &network_alias);
            }
        }

        if version < PodmanVersion::V5_0 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Pod,
                supported_version: PodmanVersion::V5_0,
            });
        }

        Ok(())
    }
}

impl Pod {
    /// Remove Quadlet options added in Podman v5.3.0.
    fn remove_v5_3_options(&mut self) {
        for add_host in std::mem::take(&mut self.add_host) {
            self.push_arg("add-host", &add_host);
        }

        match std::mem::take(&mut self.dns) {
            Dns::None => self.push_arg("dns", "none"),
            Dns::Custom(ip_addrs) => {
                for ip_addr in ip_addrs {
                    self.push_arg("dns", &ip_addr.to_string());
                }
            }
        }

        for dns_option in std::mem::take(&mut self.dns_option) {
            self.push_arg("dns-option", &dns_option);
        }

        for dns_search in std::mem::take(&mut self.dns_search) {
            self.push_arg("dns-search", &dns_search);
        }
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
        if arg.contains(char::is_whitespace) {
            podman_args.push('"');
            podman_args.push_str(arg);
            podman_args.push('"');
        } else {
            podman_args.push_str(arg);
        }
    }
}
