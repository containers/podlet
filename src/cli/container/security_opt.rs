use std::{path::PathBuf, str::FromStr};

use thiserror::Error;

use crate::quadlet::container::Unmask;

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityOpt {
    Apparmor(String),
    Label(LabelOpt),
    Mask(String),
    NoNewPrivileges,
    Seccomp(PathBuf),
    ProcOpts(String),
    Unmask(String),
}

impl FromStr for SecurityOpt {
    type Err = ParseSecurityOptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(policy) = s.strip_prefix("apparmor=") {
            Ok(Self::Apparmor(policy.to_owned()))
        } else if let Some(label) = s.strip_prefix("label=") {
            Ok(Self::Label(label.parse()?))
        } else if let Some(mask) = s.strip_prefix("mask=") {
            Ok(Self::Mask(mask.to_owned()))
        } else if s == "no-new-privileges" {
            Ok(Self::NoNewPrivileges)
        } else if let Some(profile) = s.strip_prefix("seccomp=") {
            Ok(Self::Seccomp(profile.into()))
        } else if let Some(opts) = s.strip_prefix("proc-opts=") {
            Ok(Self::ProcOpts(opts.to_owned()))
        } else if let Some(unmask) = s.strip_prefix("unmask=") {
            Ok(Self::Unmask(unmask.to_owned()))
        } else {
            Err(ParseSecurityOptError::InvalidSecurityOpt(s.to_owned()))
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseSecurityOptError {
    #[error(transparent)]
    InvalidLabelOpt(#[from] InvalidLabelOpt),
    #[error("`{0}` is not a valid security option")]
    InvalidSecurityOpt(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelOpt {
    User(String),
    Role(String),
    Type(String),
    Level(String),
    Filetype(String),
    Disable,
    Nested,
}

impl FromStr for LabelOpt {
    type Err = InvalidLabelOpt;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(user) = s.strip_prefix("user:") {
            Ok(Self::User(user.to_owned()))
        } else if let Some(role) = s.strip_prefix("role:") {
            Ok(Self::Role(role.to_owned()))
        } else if let Some(label_type) = s.strip_prefix("type:") {
            Ok(Self::Type(label_type.to_owned()))
        } else if let Some(level) = s.strip_prefix("level:") {
            Ok(Self::Level(level.to_owned()))
        } else if let Some(filetype) = s.strip_prefix("filetype:") {
            Ok(Self::Filetype(filetype.to_owned()))
        } else if s == "disable" {
            Ok(Self::Disable)
        } else if s == "nested" {
            Ok(Self::Nested)
        } else {
            Err(InvalidLabelOpt(s.to_owned()))
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("`{0}` is not a valid label option")]
pub struct InvalidLabelOpt(pub String);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QuadletOptions {
    pub mask: Vec<String>,
    pub no_new_privileges: bool,
    pub seccomp_profile: Option<PathBuf>,
    pub security_label_disable: bool,
    pub security_label_file_type: Option<String>,
    pub security_label_level: Option<String>,
    pub security_label_nested: bool,
    pub security_label_type: Option<String>,
    pub unmask: Option<Unmask>,
    pub podman_args: Vec<String>,
}

impl QuadletOptions {
    pub fn add_security_opt(&mut self, security_opt: SecurityOpt) {
        match security_opt {
            SecurityOpt::Apparmor(policy) => self.podman_args.push(format!("apparmor={policy}")),
            SecurityOpt::Label(label_opt) => self.add_label_opt(label_opt),
            SecurityOpt::Mask(mask) => self.mask.extend(mask.split(':').map(Into::into)),
            SecurityOpt::NoNewPrivileges => self.no_new_privileges = true,
            SecurityOpt::Seccomp(profile) => self.seccomp_profile = Some(profile),
            SecurityOpt::ProcOpts(proc_opts) => {
                self.podman_args.push(format!("proc-opts={proc_opts}"));
            }
            SecurityOpt::Unmask(paths) => self
                .unmask
                .get_or_insert_with(Unmask::new)
                .extend(paths.split(':')),
        }
    }

    pub fn add_label_opt(&mut self, label_opt: LabelOpt) {
        match label_opt {
            LabelOpt::User(user) => self.podman_args.push(format!("label=user:{user}")),
            LabelOpt::Role(role) => self.podman_args.push(format!("label=role:{role}")),
            LabelOpt::Type(label_type) => self.security_label_type = Some(label_type),
            LabelOpt::Level(level) => self.security_label_level = Some(level),
            LabelOpt::Filetype(file_type) => self.security_label_file_type = Some(file_type),
            LabelOpt::Disable => self.security_label_disable = true,
            LabelOpt::Nested => self.security_label_nested = true,
        }
    }
}
