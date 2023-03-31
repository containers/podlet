use std::fmt::Display;

use clap::Args;

/// Common systemd unit options
///
/// From [systemd.unit](https://www.freedesktop.org/software/systemd/man/systemd.unit.html)
#[derive(Args, Debug, Clone, PartialEq)]
pub struct Unit {
    /// Add a description to the unit
    ///
    /// A description should be a short, human readable title of the unit
    ///
    /// Converts to "Description=DESCRIPTION"
    #[arg(short, long)]
    description: Option<String>,
}

impl Unit {
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Unit]")?;

        if let Some(description) = &self.description {
            writeln!(f, "Description={description}")?;
        }

        Ok(())
    }
}
