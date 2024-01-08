use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
    path::PathBuf,
};

use serde::Serialize;

use super::{DowngradeError, PodmanVersion, ResourceKind};

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Image {
    /// All tagged images in the repository are pulled.
    #[serde(skip_serializing_if = "Not::not")]
    pub all_tags: bool,

    /// Override the architecture, defaults to hosts, of the image to be pulled.
    pub arch: Option<String>,

    /// Path of the authentication file.
    pub auth_file: Option<PathBuf>,

    /// Use certificates at path (*.crt, *.cert, *.key) to connect to the registry.
    pub cert_dir: Option<PathBuf>,

    /// The username and/or password to use to authenticate with the registry, if required.
    pub creds: Option<String>,

    /// The key and optional passphrase to be used for decryption of images.
    pub decryption_key: Option<String>,

    /// The image to pull.
    #[allow(clippy::struct_field_names)]
    pub image: String,

    /// Actual FQIN of the referenced Image.
    /// Only meaningful when source is a file or directory archive.
    #[allow(clippy::struct_field_names)]
    pub image_tag: Option<String>,

    /// Override the OS, defaults to hosts, of the image to be pulled.
    #[serde(rename = "OS")]
    pub os: Option<String>,

    /// A list of arguments passed directly to the end of the `podman image pull` command in the
    /// generated file.
    pub podman_args: Option<String>,

    /// Require HTTPS and verification of certificates when contacting registries.
    #[serde(rename = "TLSVerify")]
    pub tls_verify: Option<bool>,

    /// Override the default architecture variant of the container image.
    pub variant: Option<String>,
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let image = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&image)
    }
}

impl Image {
    /// Downgrade compatibility to `version`.
    ///
    /// This is a one-way transformation, calling downgrade a second time with a higher version
    /// will not increase the quadlet options used.
    ///
    /// # Errors
    ///
    /// Returns an error if the given [`PodmanVersion`] does not support `.image` quadlet files,
    /// or a used quadlet option is incompatible with it.
    #[allow(clippy::unused_self)]
    pub fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_8 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Image,
                supported_version: PodmanVersion::V4_8,
            });
        }

        Ok(())
    }
}
