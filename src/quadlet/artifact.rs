//! The [`Artifact`] Quadlet type.

use std::{ops::Not, path::PathBuf};

use serde::Serialize;

use crate::quadlet::{
    Downgrade, DowngradeError, HostPaths, PodmanVersion, ResourceKind, image::DecryptionKey,
};

/// Options for the \[Artifact\] section of a `.artifact` Quadlet file.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Artifact {
    /// The artifact to pull from a registry onto the local machine.
    #[expect(clippy::struct_field_names, reason = "Quadlet option")]
    pub artifact: String,

    /// Path of the authentication file.
    pub auth_file: Option<PathBuf>,

    /// Use certificates at path (*.crt, *.cert, *.key) to connect to the registry.
    pub cert_dir: Option<PathBuf>,

    /// The credentials to use when contacting the registry in the format `[username[:password]]`.
    pub creds: Option<String>,

    /// The key and optional passphrase to be used for decryption of artifacts.
    pub decryption_key: Option<DecryptionKey>,

    /// A list of arguments passed directly to the end of the `podman artifact pull` command in the
    /// generated file.
    pub podman_args: Option<String>,

    /// Suppress output information when pulling artifacts.
    #[serde(skip_serializing_if = "Not::not")]
    pub quiet: bool,

    /// Number of times to retry the artifact pull when a HTTP error occurs.
    pub retry: Option<u64>,

    /// Delay between retries.
    pub retry_delay: Option<String>,

    /// Require HTTPS and verification of certificates when contacting registries.
    #[serde(rename = "TLSVerify")]
    pub tls_verify: Option<bool>,
}

impl HostPaths for Artifact {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.auth_file
            .iter_mut()
            .chain(&mut self.cert_dir)
            .chain(self.decryption_key.host_paths())
    }
}

impl Downgrade for Artifact {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V5_7 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Artifact,
                supported_version: PodmanVersion::V5_7,
            });
        }

        Ok(())
    }
}
