use clap::Args;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Install {
    /// Add an [Install] section to the unit
    ///
    /// By default, if the --wanted-by and --required-by options are not used,
    /// the section will have "WantedBy=default.target".
    #[allow(clippy::struct_field_names)]
    #[arg(short, long)]
    pub install: bool,

    /// Add (weak) parent dependencies to the unit
    ///
    /// Requires the --install option
    ///
    /// Converts to "WantedBy=WANTED_BY"
    ///
    /// Can be specified multiple times
    #[arg(long, requires = "install")]
    wanted_by: Vec<String>,

    /// Similar to --wanted-by, but adds stronger parent dependencies
    ///
    /// Requires the --install option
    ///
    /// Converts to "RequiredBy=REQUIRED_BY"
    ///
    /// Can be specified multiple times
    #[arg(long, requires = "install")]
    required_by: Vec<String>,
}

impl From<Install> for crate::quadlet::Install {
    fn from(value: Install) -> Self {
        Self {
            wanted_by: if value.wanted_by.is_empty() && value.required_by.is_empty() {
                vec![String::from("default.target")]
            } else {
                value.wanted_by
            },
            required_by: value.required_by,
        }
    }
}
