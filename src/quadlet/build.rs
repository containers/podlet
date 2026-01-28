use std::{
    fmt::{self, Display, Formatter},
    iter,
    path::PathBuf,
    str::FromStr,
};

use color_eyre::eyre::{OptionExt, bail, ensure};
use compose_spec::service::build::Context;
use serde::{Serialize, Serializer};

use crate::serde::{quadlet::seq_quote_whitespace, skip_true};

use super::{
    Downgrade, DowngradeError, HostPaths, PodmanVersion, ResourceKind,
    container::{Dns, PullPolicy},
};

/// Options for the \[Build\] section of a `.build` Quadlet file.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Build {
    /// Add an image annotation (e.g. annotation=value) to the image metadata.
    pub annotation: Vec<String>,

    /// Override the architecture, defaults to the host's, of the image to be built.
    pub arch: Option<String>,

    /// Path of the authentication file.
    pub auth_file: Option<PathBuf>,

    /// Set network-scoped DNS resolver/nameserver for the build container.
    #[serde(rename = "DNS")]
    pub dns: Dns,

    /// Set custom DNS options.
    #[serde(rename = "DNSOption")]
    pub dns_option: Vec<String>,

    /// Set custom DNS search domains.
    #[serde(rename = "DNSSearch")]
    pub dns_search: Vec<String>,

    /// Add a value (e.g. env=value) to the built image.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub environment: Vec<String>,

    /// Specifies a Containerfile which contains instructions for building the image.
    pub file: Option<Context>,

    /// Always remove intermediate containers after a build, even if the build fails.
    #[serde(rename = "ForceRM", skip_serializing_if = "skip_true")]
    pub force_rm: bool,

    /// Assign additional groups to the primary user running within the container process.
    pub group_add: Vec<String>,

    /// Specifies the name which is assigned to the resulting image if the build process completes
    /// successfully.
    pub image_tag: String,

    /// Add an image label (e.g. label=value) to the image metadata.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub label: Vec<String>,

    /// Sets the configuration for network namespaces when handling `RUN` instructions.
    pub network: Vec<String>,

    /// A list of arguments passed directly to the end of the `podman build` command in the
    /// generated file.
    pub podman_args: Option<String>,

    /// Set the image pull policy.
    pub pull: Option<PullPolicy>,

    /// Pass secret information used in Containerfile build stages in a safe way.
    pub secret: Vec<Secret>,

    /// Provide context (a working directory) to `podman build`.
    pub set_working_directory: Option<Context>,

    /// Set the target build stage to build.
    pub target: Option<String>,

    /// Require HTTPS and verification of certificates when contacting registries.
    #[serde(rename = "TLSVerify")]
    pub tls_verify: Option<bool>,

    /// Override the default architecture variant of the container image to be built.
    pub variant: Option<String>,

    /// Mount a volume to containers when executing `RUN` instructions during the build.
    pub volume: Vec<String>,
}

impl HostPaths for Build {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.auth_file
            .iter_mut()
            .chain(self.file.host_paths())
            .chain(self.secret.host_paths())
            .chain(self.set_working_directory.host_paths())
    }
}

impl Downgrade for Build {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V5_2 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Build,
                supported_version: PodmanVersion::V5_2,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret {
    id: String,
    source: PathBuf,
}

impl FromStr for Secret {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = None;
        let mut source = None;

        for option in s.split(',') {
            let (option, value) = option
                .split_once('=')
                .ok_or_eyre("secret option missing `=`")?;
            match option {
                "id" => {
                    ensure!(id.is_none(), "secret `id` cannot be set multiple times");
                    id = Some(value.to_owned());
                }
                "src" => {
                    ensure!(
                        source.is_none(),
                        "secret `src` cannot be set multiple times"
                    );
                    source = Some(value.into());
                }
                option => bail!("unknown secret option `{option}`"),
            }
        }

        Ok(Self {
            id: id.ok_or_eyre("missing secret `id`")?,
            source: source.ok_or_eyre("missing secret `src`")?,
        })
    }
}

impl Display for Secret {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self { id, source } = self;
        write!(f, "id={id},src={}", source.display())
    }
}

impl Serialize for Secret {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl HostPaths for Secret {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        iter::once(&mut self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_parse() -> color_eyre::Result<()> {
        let secret = Secret {
            id: "secret".to_owned(),
            source: "/source".into(),
        };
        assert_eq!(secret, "id=secret,src=/source".parse()?);
        assert_eq!(secret, "src=/source,id=secret".parse()?);

        Ok(())
    }
}
