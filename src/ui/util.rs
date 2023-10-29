use tui::prelude::Rect;

use crate::{Greeter, Mode};

pub fn titleize(message: &str) -> String {
  format!(" {message} ")
}

// Determinew whether the cursor should be shown or hidden from the current
// mode and configuration. Usually, we will show the cursor only when expecting
// text entries from the user.
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

// Computes the height of the main window where we display content, depending on
// the mode and spacing configuration.
//
// +------------------------+
// |                        | <- container padding
// |        Greeting        | <- greeting height
// |                        | <- auto-padding if greeting
// | Username:              | <- username
// | Password:              | <- password if prompt == Some(_)
// |                        | <- container padding
// +------------------------+
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
    Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => 2 * container_padding,
  };

  match greeter.mode {
    Mode::Command | Mode::Sessions | Mode::Power | Mode::Processing => initial,
    _ => initial + greeting_height,
  }
}

// Get the coordinates and size of the main window area, from the terminal size,
// and the content we need to display.
pub fn get_rect_bounds(greeter: &Greeter, area: Rect, items: usize) -> (u16, u16, u16, u16) {
  let width = greeter.width();
  let height: u16 = get_height(greeter) + items as u16;

  let x = if width < area.width { (area.width - width) / 2 } else { 0 };
  let y = if height < area.height { (area.height - height) / 2 } else { 0 };

  let (x, width) = if (x + width) >= area.width { (0, area.width) } else { (x, width) };
  let (y, height) = if (y + height) >= area.height { (0, area.height) } else { (y, height) };

  (x, y, width, height)
}

// Computes the size of a text entry, from the container width and, if
// applicable, the prompt length.
pub fn get_input_width(greeter: &Greeter, width: u16, label: &Option<String>) -> u16 {
  let width = std::cmp::min(greeter.width(), width);

  let label_width = match label {
    None => 0,
    Some(label) => label.chars().count(),
  };

  width - label_width as u16 - 4 - 1
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

#[cfg(test)]
mod test {
  use tui::prelude::Rect;

  use crate::{
    ui::util::{get_greeting_height, get_height},
    Greeter, Mode,
  };

  use super::{get_input_width, get_rect_bounds};

  // +-----------+
  // | Username: |
  // +-----------+
  #[test]
  fn test_container_height_username_padding_zero() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--container-padding", "0"]).ok();
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 3);
  }

  // +-----------+
  // |           |
  // | Username: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_username_padding_one() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--container-padding", "1"]).ok();
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 5);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_username_greeting_padding_one() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--container-padding", "1"]).ok();
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Username;

    assert_eq!(get_height(&greeter), 7);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // |           |
  // | Password: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_password_greeting_padding_one_prompt_padding_1() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--container-padding", "1"]).ok();
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Password;
    greeter.prompt = Some("Password:".into());

    assert_eq!(get_height(&greeter), 9);
  }

  // +-----------+
  // |           |
  // | Greeting  |
  // |           |
  // | Username: |
  // | Password: |
  // |           |
  // +-----------+
  #[test]
  fn test_container_height_password_greeting_padding_one_prompt_padding_0() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--container-padding", "1", "--prompt-padding", "0"]).ok();
    greeter.greeting = Some("Hello".into());
    greeter.mode = Mode::Password;
    greeter.prompt = Some("Password:".into());

    assert_eq!(get_height(&greeter), 8);
  }

  #[test]
  fn test_rect_bounds() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--width", "50"]).ok();

    let (x, y, width, height) = get_rect_bounds(&greeter, Rect::new(0, 0, 100, 100), 1);

    assert_eq!(x, 25);
    assert_eq!(y, 47);
    assert_eq!(width, 50);
    assert_eq!(height, 6);
  }

  // | Username: __________________________ |
  // <--------------------------------------> width 40 (padding 1)
  //   <-------> prompt width 9
  //             <------------------------> input width 26
  #[test]
  fn input_width() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--width", "40", "--container-padding", "1"]).ok();

    let input_width = get_input_width(&greeter, 40, &Some("Username:".into()));

    assert_eq!(input_width, 26);
  }

  #[test]
  fn greeting_height_one_line() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--width", "10", "--container-padding", "1"]).ok();
    greeter.greeting = Some("Hello".into());

    let (text, width) = get_greeting_height(&greeter, 1, 0);

    assert!(matches!(text.as_deref(), Some("Hello")));
    assert_eq!(width, 2);
  }

  #[test]
  fn greeting_height_two_lines() {
    let mut greeter = Greeter::default();
    greeter.config = Greeter::options().parse(&["--width", "10", "--container-padding", "1"]).ok();
    greeter.greeting = Some("Hello World".into());

    let (text, width) = get_greeting_height(&greeter, 1, 0);

    assert!(matches!(text.as_deref(), Some("Hello\nWorld")));
    assert_eq!(width, 3);
  }
}
