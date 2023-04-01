use std::fmt::Display;

use clap::Args;

use crate::cli::escape_spaces_join;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Install {
    /// Add an [Install] section to the unit
    ///
    /// By default, if the --wanted-by and --required-by options are not used,
    /// the section will have "WantedBy=default.target".
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

impl Display for Install {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Install]")?;

        if self.wanted_by.is_empty() && self.required_by.is_empty() {
            writeln!(f, "WantedBy=default.target")?;
        }

        if !self.wanted_by.is_empty() {
            writeln!(f, "WantedBy={}", escape_spaces_join(&self.wanted_by))?;
        }

        if !self.required_by.is_empty() {
            writeln!(f, "RequiredBy={}", escape_spaces_join(&self.required_by))?;
        }

        Ok(())
    }
}
