use std::fmt::Display;

use clap::Args;

mod podman;
mod quadlet;

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Container {
    #[command(flatten)]
    quadlet_options: quadlet::QuadletOptions,

    /// Converts to "PodmanArgs=ARGS"
    #[command(flatten)]
    podman_args: podman::PodmanArgs,

    /// The image to run in the container
    ///
    /// Converts to "Image=IMAGE"
    image: String,

    /// Optionally, the command to run in the container
    ///
    /// Converts to "Exec=COMMAND..."
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

impl Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Container]")?;
        writeln!(f, "Image={}", self.image)?;
        write!(f, "{}", self.quadlet_options)?;
        write!(f, "{}", self.podman_args)?;
        if !self.command.is_empty() {
            let command = shlex::join(self.command.iter().map(String::as_str));
            writeln!(f, "Exec={command}")?;
        }
        Ok(())
    }
}
