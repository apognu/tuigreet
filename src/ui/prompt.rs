use std::error::Error;

use rand::{prelude::StdRng, Rng, SeedableRng};
use tui::{
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  text::Span,
  widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
  info::get_hostname,
  ui::{prompt_value, util::*, Frame},
  GreetAlign, Greeter, Mode, SecretDisplay,
};

use super::common::style::Themed;

const GREETING_INDEX: usize = 0;
const USERNAME_INDEX: usize = 1;
const ANSWER_INDEX: usize = 3;

pub fn draw(greeter: &mut Greeter, f: &mut Frame) -> Result<(u16, u16), Box<dyn Error>> {
  let theme = &greeter.theme;

  let size = f.size();
  let (x, y, width, height) = get_rect_bounds(greeter, size, 0);

  let container_padding = greeter.container_padding();
  let prompt_padding = greeter.prompt_padding();
  let greeting_alignment = match greeter.greet_align() {
    GreetAlign::Center => Alignment::Center,
    GreetAlign::Left => Alignment::Left,
    GreetAlign::Right => Alignment::Right,
  };

  let container = Rect::new(x, y, width, height);
  let frame = Rect::new(x + container_padding, y + container_padding, width - (2 * container_padding), height - (2 * container_padding));

  let hostname = Span::from(titleize(&fl!("title_authenticate", hostname = get_hostname())));
  let block = Block::default()
    .title(hostname)
    .title_style(theme.of(&[Themed::Title]))
    .style(theme.of(&[Themed::Container]))
    .borders(Borders::ALL)
    .border_type(BorderType::Plain)
    .border_style(theme.of(&[Themed::Border]));

  f.render_widget(block, container);

  let (message, message_height) = get_message_height(greeter, container_padding, 1);
  let (greeting, greeting_height) = get_greeting_height(greeter, container_padding, 0);

  let should_display_answer = greeter.mode == Mode::Password;

  let constraints = [
    Constraint::Length(greeting_height),                                        // Greeting
    Constraint::Length(1),                                                      // Username
    Constraint::Length(if should_display_answer { prompt_padding } else { 0 }), // Prompt padding
    Constraint::Length(if should_display_answer { 1 } else { 0 }),              // Answer
  ];

  let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints.as_ref()).split(frame);
  let cursor = chunks[USERNAME_INDEX];

  if let Some(greeting) = greeting {
    let greeting_label = greeting.alignment(greeting_alignment).style(theme.of(&[Themed::Greet]));

    f.render_widget(greeting_label, chunks[GREETING_INDEX]);
  }

  let username_label = if greeter.user_menu && greeter.username.value.is_empty() {
    let prompt_text = Span::from(fl!("select_user"));

    Paragraph::new(prompt_text).alignment(Alignment::Center)
  } else {
    let username_text = prompt_value(theme, Some(fl!("username")));

    Paragraph::new(username_text)
  };

  let username = greeter.username.get();
  let username_value_text = Span::from(username);
  let username_value = Paragraph::new(username_value_text).style(theme.of(&[Themed::Input]));

  match greeter.mode {
    Mode::Username | Mode::Password | Mode::Action => {
      f.render_widget(username_label, chunks[USERNAME_INDEX]);

      if !greeter.user_menu || !greeter.username.value.is_empty() {
        f.render_widget(
          username_value,
          Rect::new(
            1 + chunks[USERNAME_INDEX].x + fl!("username").chars().count() as u16,
            chunks[USERNAME_INDEX].y,
            get_input_width(greeter, width, &Some(fl!("username"))),
            1,
          ),
        );
      }

      let answer_text = if greeter.working { Span::from(fl!("wait")) } else { prompt_value(theme, greeter.prompt.as_ref()) };

      let answer_label = Paragraph::new(answer_text);

      if greeter.mode == Mode::Password || greeter.previous_mode == Mode::Password {
        f.render_widget(answer_label, chunks[ANSWER_INDEX]);

        if !greeter.asking_for_secret || greeter.secret_display.show() {
          let value = match (greeter.asking_for_secret, &greeter.secret_display) {
            (true, SecretDisplay::Character(pool)) => {
              if pool.chars().count() == 1 {
                pool.repeat(greeter.buffer.chars().count())
              } else {
                let mut rng = StdRng::seed_from_u64(0);

                greeter.buffer.chars().map(|_| pool.chars().nth(rng.gen_range(0..pool.chars().count())).unwrap()).collect()
              }
            }

            _ => greeter.buffer.clone(),
          };

          let answer_value_text = Span::from(value);
          let answer_value = Paragraph::new(answer_value_text).style(theme.of(&[Themed::Input]));

          f.render_widget(
            answer_value,
            Rect::new(
              chunks[ANSWER_INDEX].x + greeter.prompt_width() as u16,
              chunks[ANSWER_INDEX].y,
              get_input_width(greeter, width, &greeter.prompt),
              1,
            ),
          );
        }
      }

      if let Some(message) = message {
        let message = message.alignment(Alignment::Center);

        f.render_widget(message, Rect::new(x, y + height, width, message_height));
      }
    }

    _ => {}
  }

  match greeter.mode {
    Mode::Username => {
      let username_length = greeter.username.get().chars().count();
      let offset = get_cursor_offset(greeter, username_length);

      Ok((2 + cursor.x + fl!("username").chars().count() as u16 + offset as u16, USERNAME_INDEX as u16 + cursor.y))
    }

    Mode::Password => {
      let answer_length = greeter.buffer.chars().count();
      let offset = get_cursor_offset(greeter, answer_length);

      if greeter.asking_for_secret && !greeter.secret_display.show() {
        Ok((1 + cursor.x + greeter.prompt_width() as u16, ANSWER_INDEX as u16 + prompt_padding + cursor.y - 1))
      } else {
        Ok((1 + cursor.x + greeter.prompt_width() as u16 + offset as u16, ANSWER_INDEX as u16 + prompt_padding + cursor.y - 1))
      }
    }

    _ => Ok((1, 1)),
  }
}
