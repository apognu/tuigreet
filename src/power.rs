use std::{
  io,
  process::{Command, ExitStatus, Stdio},
};

use crate::Greeter;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerOption {
  Shutdown,
  Reboot,
}

pub fn power(greeter: &Greeter, option: PowerOption) -> Result<ExitStatus, io::Error> {
  let mut command = match greeter.power_commands.get(&option) {
    None => {
      let mut command = Command::new("shutdown");

      match option {
        PowerOption::Shutdown => command.arg("-h"),
        PowerOption::Reboot => command.arg("-r"),
      };

      command.arg("now");
      command
    }

    Some(command) => {
      let mut args: Vec<&str> = command.split(' ').collect();
      let exe = args.remove(0);

      let mut command = Command::new(exe);
      command.args(args);
      command
    }
  };

  command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).env_clear().status()
}
