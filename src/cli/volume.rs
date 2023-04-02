mod opt;

use std::fmt::Display;

use clap::{Args, Subcommand};

use self::opt::Opt;
use super::escape_spaces_join;

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Volume {
    /// Generate a podman quadlet `.volume` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-volume-create.1.html and
    /// https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#volume-units-volume
    #[group(skip)]
    Create {
        #[command(flatten)]
        create: Create,
    },
}

impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Create { create } = self;
        writeln!(f, "[Volume]")?;
        write!(f, "{create}")
    }
}

impl Volume {
    pub fn name(&self) -> &str {
        let Volume::Create { create } = self;
        &create.name
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Create {
    /// Set driver specific options
    ///
    /// "copy" converts to "Copy=true"
    ///
    /// "device=DEVICE" converts to "Device=DEVICE"
    ///
    /// "type=TYPE" converts to "Type=TYPE"
    ///
    /// "o=uid=UID" converts to "User=UID"
    ///
    /// "o=gid=GID" converts to "Group=GID"
    ///
    /// "o=OPTIONS" converts to "Options=OPTIONS"
    ///
    /// Can be specified multiple times
    #[arg(short, long, value_name = "OPTION")]
    opt: Vec<Opt>,

    /// Set one or more OCI labels on the volume
    ///
    /// Converts to "Label=KEY=VALUE"
    ///
    /// Can be specified multiple times
    #[arg(short, long, value_name = "KEY=VALUE")]
    label: Vec<String>,

    /// The name of the volume to create
    ///
    /// This will be used as the name of the generated file when used with
    /// the --file option without a filename
    name: String,
}

impl Display for Create {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut mount_options = Vec::new();
        for opt in &self.opt {
            match opt {
                Opt::Type(opt_type) => writeln!(f, "Type={opt_type}")?,
                Opt::Device(device) => writeln!(f, "Device={device}")?,
                Opt::Copy => writeln!(f, "Copy=true")?,
                Opt::Mount(options) => {
                    for option in options {
                        match option {
                            opt::Mount::Uid(uid) => writeln!(f, "User={uid}")?,
                            opt::Mount::Gid(gid) => writeln!(f, "Group={gid}")?,
                            opt::Mount::Other(option) => mount_options.push(option),
                        }
                    }
                }
            }
        }
        if !mount_options.is_empty() {
            writeln!(
                f,
                "Options={}",
                mount_options
                    .into_iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }

        if !self.label.is_empty() {
            writeln!(f, "Label={}", escape_spaces_join(&self.label))?;
        }

        Ok(())
    }
}
