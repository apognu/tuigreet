use std::error::Error;

use greetd_ipc::Request;
use termion::event::Key;

use crate::{
  event::{Event, Events},
  info::delete_last_username,
  AuthStatus, Greeter, Mode,
};

pub fn handle(greeter: &mut Greeter, events: &Events) -> Result<(), Box<dyn Error>> {
  if let Event::Input(input) = events.next()? {
    match input {
      Key::Esc => match greeter.mode {
        Mode::Command | Mode::Sessions => greeter.mode = greeter.previous_mode,

        _ => {
          delete_last_username();

          crate::exit(greeter, AuthStatus::Cancel)?;
        }
      },

      Key::Left => greeter.cursor_offset -= 1,
      Key::Right => greeter.cursor_offset += 1,

      Key::F(2) => {
        greeter.previous_mode = match greeter.mode {
          Mode::Command | Mode::Sessions => greeter.previous_mode,
          _ => greeter.mode,
        };

        greeter.new_command = greeter.command.clone().unwrap_or_default();
        greeter.mode = Mode::Command;
      }

      Key::F(3) => {
        greeter.previous_mode = match greeter.mode {
          Mode::Command | Mode::Sessions => greeter.previous_mode,
          _ => greeter.mode,
        };

        greeter.mode = Mode::Sessions;
      }

      Key::Up => {
        if let Mode::Sessions = greeter.mode {
          if greeter.selected_session > 0 {
            greeter.selected_session -= 1;
          }
        }
      }

      Key::Down => {
        if let Mode::Sessions = greeter.mode {
          if greeter.selected_session < greeter.sessions.len() - 1 {
            greeter.selected_session += 1;
          }
        }
      }

      Key::Ctrl('a') => {
        let value = match greeter.mode {
          Mode::Username => &greeter.username,
          _ => &greeter.answer,
        };

        greeter.cursor_offset = -(value.chars().count() as i16);
      }

      Key::Ctrl('e') => greeter.cursor_offset = 0,

      Key::Char('\n') | Key::Char('\t') => match greeter.mode {
        Mode::Username => {
          greeter.working = true;
          greeter.message = None;
          greeter.request = Some(Request::CreateSession { username: greeter.username.clone() });
          greeter.answer = String::new();
        }

        Mode::Password => {
          greeter.working = true;
          greeter.message = None;

          greeter.request = Some(Request::PostAuthMessageResponse {
            response: Some(greeter.answer.clone()),
          });

          greeter.answer = String::new();
        }

        Mode::Command => {
          greeter.command = Some(greeter.new_command.clone());
          greeter.selected_session = greeter.sessions.iter().position(|(_, command)| Some(command) == greeter.command.as_ref()).unwrap_or(0);
          greeter.mode = greeter.previous_mode;
        }

        Mode::Sessions => {
          if let Some((_, command)) = greeter.sessions.get(greeter.selected_session) {
            greeter.command = Some(command.clone());
          }

          greeter.mode = greeter.previous_mode;
        }
      },

      Key::Char(c) => insert_key(greeter, c),

      Key::Backspace | Key::Delete => delete_key(greeter, input),

      Key::Ctrl('u') => match greeter.mode {
        Mode::Username => greeter.username = String::new(),
        Mode::Password => greeter.answer = String::new(),
        Mode::Command => greeter.new_command = String::new(),
        _ => {}
      },

      _ => {}
    }
  }

  Ok(())
}

fn insert_key(greeter: &mut Greeter, c: char) {
  let value = match greeter.mode {
    Mode::Username => &mut greeter.username,
    Mode::Password => &mut greeter.answer,
    Mode::Command => &mut greeter.new_command,
    Mode::Sessions => return,
  };

  let index = (value.chars().count() as i16 + greeter.cursor_offset) as usize;
  let left = value.chars().take(index);
  let right = value.chars().skip(index);

  *value = left.chain(vec![c].into_iter()).chain(right).collect();
}

fn delete_key(greeter: &mut Greeter, key: Key) {
  let value = match greeter.mode {
    Mode::Username => &mut greeter.username,
    Mode::Password => &mut greeter.answer,
    Mode::Command => &mut greeter.new_command,
    Mode::Sessions => return,
  };

  let index = match key {
    Key::Backspace => (value.chars().count() as i16 + greeter.cursor_offset - 1) as usize,
    Key::Delete => (value.chars().count() as i16 + greeter.cursor_offset) as usize,
    _ => 0,
  };

  if value.chars().nth(index as usize).is_some() {
    let left = value.chars().take(index);
    let right = value.chars().skip(index + 1);

    *value = left.chain(right).collect();

    if let Key::Delete = key {
      greeter.cursor_offset += 1;
    }
  }
}
