use std::fmt::{self, Display, Formatter};

use super::escape_spaces_join;

#[derive(Debug, Clone, PartialEq)]
pub struct Install {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

impl Display for Install {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Install]")?;

        if !self.wanted_by.is_empty() {
            writeln!(f, "WantedBy={}", escape_spaces_join(&self.wanted_by))?;
        }

        if !self.required_by.is_empty() {
            writeln!(f, "RequiredBy={}", escape_spaces_join(&self.required_by))?;
        }

        Ok(())
    }
}
