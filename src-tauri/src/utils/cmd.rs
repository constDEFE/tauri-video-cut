use std::path::Path;
use tokio::process::Command;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn new_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);

    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}
