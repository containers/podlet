#![allow(clippy::same_name_method)] // triggered by `proxy` macro

use nix::unistd::Uid;
use zbus::{blocking::Connection, proxy};

pub fn unit_files() -> zbus::Result<impl Iterator<Item = UnitFile>> {
    let connection = Connection::system()?;
    let manager = ManagerProxyBlocking::new(&connection)?;
    let mut unit_files = manager.list_unit_files()?;

    if !Uid::current().is_root() {
        let connection = Connection::session()?;
        let manager = ManagerProxyBlocking::new(&connection)?;
        unit_files.extend(manager.list_unit_files()?);
    }

    Ok(unit_files.into_iter().map(Into::into))
}

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    fn list_unit_files(&self) -> zbus::Result<Vec<(String, String)>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFile {
    pub file_name: String,
    pub status: String,
}

impl From<(String, String)> for UnitFile {
    fn from(value: (String, String)) -> Self {
        Self {
            file_name: value.0,
            status: value.1,
        }
    }
}
