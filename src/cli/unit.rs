use std::fmt::{self, Display, Formatter};

use clap::Args;
use serde::Serialize;

use crate::serde::quadlet::quote_spaces_join_space;

// Common systemd unit options
// From [systemd.unit](https://www.freedesktop.org/software/systemd/man/systemd.unit.html)
#[derive(Serialize, Args, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Unit {
    /// Add a description to the unit
    ///
    /// A description should be a short, human readable title of the unit
    ///
    /// Converts to "Description=DESCRIPTION"
    #[arg(short, long)]
    description: Option<String>,

    /// Add (weak) requirement dependencies to the unit
    ///
    /// Converts to "Wants=WANTS[ ...]"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    wants: Vec<String>,

    /// Similar to --wants, but adds stronger requirement dependencies
    ///
    /// Converts to "Requires=REQUIRES[ ...]"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    requires: Vec<String>,

    /// Configure ordering dependency between units
    ///
    /// Converts to "Before=BEFORE[ ...]"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    before: Vec<String>,

    /// Configure ordering dependency between units
    ///
    /// Converts to "After=AFTER[ ...]"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    after: Vec<String>,
}

impl Unit {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /*
    pub fn add_dependencies(&mut self, depends_on: docker_compose_types::DependsOnOptions) {
        let depends_on = match depends_on {
            docker_compose_types::DependsOnOptions::Simple(vec) => vec,
            docker_compose_types::DependsOnOptions::Conditional(map) => map.into_keys().collect(),
        };

        self.requires.extend(
            depends_on
                .into_iter()
                .map(|dependency| dependency + ".service"),
        );
    }
    */
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let unit = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&unit)
    }
}
