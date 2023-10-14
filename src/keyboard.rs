use std::{error::Error, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use greetd_ipc::Request;
use tokio::sync::RwLock;

use crate::{
  event::{Event, Events},
  info::{get_last_user_session, write_last_session},
  ipc::Ipc,
  power::power,
  ui::POWER_OPTIONS,
  Greeter, Mode,
};

pub async fn handle(greeter: Arc<RwLock<Greeter>>, events: &mut Events, ipc: Ipc) -> Result<(), Box<dyn Error>> {
  if let Some(Event::Key(input)) = events.next().await {
    let mut greeter = greeter.write().await;

    match input {
      KeyEvent {
        code: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => match greeter.mode {
        Mode::Username => greeter.username = String::new(),
        Mode::Password => greeter.answer = String::new(),
        Mode::Command => greeter.new_command = String::new(),
        _ => {}
      },

      #[cfg(debug_assertions)]
      KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => {
        use crate::greeter::AuthStatus;

        crate::exit(&mut greeter, AuthStatus::Cancel).await;
      }

      KeyEvent { code: KeyCode::Esc, .. } => {
        Ipc::cancel(&mut greeter).await;
        greeter.reset().await;
      }

      KeyEvent { code: KeyCode::Left, .. } => greeter.cursor_offset -= 1,
      KeyEvent { code: KeyCode::Right, .. } => greeter.cursor_offset += 1,

      KeyEvent { code: KeyCode::F(2), .. } => {
        greeter.previous_mode = match greeter.mode {
          Mode::Users | Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
          _ => greeter.mode,
        };

        greeter.new_command = greeter.command.clone().unwrap_or_default();
        greeter.mode = Mode::Command;
      }

      KeyEvent { code: KeyCode::F(3), .. } => {
        greeter.previous_mode = match greeter.mode {
          Mode::Users | Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
          _ => greeter.mode,
        };

        greeter.mode = Mode::Sessions;
      }

      KeyEvent { code: KeyCode::F(12), .. } => {
        greeter.previous_mode = match greeter.mode {
          Mode::Users | Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
          _ => greeter.mode,
        };

        greeter.mode = Mode::Power;
      }

      KeyEvent { code: KeyCode::Up, .. } => {
        if let Mode::Users = greeter.mode {
          if greeter.selected_user > 0 {
            greeter.selected_user -= 1;
          }
        }

        if let Mode::Sessions = greeter.mode {
          if greeter.selected_session > 0 {
            greeter.selected_session -= 1;
          }
        }

        if let Mode::Power = greeter.mode {
          if greeter.selected_power_option > 0 {
            greeter.selected_power_option -= 1;
          }
        }
      }

      KeyEvent { code: KeyCode::Down, .. } => {
        if let Mode::Users = greeter.mode {
          if greeter.selected_user < greeter.users.len() - 1 {
            greeter.selected_user += 1;
          }
        }

        if let Mode::Sessions = greeter.mode {
          if greeter.selected_session < greeter.sessions.len() - 1 {
            greeter.selected_session += 1;
          }
        }

        if let Mode::Power = greeter.mode {
          if greeter.selected_power_option < POWER_OPTIONS.len() - 1 {
            greeter.selected_power_option += 1;
          }
        }
      }

      KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => {
        let value = {
          match greeter.mode {
            Mode::Username => &greeter.username,
            _ => &greeter.answer,
          }
        };

        greeter.cursor_offset = -(value.chars().count() as i16);
      }

      KeyEvent {
        code: KeyCode::Char('e'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => greeter.cursor_offset = 0,

      KeyEvent { code: KeyCode::Tab, .. } => match greeter.mode {
        Mode::Username if !greeter.username.is_empty() => validate_username(&mut greeter, &ipc).await,
        _ => {}
      },

      KeyEvent { code: KeyCode::Enter, .. } => match greeter.mode {
        Mode::Username if !greeter.username.is_empty() => validate_username(&mut greeter, &ipc).await,

        Mode::Username if greeter.user_menu => {
          greeter.previous_mode = match greeter.mode {
            Mode::Users | Mode::Command | Mode::Sessions | Mode::Power => greeter.previous_mode,
            _ => greeter.mode,
          };

          greeter.mode = Mode::Users;
        }

        Mode::Username => {}

        Mode::Password => {
          greeter.working = true;
          greeter.message = None;

          ipc
            .send(Request::PostAuthMessageResponse {
              response: Some(greeter.answer.clone()),
            })
            .await;

          greeter.answer = String::new();
        }

        Mode::Command => {
          let cmd = &greeter.command;

          greeter.selected_session = greeter.sessions.iter().position(|(_, command)| Some(command) == cmd.as_ref()).unwrap_or(0);
          greeter.command = Some(greeter.new_command.clone());

          if greeter.remember_session {
            write_last_session(&greeter.new_command);
          }

          greeter.mode = greeter.previous_mode;
        }

        Mode::Users => {
          let username = greeter.users.get(greeter.selected_user).cloned();

          if let Some((username, name)) = username {
            greeter.username = username;
            greeter.username_mask = name;
          }

          validate_username(&mut greeter, &ipc).await;
        }

        Mode::Sessions => {
          let session = match greeter.sessions.get(greeter.selected_session) {
            Some((_, command)) => Some(command.clone()),
            _ => None,
          };

          if let Some(command) = session {
            if greeter.remember_session {
              write_last_session(&command);
            }

            greeter.command = Some(command);
          }

          greeter.mode = greeter.previous_mode;
        }

        Mode::Power => {
          if let Some((option, _)) = POWER_OPTIONS.get(greeter.selected_power_option) {
            power(&mut greeter, *option);
          }

          greeter.mode = greeter.previous_mode;
        }

        Mode::Processing => {}
      },

      KeyEvent { code: KeyCode::Char(c), .. } => insert_key(&mut greeter, c).await,

      KeyEvent { code: KeyCode::Backspace, .. } | KeyEvent { code: KeyCode::Delete, .. } => delete_key(&mut greeter, input.code).await,

      _ => {}
    }
  }

  Ok(())
}

async fn insert_key(greeter: &mut Greeter, c: char) {
  let value = match greeter.mode {
    Mode::Username => &greeter.username,
    Mode::Password => &greeter.answer,
    Mode::Command => &greeter.new_command,
    Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => return,
  };

  let index = (value.chars().count() as i16 + greeter.cursor_offset) as usize;
  let left = value.chars().take(index);
  let right = value.chars().skip(index);

  let value = left.chain(vec![c].into_iter()).chain(right).collect();
  let mode = greeter.mode;

  match mode {
    Mode::Username => greeter.username = value,
    Mode::Password => greeter.answer = value,
    Mode::Command => greeter.new_command = value,
    _ => {}
  };
}

async fn delete_key(greeter: &mut Greeter, key: KeyCode) {
  let value = match greeter.mode {
    Mode::Username => &greeter.username,
    Mode::Password => &greeter.answer,
    Mode::Command => &greeter.new_command,
    Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => return,
  };

  let index = match key {
    KeyCode::Backspace => (value.chars().count() as i16 + greeter.cursor_offset - 1) as usize,
    KeyCode::Delete => (value.chars().count() as i16 + greeter.cursor_offset) as usize,
    _ => 0,
  };

  if value.chars().nth(index).is_some() {
    let left = value.chars().take(index);
    let right = value.chars().skip(index + 1);

    let value = left.chain(right).collect();

    match greeter.mode {
      Mode::Username => greeter.username = value,
      Mode::Password => greeter.answer = value,
      Mode::Command => greeter.new_command = value,
      Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => return,
    };

    if let KeyCode::Delete = key {
      greeter.cursor_offset += 1;
    }
  }
}

async fn validate_username(greeter: &mut Greeter, ipc: &Ipc) {
  greeter.working = true;
  greeter.message = None;

  ipc.send(Request::CreateSession { username: greeter.username.clone() }).await;
  greeter.answer = String::new();

  if greeter.remember_user_session {
    if let Ok(command) = get_last_user_session(&greeter.username) {
      greeter.selected_session = greeter.sessions.iter().position(|(_, cmd)| Some(cmd) == Some(&command)).unwrap_or(0);
      greeter.command = Some(command);
    }
  }
}
