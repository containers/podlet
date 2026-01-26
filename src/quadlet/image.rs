use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter, Write},
    ops::Not,
    path::PathBuf,
    str::FromStr,
};

use serde::{Serialize, Serializer};

use super::{Downgrade, DowngradeError, HostPaths, PodmanVersion, ResourceKind};

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
    pub decryption_key: Option<DecryptionKey>,

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

impl HostPaths for Image {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        let decryption_key = self
            .decryption_key
            .as_mut()
            .map(|decryption_key| &mut decryption_key.key);

        self.auth_file
            .iter_mut()
            .chain(&mut self.cert_dir)
            .chain(decryption_key)
    }
}

impl Downgrade for Image {
    #[allow(clippy::unused_self)]
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        if version < PodmanVersion::V4_8 {
            return Err(DowngradeError::Kind {
                kind: ResourceKind::Image,
                supported_version: PodmanVersion::V4_8,
            });
        }

        Ok(())
    }
}

/// The key and optional passphrase for decryption of images.
///
/// See the `--decryption-key` section of
/// [**podman-pull(1)**](https://docs.podman.io/en/stable/markdown/podman-pull.1.html#decryption-key-key-passphrase).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecryptionKey {
    pub key: PathBuf,
    pub passphrase: Option<String>,
}

impl DecryptionKey {
    /// Parse a [`DecryptionKey`] from a string.
    ///
    /// The format is "key\[:passphrase\]".
    fn parse<T>(key: T) -> Self
    where
        T: AsRef<str> + Into<PathBuf>,
    {
        if let Some((key, passphrase)) = key.as_ref().split_once(':') {
            Self {
                key: key.into(),
                passphrase: Some(passphrase.to_owned()),
            }
        } else {
            Self {
                key: key.into(),
                passphrase: None,
            }
        }
    }
}

impl From<String> for DecryptionKey {
    fn from(value: String) -> Self {
        Self::parse(value)
    }
}

impl From<&str> for DecryptionKey {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl FromStr for DecryptionKey {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for DecryptionKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self { key, passphrase } = self;
        // Format is "key[:passphrase]".

        key.display().fmt(f)?;

        if let Some(passphrase) = passphrase {
            f.write_char(':')?;
            f.write_str(passphrase)?;
        }

        Ok(())
    }
}

impl Serialize for DecryptionKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
