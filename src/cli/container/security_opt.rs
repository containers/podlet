use std::str::FromStr;

use thiserror::Error;

use super::Output;

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityOpt {
    Apparmor(String),
    Label(LabelOpt),
    Mask(String),
    NoNewPrivileges,
    Seccomp(String),
    ProcOpts(String),
    Unmask(String),
}

impl FromStr for SecurityOpt {
    type Err = ParseSecurityOptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("apparmor=") => {
                let (_, policy) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::Apparmor(String::from(policy)))
            }
            s if s.starts_with("label=") => {
                let (_, label) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::Label(label.parse()?))
            }
            s if s.starts_with("mask=") => {
                let (_, mask) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::Mask(String::from(mask)))
            }
            "no-new-privileges" => Ok(SecurityOpt::NoNewPrivileges),
            s if s.starts_with("seccomp=") => {
                let (_, profile) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::Seccomp(String::from(profile)))
            }
            s if s.starts_with("proc-opts=") => {
                let (_, opts) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::ProcOpts(String::from(opts)))
            }
            s if s.starts_with("unmask=") => {
                let (_, unmask) = s.split_once('=').expect("delimiter is in guard");
                Ok(SecurityOpt::Unmask(String::from(unmask)))
            }
            _ => Err(ParseSecurityOptError::InvalidSecurityOpt(String::from(s))),
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

impl From<SecurityOpt> for Output {
    fn from(value: SecurityOpt) -> Self {
        (&value).into()
    }
}

impl From<&SecurityOpt> for Output {
    fn from(value: &SecurityOpt) -> Self {
        match value {
            SecurityOpt::Apparmor(policy) => Self::PodmanArg(format!("apparmor={policy}")),
            SecurityOpt::Label(label_opt) => Self::from(label_opt),
            SecurityOpt::Mask(mask) => Self::PodmanArg(format!("mask={mask}")),
            SecurityOpt::NoNewPrivileges => {
                Self::QuadletOptions(String::from("NoNewPrivileges=true"))
            }
            SecurityOpt::Seccomp(profile) => {
                Self::QuadletOptions(format!("SeccompProfile={profile}"))
            }
            SecurityOpt::ProcOpts(proc_opts) => Self::PodmanArg(format!("proc-opts={proc_opts}")),
            SecurityOpt::Unmask(unmask) => Self::PodmanArg(format!("unmask={unmask}")),
        }
    }
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
        match s {
            s if s.starts_with("user:") => {
                let (_, user) = s.split_once(':').expect("delimiter is in guard");
                Ok(LabelOpt::User(String::from(user)))
            }
            s if s.starts_with("role:") => {
                let (_, role) = s.split_once(':').expect("delimiter is in guard");
                Ok(LabelOpt::Role(String::from(role)))
            }
            s if s.starts_with("type:") => {
                let (_, label_type) = s.split_once(':').expect("delimiter is in guard");
                Ok(LabelOpt::Type(String::from(label_type)))
            }
            s if s.starts_with("level:") => {
                let (_, level) = s.split_once(':').expect("delimiter is in guard");
                Ok(LabelOpt::Level(String::from(level)))
            }
            s if s.starts_with("filetype:") => {
                let (_, filetype) = s.split_once(':').expect("delimiter is in guard");
                Ok(LabelOpt::Filetype(String::from(filetype)))
            }
            "disable" => Ok(LabelOpt::Disable),
            "nested" => Ok(LabelOpt::Nested),
            _ => Err(InvalidLabelOpt(String::from(s))),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("`{0}` is not a valid label option")]
pub struct InvalidLabelOpt(pub String);

impl From<LabelOpt> for Output {
    fn from(value: LabelOpt) -> Self {
        (&value).into()
    }
}

impl From<&LabelOpt> for Output {
    fn from(value: &LabelOpt) -> Self {
        match value {
            LabelOpt::User(user) => Self::PodmanArg(format!("label=user:{user}")),
            LabelOpt::Role(role) => Self::PodmanArg(format!("label=role:{role}")),
            LabelOpt::Type(label_type) => {
                Self::QuadletOptions(format!("SecurityLabelType={label_type}"))
            }
            LabelOpt::Level(level) => Self::QuadletOptions(format!("SecurityLabelLevel={level}")),
            LabelOpt::Filetype(filetype) => {
                Self::QuadletOptions(format!("SecurityLabelFileType={filetype}"))
            }
            LabelOpt::Disable => Self::QuadletOptions(String::from("SecurityLabelDisable=true")),
            LabelOpt::Nested => Self::PodmanArg(String::from("nested")),
        }
    }
}
