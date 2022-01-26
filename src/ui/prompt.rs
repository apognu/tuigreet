use std::error::Error;

use tui::{
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  text::{Span, Text},
  widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
  info::get_hostname,
  ui::{prompt_value, util::*, Frame},
  Greeter, Mode,
};

const GREETING_INDEX: usize = 0;
const USERNAME_INDEX: usize = 1;
const ANSWER_INDEX: usize = 2;

pub fn draw(greeter: &mut Greeter, f: &mut Frame) -> Result<(u16, u16), Box<dyn Error>> {
  let size = f.size();

  let width = greeter.width();
  let height = get_height(greeter);
  let container_padding = greeter.container_padding();
  let prompt_padding = greeter.prompt_padding();
  let x = (size.width - width) / 2;
  let y = (size.height - height) / 2;

  let container = Rect::new(x, y, width, height);
  let frame = Rect::new(x + container_padding, y + container_padding, width - (2 * container_padding), height - (2 * container_padding));

  let hostname = Span::from(titleize(&fl!("title_authenticate", hostname = get_hostname())));
  let block = Block::default().title(hostname).borders(Borders::ALL).border_type(BorderType::Plain);

  f.render_widget(block, container);

  let (message, message_height) = get_message_height(greeter, container_padding, 1);
  let (greeting, greeting_height) = get_greeting_height(greeter, container_padding, 0);

  let username_padding = if greeter.mode == Mode::Username && prompt_padding == 0 { 1 } else { prompt_padding };
  let answer_padding = if prompt_padding == 0 { 1 } else { prompt_padding };

  let constraints = [
    Constraint::Length(greeting_height),                                                     // Greeting
    Constraint::Length(1 + username_padding),                                                // Username
    Constraint::Length(if greeter.mode == Mode::Username { 0 } else { 1 + answer_padding }), // Answer
  ];

  let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints.as_ref()).split(frame);
  let cursor = chunks[USERNAME_INDEX];

  if let Some(greeting) = &greeting {
    let greeting_text = greeting.trim_end();
    let greeting_label = Paragraph::new(greeting_text).alignment(Alignment::Center);

    f.render_widget(greeting_label, chunks[GREETING_INDEX]);
  }

  let username_label = if greeter.user_menu && greeter.username.is_empty() {
    let prompt_text = Span::from(fl!("select_user"));

    Paragraph::new(prompt_text).alignment(Alignment::Center)
  } else {
    let username_text = prompt_value(Some(fl!("username")));

    Paragraph::new(username_text)
  };

  let username_value_text = Span::from(greeter.username.as_str());
  let username_value = Paragraph::new(username_value_text);

  match greeter.mode {
    Mode::Username | Mode::Password => {
      f.render_widget(username_label, chunks[USERNAME_INDEX]);

      if !greeter.user_menu || !greeter.username.is_empty() {
        f.render_widget(
          username_value,
          Rect::new(
            1 + chunks[USERNAME_INDEX].x + fl!("username").len() as u16,
            chunks[USERNAME_INDEX].y,
            get_input_width(greeter, &Some(fl!("username"))),
            1,
          ),
        );
      }

      let answer_text = if greeter.working { Span::from(fl!("wait")) } else { prompt_value(greeter.prompt.as_ref()) };

      let answer_label = Paragraph::new(answer_text);

      if greeter.mode == Mode::Password || greeter.previous_mode == Mode::Password {
        f.render_widget(answer_label, chunks[ANSWER_INDEX]);

        if !greeter.secret || greeter.asterisks {
          let value = if greeter.secret && greeter.asterisks {
            greeter.asterisks_char.to_string().repeat(greeter.answer.len())
          } else {
            greeter.answer.clone()
          };

          let answer_value_text = Span::from(value);
          let answer_value = Paragraph::new(answer_value_text);

          f.render_widget(
            answer_value,
            Rect::new(
              chunks[ANSWER_INDEX].x + greeter.prompt_width() as u16,
              chunks[ANSWER_INDEX].y,
              get_input_width(greeter, &greeter.prompt),
              1,
            ),
          );
        }
      }

      if let Some(message) = message {
        let message_text = Text::from(message);
        let message = Paragraph::new(message_text).alignment(Alignment::Center);

        f.render_widget(message, Rect::new(x, y + height, width, message_height));
      }
    }

    _ => {}
  }

  match greeter.mode {
    Mode::Username => {
      let username_length = greeter.username.chars().count();
      let offset = get_cursor_offset(greeter, username_length);

      Ok((2 + cursor.x + fl!("username").len() as u16 + offset as u16, USERNAME_INDEX as u16 + cursor.y))
    }

    Mode::Password => {
      let answer_length = greeter.answer.chars().count();
      let offset = get_cursor_offset(greeter, answer_length);

      if greeter.secret && !greeter.asterisks {
        Ok((1 + cursor.x + greeter.prompt_width() as u16, ANSWER_INDEX as u16 + prompt_padding + cursor.y))
      } else {
        Ok((1 + cursor.x + greeter.prompt_width() as u16 + offset as u16, ANSWER_INDEX as u16 + prompt_padding + cursor.y))
      }
    }

    _ => Ok((1, 1)),
  }
}
