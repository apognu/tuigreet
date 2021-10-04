use std::process::Stdio;

use tokio::process::Command;

use crate::Greeter;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerOption {
  Shutdown,
  Reboot,
}

pub fn power(greeter: &mut Greeter, option: PowerOption) {
  let command = match greeter.power_commands.get(&option) {
    None => {
      let mut command = Command::new("shutdown");

      match option {
        PowerOption::Shutdown => command.arg("-h"),
        PowerOption::Reboot => command.arg("-r"),
      };

      command.arg("now");
      command
    }

    Some(args) => {
      let mut command = match greeter.power_setsid {
        true => {
          let mut command = Command::new("setsid");
          command.args(args.split(' '));
          command
        }

        false => {
          let mut args = args.split(' ');

          let mut command = Command::new(args.next().unwrap_or_default());
          command.args(args);
          command
        }
      };

      command.stdin(Stdio::null());
      command.stdout(Stdio::null());
      command.stderr(Stdio::null());

      command
    }
  };

  greeter.power_command = Some(command);
}
