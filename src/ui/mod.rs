mod command;
mod i18n;
mod power;
mod prompt;
mod sessions;
mod util;

use std::{
  error::Error,
  io::{self, Write},
  sync::Arc,
};

use chrono::prelude::*;
use termion::raw::RawTerminal;
use tokio::sync::RwLock;
use tui::{
  backend::TermionBackend,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Modifier, Style},
  text::{Span, Spans},
  widgets::Paragraph,
  Terminal,
};

use crate::{info::capslock_status, ui::util::titleize, Greeter, Mode};

pub use self::{
  i18n::MESSAGES,
  power::{Option as PowerOption, OPTIONS as POWER_OPTIONS},
};

const TITLEBAR_INDEX: usize = 1;
const STATUSBAR_INDEX: usize = 3;
const STATUSBAR_LEFT_INDEX: usize = 1;
const STATUSBAR_RIGHT_INDEX: usize = 2;

type Term = Terminal<TermionBackend<RawTerminal<io::Stdout>>>;

pub async fn draw(greeter: Arc<RwLock<Greeter>>, terminal: &mut Term) -> Result<(), Box<dyn Error>> {
  let mut greeter = greeter.write().await;

  let hide_cursor = if greeter.working || greeter.mode == Mode::Sessions {
    terminal.hide_cursor()?;
    true
  } else {
    false
  };

  terminal.draw(|mut f| {
    let size = f.size();
    let chunks = Layout::default()
      .constraints(
        [
          Constraint::Length(greeter.window_padding()), // Top vertical padding
          Constraint::Length(1),                        // Date and time
          Constraint::Min(1),                           // Main area
          Constraint::Length(1),                        // Status line
          Constraint::Length(greeter.window_padding()), // Bottom vertical padding
        ]
        .as_ref(),
      )
      .split(size);

    if greeter.config().opt_present("time") {
      let time_text = Span::from(get_time(&greeter));
      let time = Paragraph::new(time_text).alignment(Alignment::Center);

      f.render_widget(time, chunks[TITLEBAR_INDEX]);
    }

    let status_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
        [
          Constraint::Length(greeter.window_padding()),
          Constraint::Percentage(50),
          Constraint::Percentage(50),
          Constraint::Length(greeter.window_padding()),
        ]
        .as_ref(),
      )
      .split(chunks[STATUSBAR_INDEX]);

    let command = greeter.command.clone().unwrap_or_else(|| "-".to_string());
    let status_left_text = Spans::from(vec![
      status_label("ESC"),
      status_value(fl!("action_reset")),
      status_label("F2"),
      status_value(fl!("action_command")),
      status_label("F3"),
      status_value(fl!("action_session")),
      status_label("F12"),
      status_value(fl!("action_power")),
      status_label(fl!("status_command")),
      status_value(command),
    ]);
    let status_left = Paragraph::new(status_left_text);

    f.render_widget(status_left, status_chunks[STATUSBAR_LEFT_INDEX]);

    if capslock_status() {
      let status_right_text = status_label(fl!("status_caps"));
      let status_right = Paragraph::new(status_right_text).alignment(Alignment::Right);

      f.render_widget(status_right, status_chunks[STATUSBAR_RIGHT_INDEX]);
    }

    let cursor = match greeter.mode {
      Mode::Command => self::command::draw(&mut greeter, &mut f).ok(),
      Mode::Sessions => self::sessions::draw(&mut greeter, &mut f).ok(),
      Mode::Power => self::power::draw(&mut greeter, &mut f).ok(),
      _ => self::prompt::draw(&mut greeter, &mut f).ok(),
    };

    if !hide_cursor {
      if let Some(cursor) = cursor {
        f.set_cursor(cursor.0 - 1, cursor.1 - 1);
      }
    }
  })?;

  io::stdout().flush()?;

  Ok(())
}

fn get_time(greeter: &Greeter) -> String {
  Local::now().format_localized(&fl!("date"), greeter.locale).to_string()
}

fn status_label<'s, S>(text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::styled(text.into(), Style::default().add_modifier(Modifier::REVERSED))
}

fn status_value<'s, S>(text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::from(titleize(&text.into()))
}

fn prompt_value<'s, S>(text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::styled(text.into(), Style::default().add_modifier(Modifier::BOLD))
}
