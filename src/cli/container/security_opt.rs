use std::str::FromStr;

use thiserror::Error;

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
                Ok(Self::Apparmor(String::from(policy)))
            }
            s if s.starts_with("label=") => {
                let (_, label) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Label(label.parse()?))
            }
            s if s.starts_with("mask=") => {
                let (_, mask) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Mask(String::from(mask)))
            }
            "no-new-privileges" => Ok(Self::NoNewPrivileges),
            s if s.starts_with("seccomp=") => {
                let (_, profile) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Seccomp(String::from(profile)))
            }
            s if s.starts_with("proc-opts=") => {
                let (_, opts) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::ProcOpts(String::from(opts)))
            }
            s if s.starts_with("unmask=") => {
                let (_, unmask) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Unmask(String::from(unmask)))
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
                Ok(Self::User(String::from(user)))
            }
            s if s.starts_with("role:") => {
                let (_, role) = s.split_once(':').expect("delimiter is in guard");
                Ok(Self::Role(String::from(role)))
            }
            s if s.starts_with("type:") => {
                let (_, label_type) = s.split_once(':').expect("delimiter is in guard");
                Ok(Self::Type(String::from(label_type)))
            }
            s if s.starts_with("level:") => {
                let (_, level) = s.split_once(':').expect("delimiter is in guard");
                Ok(Self::Level(String::from(level)))
            }
            s if s.starts_with("filetype:") => {
                let (_, filetype) = s.split_once(':').expect("delimiter is in guard");
                Ok(Self::Filetype(String::from(filetype)))
            }
            "disable" => Ok(Self::Disable),
            "nested" => Ok(Self::Nested),
            _ => Err(InvalidLabelOpt(String::from(s))),
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
    pub seccomp_profile: Option<String>,
    pub security_label_disable: bool,
    pub security_label_file_type: Option<String>,
    pub security_label_level: Option<String>,
    pub security_label_nested: bool,
    pub security_label_type: Option<String>,
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
            SecurityOpt::Unmask(unmask) => self.podman_args.push(format!("unmask={unmask}")),
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
