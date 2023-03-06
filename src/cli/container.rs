use std::fmt::Display;

use clap::Args;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct QuadletOptions {
    /// Add Linux capabilities
    ///
    /// Converts to "AddCapability=CAPABILITY"
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "CAPABILITY")]
    cap_add: Vec<String>,

    /// Mount a volume in the container
    ///
    /// Converts to "Volume=VOLUME"
    ///
    /// Can be specified multiple times
    #[arg(
        short,
        long,
        value_name = "[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]"
    )]
    volume: Vec<String>,
}

impl Display for QuadletOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.cap_add.is_empty() {
            writeln!(f, "AddCapability={}", self.cap_add.join(" "))?;
        }

        for volume in &self.volume {
            writeln!(f, "Volume={volume}")?;
        }

        Ok(())
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct PodmanArgs {
    /// Add a custom host-to-IP mapping
    ///
    /// Can be specified multiple times
    #[arg(long, value_name = "HOST:IP")]
    add_host: Vec<String>,
}

impl PodmanArgs {
    /// Whether all fields are empty
    fn is_empty(&self) -> bool {
        self.add_host.is_empty()
    }
}

impl Display for PodmanArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            Ok(())
        } else {
            let args_iter = self.add_host.iter().flat_map(|host| ["--add-host", host]);

            writeln!(f, "PodmanArgs={}", shlex::join(args_iter))
        }
    }
}
