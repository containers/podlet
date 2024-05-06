use std::{path::PathBuf, str::FromStr};

use clap::{Args, Subcommand};
use thiserror::Error;

use crate::quadlet::{self, image::DecryptionKey};

use super::image_to_name;

/// [`Subcommand`]s for `podlet podman image`
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Image {
    /// Generate a podman quadlet `.image` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-pull.1.html and
    /// https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html#image-units-image
    #[allow(clippy::doc_markdown)]
    #[group(skip)]
    Pull {
        #[command(flatten)]
        pull: Pull,
    },
}

impl Image {
    /// Name suitable for use as the filename of the generated quadlet file.
    pub fn name(&self) -> &str {
        let Self::Pull {
            pull: Pull { source, .. },
        } = self;

        image_to_name(source)
    }
}

impl From<Image> for quadlet::Image {
    fn from(value: Image) -> Self {
        let Image::Pull { pull } = value;
        pull.into()
    }
}

impl From<Image> for quadlet::Resource {
    fn from(value: Image) -> Self {
        quadlet::Image::from(value).into()
    }
}

/// [`Args`] for `podman image pull`
#[allow(clippy::doc_markdown)]
#[derive(Args, Default, Debug, Clone, PartialEq)]
pub struct Pull {
    /// All tagged images in the repository are pulled.
    ///
    /// Converts to "AllTags=true"
    #[arg(short, long)]
    pub all_tags: bool,

    /// Override the architecture, defaults to hosts, of the image to be pulled.
    ///
    /// Converts to "Arch=ARCH"
    #[arg(long)]
    pub arch: Option<String>,

    /// Path of the authentication file.
    ///
    /// Converts to "AuthFile=PATH"
    #[arg(long, value_name = "PATH")]
    pub authfile: Option<PathBuf>,

    /// Use certificates at path (*.crt, *.cert, *.key) to connect to the registry.
    ///
    /// Converts to "CertDir=PATH"
    #[arg(long, value_name = "PATH")]
    pub cert_dir: Option<PathBuf>,

    /// The username and/or password to use to authenticate with the registry, if required.
    ///
    /// Converts to "Creds=[USERNAME][:PASSWORD]"
    #[arg(long, value_name = "[USERNAME][:PASSWORD]")]
    pub creds: Option<String>,

    /// The key and optional passphrase to be used for decryption of images.
    ///
    /// Converts to "DecryptionKey=KEY[:PASSPHRASE]"
    #[arg(long, value_name = "KEY[:PASSPHRASE]")]
    pub decryption_key: Option<DecryptionKey>,

    /// Docker-specific option to disable image verification to a container registry.
    ///
    /// Not supported by Podman
    ///
    /// This option is a NOOP and provided solely for scripting compatibility.
    #[arg(long)]
    pub disable_content_trust: bool,

    /// Override the OS, defaults to hosts, of the image to be pulled.
    ///
    /// Converts to "OS=OS"
    #[arg(long)]
    pub os: Option<String>,

    /// Specify the platform for selecting the image.
    ///
    /// Converts to "OS=OS" and "Arch=ARCH"
    #[arg(long, conflicts_with_all = ["os", "arch"], value_name = "OS/ARCH")]
    pub platform: Option<Platform>,

    /// Require HTTPS and verify certificates when contacting registries
    ///
    /// Converts to "TLSVerify=TLS_VERIFY"
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    pub tls_verify: Option<bool>,

    /// Use the given variant instead of the running architecture variant for choosing images.
    ///
    /// Converts to "Variant=VARIANT"
    #[arg(long)]
    pub variant: Option<String>,

    /// Location from which the container image is pulled from.
    ///
    /// Converts to "Image=SOURCE"
    pub source: String,
}

impl From<Pull> for quadlet::Image {
    fn from(
        Pull {
            all_tags,
            arch,
            authfile: auth_file,
            cert_dir,
            creds,
            decryption_key,
            disable_content_trust: _,
            os,
            platform,
            tls_verify,
            variant,
            source: image,
        }: Pull,
    ) -> Self {
        let (os, arch) = platform.map_or((os, arch), |platform| {
            (Some(platform.os), Some(platform.arch))
        });

        Self {
            all_tags,
            arch,
            auth_file,
            cert_dir,
            creds,
            decryption_key,
            image,
            image_tag: None,
            os,
            podman_args: None,
            tls_verify,
            variant,
        }
    }
}

/// `podman image pull --platform` option
#[derive(Debug, Clone, PartialEq)]
pub struct Platform {
    pub os: String,
    pub arch: String,
}

impl FromStr for Platform {
    type Err = ParsePlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (os, arch) = s.split_once('/').ok_or(ParsePlatformError::MissingArch)?;
        Ok(Self {
            os: os.to_owned(),
            arch: arch.to_owned(),
        })
    }
}

#[derive(Error, Debug)]
pub enum ParsePlatformError {
    #[error("platform must be in the form \"OS/ARCH\"")]
    MissingArch,
}
