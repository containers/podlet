use serde::Serialize;

use crate::serde::quadlet::seq_quote_whitespace;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    /// Add weak parent dependencies to the unit.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub wanted_by: Vec<String>,

    /// Add stronger parent dependencies to the unit.
    #[serde(serialize_with = "seq_quote_whitespace")]
    pub required_by: Vec<String>,
}
