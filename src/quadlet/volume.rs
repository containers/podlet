use std::fmt::{self, Display, Formatter};

use super::escape_spaces_join;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Volume {
    pub copy: bool,
    pub device: Option<String>,
    pub group: Option<String>,
    pub label: Vec<String>,
    pub options: Option<String>,
    pub fs_type: Option<String>,
    pub user: Option<String>,
}

impl Display for Volume {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "[Volume]")?;

        if self.copy {
            writeln!(f, "Copy=true")?;
        }

        if let Some(device) = &self.device {
            writeln!(f, "Device={device}")?;
        }

        if let Some(group) = &self.group {
            writeln!(f, "Group={group}")?;
        }

        if !self.label.is_empty() {
            writeln!(f, "Label={}", escape_spaces_join(&self.label))?;
        }

        if let Some(options) = &self.options {
            writeln!(f, "Options={options}")?;
        }

        if let Some(fs_type) = &self.fs_type {
            writeln!(f, "Type={fs_type}")?;
        }

        if let Some(user) = &self.user {
            writeln!(f, "User={user}")?;
        }

        Ok(())
    }
}
