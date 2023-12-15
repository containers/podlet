use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::serde::quadlet::quote_spaces_join_space;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    /// Add weak parent dependencies to the unit.
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub wanted_by: Vec<String>,

    /// Add stronger parent dependencies to the unit.
    #[serde(
        serialize_with = "quote_spaces_join_space",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub required_by: Vec<String>,
}

impl Display for Install {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let install = crate::serde::quadlet::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&install)
    }
}
