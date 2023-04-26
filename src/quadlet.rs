mod container;
mod install;
mod kube;
mod network;
mod volume;

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

pub use self::{
    container::Container, install::Install, kube::Kube, network::Network, volume::Volume,
};
use crate::cli::{service::Service, unit::Unit};

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub unit: Option<Unit>,
    pub resource: Resource,
    pub service: Option<Service>,
    pub install: Option<Install>,
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(unit) = &self.unit {
            writeln!(f, "{unit}")?;
        }

        write!(f, "{}", self.resource)?;

        if let Some(service) = &self.service {
            write!(f, "\n{service}")?;
        }

        if let Some(install) = &self.install {
            write!(f, "\n{install}")?;
        }

        Ok(())
    }
}

impl From<Resource> for File {
    fn from(value: Resource) -> Self {
        Self {
            unit: None,
            resource: value,
            service: None,
            install: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource {
    Container(Box<Container>),
    Kube(Kube),
    Network(Network),
    Volume(Volume),
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Container(container) => write!(f, "{container}"),
            Self::Kube(kube) => write!(f, "{kube}"),
            Self::Network(network) => write!(f, "{network}"),
            Self::Volume(volume) => write!(f, "{volume}"),
        }
    }
}

impl From<Container> for Resource {
    fn from(value: Container) -> Self {
        Self::Container(Box::new(value))
    }
}

impl From<Box<Container>> for Resource {
    fn from(value: Box<Container>) -> Self {
        Self::Container(value)
    }
}

impl From<Kube> for Resource {
    fn from(value: Kube) -> Self {
        Self::Kube(value)
    }
}

impl From<Network> for Resource {
    fn from(value: Network) -> Self {
        Self::Network(value)
    }
}

impl From<Volume> for Resource {
    fn from(value: Volume) -> Self {
        Self::Volume(value)
    }
}

fn escape_spaces_join<'a>(words: impl IntoIterator<Item = &'a String>) -> String {
    words
        .into_iter()
        .map(|word| {
            if word.contains(' ') {
                format!("\"{word}\"").into()
            } else {
                word.into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join(" ")
}
