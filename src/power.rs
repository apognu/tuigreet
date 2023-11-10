use std::{process::Stdio, sync::Arc};

use tokio::{process::Command, sync::RwLock};

use crate::{event::Event, ui::power::Power, Greeter, Mode};

#[derive(SmartDefault, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerOption {
  #[default]
  Shutdown,
  Reboot,
}

pub async fn power(greeter: &mut Greeter, option: PowerOption) {
  let command = match greeter.powers.options.iter().find(|opt| opt.action == option) {
    None => None,

    Some(Power { command: Some(args), .. }) => {
      let command = match greeter.power_setsid {
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

      Some(command)
    }

    Some(_) => {
      let mut command = Command::new("shutdown");

      match option {
        PowerOption::Shutdown => command.arg("-h"),
        PowerOption::Reboot => command.arg("-r"),
      };

      command.arg("now");

      Some(command)
    }
  };

  if let Some(mut command) = command {
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    if let Some(ref sender) = greeter.events {
      let _ = sender.send(Event::PowerCommand(command)).await;
    }
  }
}

pub async fn run(greeter: &Arc<RwLock<Greeter>>, mut command: Command) {
  greeter.write().await.mode = Mode::Processing;

  let message = match command.output().await {
    Ok(result) => match (result.status, result.stderr) {
      (status, _) if status.success() => None,
      (status, output) => {
        let status = format!("{} {status}", fl!("command_exited"));
        let output = String::from_utf8(output).unwrap_or_default();

        Some(format!("{status}\n{output}"))
      }
    },

    Err(err) => Some(format!("{}: {err}", fl!("command_failed"))),
  };

  let mode = greeter.read().await.previous_mode;

  let mut greeter = greeter.write().await;

  greeter.mode = mode;
  greeter.message = message;
}
