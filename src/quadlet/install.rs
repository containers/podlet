use std::fmt::{self, Display, Formatter};

use super::writeln_escape_spaces;

#[derive(Debug, Clone, PartialEq)]
pub struct Install {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

impl Display for Install {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Install]")?;

        if !self.wanted_by.is_empty() {
            writeln_escape_spaces(f, "WantedBy", &self.wanted_by)?;
        }

        if !self.required_by.is_empty() {
            writeln_escape_spaces(f, "RequiredBy", &self.required_by)?;
        }

        Ok(())
    }
}
