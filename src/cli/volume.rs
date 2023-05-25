pub mod opt;

use clap::{Args, Subcommand};

use self::opt::Opt;

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

impl From<Volume> for crate::quadlet::Volume {
    fn from(value: Volume) -> Self {
        let Volume::Create { create } = value;
        create.into()
    }
}

impl From<Volume> for crate::quadlet::Resource {
    fn from(value: Volume) -> Self {
        crate::quadlet::Volume::from(value).into()
    }
}

impl Volume {
    pub fn name(&self) -> &str {
        let Self::Create { create } = self;
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

impl From<Create> for crate::quadlet::Volume {
    fn from(value: Create) -> Self {
        Self {
            label: value.label,
            ..value.opt.into()
        }
    }
}
