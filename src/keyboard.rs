use std::error::Error;

use greetd_ipc::Request;
use termion::event::Key;

use crate::{
  event::{Event, Events},
  AuthStatus, Greeter, Mode,
};

pub fn handle(greeter: &mut Greeter, events: &Events) -> Result<(), Box<dyn Error>> {
  if let Event::Input(input) = events.next()? {
    match input {
      Key::Esc => {
        if let Mode::Command = greeter.mode {
          greeter.mode = greeter.previous_mode;
        } else {
          crate::exit(greeter, AuthStatus::Cancel)?;
        }
      }

      Key::Left => greeter.cursor_offset -= 1,
      Key::Right => greeter.cursor_offset += 1,

      Key::F(2) => {
        greeter.new_command = greeter.command.clone().unwrap_or_else(String::new);
        greeter.previous_mode = greeter.mode;
        greeter.mode = Mode::Command;
      }

      Key::Ctrl('a') => greeter.cursor_offset = -(greeter.username.len() as i16),
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
          greeter.mode = greeter.previous_mode;
        }
      },

      Key::Char(c) => insert_key(greeter, c),

      Key::Backspace | Key::Delete => delete_key(greeter, input),

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
  };

  let index = value.len() as i16 + greeter.cursor_offset;

  value.insert(index as usize, c);
}

fn delete_key(greeter: &mut Greeter, key: Key) {
  let value = match greeter.mode {
    Mode::Username => &mut greeter.username,
    Mode::Password => &mut greeter.answer,
    Mode::Command => &mut greeter.new_command,
  };

  let index = match key {
    Key::Backspace => value.len() as i16 + greeter.cursor_offset - 1,
    Key::Delete => value.len() as i16 + greeter.cursor_offset,
    _ => 0,
  };

  if value.chars().nth(index as usize).is_some() {
    value.remove(index as usize);

    if let Key::Delete = key {
      greeter.cursor_offset += 1;
    }
  }
}
