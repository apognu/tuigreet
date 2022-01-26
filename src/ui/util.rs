use crate::{Greeter, Mode};

pub fn titleize(message: &str) -> String {
  format!(" {} ", message)
}

pub fn should_hide_cursor(greeter: &Greeter) -> bool {
  greeter.working
    || greeter.done
    || (greeter.user_menu && greeter.mode == Mode::Username && greeter.username.is_empty())
    || (greeter.mode == Mode::Password && greeter.prompt.is_none())
    || greeter.mode == Mode::Users
    || greeter.mode == Mode::Sessions
    || greeter.mode == Mode::Power
    || greeter.mode == Mode::Processing
}

pub fn get_height(greeter: &Greeter) -> u16 {
  let (_, greeting_height) = get_greeting_height(greeter, 1, 0);
  let container_padding = greeter.container_padding();
  let prompt_padding = greeter.prompt_padding();

  let initial = match greeter.mode {
    Mode::Username | Mode::Command => (2 * container_padding) + 1,
    Mode::Password => match greeter.prompt {
      Some(_) => (2 * container_padding) + prompt_padding + 2,
      None => (2 * container_padding) + 1,
    },
    Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => (2 * container_padding),
  };

  match greeter.mode {
    Mode::Command | Mode::Sessions | Mode::Power | Mode::Processing => initial,
    _ => initial + greeting_height,
  }
}

pub fn get_input_width(greeter: &Greeter, label: &Option<String>) -> u16 {
  let label_width = match label {
    None => 0,
    Some(label) => label.chars().count(),
  };

  greeter.width() - label_width as u16 - 4 - 1
}

pub fn get_cursor_offset(greeter: &mut Greeter, length: usize) -> i16 {
  let mut offset = length as i16 + greeter.cursor_offset;

  if offset < 0 {
    offset = 0;
    greeter.cursor_offset = -(length as i16);
  }

  if offset > length as i16 {
    offset = length as i16;
    greeter.cursor_offset = 0;
  }

  offset
}

pub fn get_greeting_height(greeter: &Greeter, padding: u16, fallback: u16) -> (Option<String>, u16) {
  if let Some(greeting) = &greeter.greeting {
    let width = greeter.width();
    let wrapped = textwrap::fill(greeting, (width - (2 * padding)) as usize);
    let height = wrapped.trim_end().matches('\n').count();

    (Some(wrapped), height as u16 + 2)
  } else {
    (None, fallback)
  }
}

pub fn get_message_height(greeter: &Greeter, padding: u16, fallback: u16) -> (Option<String>, u16) {
  if let Some(message) = &greeter.message {
    let width = greeter.width();
    let wrapped = textwrap::fill(message.trim_end(), width as usize - 4);
    let height = wrapped.trim_end().matches('\n').count();

    (Some(wrapped), height as u16 + padding)
  } else {
    (None, fallback)
  }
}
