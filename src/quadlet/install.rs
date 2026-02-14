use serde::Serialize;

use crate::serde::quadlet::seq_quote_whitespace;

/// The `[Install]` section of a systemd unit / Quadlet file.
#[derive(Serialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    /// Add weak parent dependencies to the unit.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub wanted_by: Vec<String>,

    /// Add stronger parent dependencies to the unit.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub required_by: Vec<String>,
}

impl Install {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        let Self {
            wanted_by,
            required_by,
        } = self;

        wanted_by.is_empty() && required_by.is_empty()
    }
}
