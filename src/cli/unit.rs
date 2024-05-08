use std::fmt::{self, Display, Formatter};

use clap::Args;
use color_eyre::{
    eyre::{self, bail, eyre},
    Section,
};
use compose_spec::service::{Condition, Dependency};
use serde::Serialize;

use crate::serde::quadlet::quote_spaces_join_space;

// Common systemd unit options
// From [systemd.unit](https://www.freedesktop.org/software/systemd/man/systemd.unit.html)
#[allow(clippy::doc_markdown)]
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

    /// Similar to --requires, but when the dependency stops, this unit also stops
    ///
    /// Converts to "BindsTo=BINDS_TO[ ...]"
    ///
    /// Can be specified multiple times
    #[arg(long)]
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    binds_to: Vec<String>,

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
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        let Self {
            description,
            wants,
            requires,
            binds_to,
            before,
            after,
        } = self;

        description.is_none()
            && wants.is_empty()
            && requires.is_empty()
            && binds_to.is_empty()
            && before.is_empty()
            && after.is_empty()
    }

    /// Add a compose [`Service`](compose_spec::Service) [`Dependency`] to the unit.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Condition`] is not [`ServiceStarted`](Condition::ServiceStarted)
    /// or the [`Dependency`] is set to `restart` but is not `required`.
    pub fn add_dependency(
        &mut self,
        name: impl Display,
        Dependency {
            condition,
            restart,
            required,
        }: Dependency,
    ) -> eyre::Result<()> {
        match condition {
            Condition::ServiceStarted => {}
            Condition::ServiceHealthy => {
                return Err(condition_eyre(condition, "Notify=healthy", "Container"));
            }
            Condition::ServiceCompletedSuccessfully => {
                return Err(condition_eyre(condition, "Type=oneshot", "Service"));
            }
        }

        // Which list to add the dependency to depends on whether to restart this unit and if the
        // dependency is required.
        let list = match (restart, required) {
            (true, true) => &mut self.binds_to,
            (true, false) => {
                bail!("restarting a service for a dependency that is not required is unsupported");
            }
            (false, true) => &mut self.requires,
            (false, false) => &mut self.wants,
        };

        let name = format!("{name}.service");
        list.push(name.clone());
        self.after.push(name);

        Ok(())
    }
}

/// Create an [`eyre::Report`] for an unsupported compose [`Dependency`] [`Condition`].
///
/// Suggests using `option` in `section` instead.
fn condition_eyre(condition: Condition, option: &str, section: &str) -> eyre::Report {
    eyre!("dependency condition `{condition}` is not directly supported").suggestion(format!(
        "try using `{option}` in the [{section}] section of the dependency"
    ))
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let unit = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&unit)
    }
}
