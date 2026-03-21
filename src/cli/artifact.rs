//! The `podlet podman artifact pull` command and CLI options.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::{
    cli::image_to_name,
    quadlet::{self, image::DecryptionKey},
};

/// [`Subcommand`]s for `podlet podman artifact`
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Artifact {
    /// Generate a Podman Quadlet `.artifact` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-pull.1.html and
    /// https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html#artifact-units-artifact
    #[expect(clippy::doc_markdown, reason = "help links")]
    #[group(skip)]
    Pull {
        #[command(flatten)]
        pull: Pull,
    },
}

impl Artifact {
    /// Name suitable for use as the filename of the generated Quadlet file.
    pub fn name(&self) -> &str {
        let Self::Pull {
            pull: Pull { source, .. },
        } = self;

        image_to_name(source)
    }
}

impl From<Artifact> for quadlet::Artifact {
    fn from(value: Artifact) -> Self {
        let Artifact::Pull { pull } = value;
        pull.into()
    }
}

impl From<Artifact> for quadlet::Resource {
    fn from(value: Artifact) -> Self {
        quadlet::Artifact::from(value).into()
    }
}

/// [`Args`] for `podman artifact pull`
#[derive(Args, Default, Debug, Clone, PartialEq)]
pub struct Pull {
    /// Path of the authentication file.
    ///
    /// Converts to "AuthFile=PATH".
    #[arg(long, value_name = "PATH")]
    pub authfile: Option<PathBuf>,

    /// Use certificates at path (*.crt, *.cert, *.key) to connect to the registry.
    ///
    /// Converts to "CertDir=PATH".
    #[arg(long, value_name = "PATH")]
    pub cert_dir: Option<PathBuf>,

    /// The username and/or password to use to authenticate with the registry, if required.
    ///
    /// Converts to "Creds=[USERNAME][:PASSWORD]".
    #[arg(long, value_name = "[USERNAME][:PASSWORD]")]
    pub creds: Option<String>,

    /// The key and optional passphrase to be used for decryption of artifacts.
    ///
    /// Converts to "DecryptionKey=KEY[:PASSPHRASE]"
    #[arg(long, value_name = "KEY[:PASSPHRASE]")]
    pub decryption_key: Option<DecryptionKey>,

    /// Suppress output information when pulling artifacts.
    #[arg(short, long)]
    pub quiet: bool,

    /// Number of times to retry pulling artifacts.
    ///
    /// Converts to "Retry=ATTEMPTS".
    ///
    /// Default is 3.
    #[arg(long, value_name = "ATTEMPTS")]
    #[arg(long)]
    pub retry: Option<u64>,

    /// Duration of delay between retry attempts when pulling artifacts.
    ///
    /// Converts to "RetryDelay=DURATION".
    ///
    /// Default is to start at two seconds and then exponentially back off.
    #[arg(long, value_name = "DURATION")]
    pub retry_delay: Option<String>,

    /// Require HTTPS and verify certificates when contacting registries.
    ///
    /// Converts to "TLSVerify=TLS_VERIFY".
    #[expect(clippy::doc_markdown, reason = "Quadlet option")]
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    pub tls_verify: Option<bool>,

    /// The location from which the artifact image is obtained.
    pub source: String,
}

impl From<Pull> for quadlet::Artifact {
    fn from(
        Pull {
            authfile,
            cert_dir,
            creds,
            decryption_key,
            quiet,
            retry,
            retry_delay,
            tls_verify,
            source,
        }: Pull,
    ) -> Self {
        Self {
            artifact: source,
            auth_file: authfile,
            cert_dir,
            creds,
            decryption_key,
            podman_args: None,
            quiet,
            retry,
            retry_delay,
            tls_verify,
        }
    }
}
