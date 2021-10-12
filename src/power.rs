use std::{process::Stdio, sync::Arc};

use tokio::{process::Command, sync::RwLock};

use crate::{Greeter, Mode};

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

pub async fn run(greeter: &Arc<RwLock<Greeter>>, mut command: Command) {
  greeter.write().await.mode = Mode::Processing;

  let message = match tokio::spawn(async move { command.status().await }).await {
    Ok(result) => match result {
      Ok(status) if status.success() => None,
      Ok(status) => Some(format!("{} {}", fl!("command_exited"), status)),
      Err(err) => Some(format!("{}: {}", fl!("command_failed"), err)),
    },

    Err(_) => Some(fl!("command_failed")),
  };

  let mode = greeter.read().await.previous_mode;

  let mut greeter = greeter.write().await;

  greeter.mode = mode;
  greeter.message = message;
}
