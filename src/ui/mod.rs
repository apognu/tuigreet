mod command;
mod power;
mod prompt;
mod sessions;
mod util;

use std::{
  error::Error,
  io::{self, Write},
};

use chrono::prelude::*;
use termion::raw::RawTerminal;
use tui::{
  backend::TermionBackend,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Modifier, Style},
  text::{Span, Spans},
  widgets::Paragraph,
  Terminal,
};

use crate::{info::capslock_status, Greeter, Mode};

pub use self::power::{Option as PowerOption, OPTIONS as POWER_OPTIONS};

const SESSIONS: &str = "Choose session";
const CHANGE_COMMAND: &str = "Change command";
const COMMAND: &str = "COMMAND";
const POWER: &str = "POWER";
const CAPS_LOCK: &str = "CAPS LOCK";

const TITLEBAR_INDEX: usize = 1;
const STATUSBAR_INDEX: usize = 3;
const STATUSBAR_LEFT_INDEX: usize = 1;
const STATUSBAR_RIGHT_INDEX: usize = 2;

pub fn draw(terminal: &mut Terminal<TermionBackend<RawTerminal<io::Stdout>>>, greeter: &mut Greeter) -> Result<(), Box<dyn Error>> {
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
      let time_text = Span::from(get_time());
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
      status_label("F2"),
      status_value(CHANGE_COMMAND),
      status_label("F3"),
      status_value(SESSIONS),
      status_label("F12"),
      status_value(POWER),
      status_label(COMMAND),
      status_value(command),
    ]);
    let status_left = Paragraph::new(status_left_text);

    f.render_widget(status_left, status_chunks[STATUSBAR_LEFT_INDEX]);

    if capslock_status() {
      let status_right_text = status_label(format!(" {} ", CAPS_LOCK));
      let status_right = Paragraph::new(status_right_text).alignment(Alignment::Right);

      f.render_widget(status_right, status_chunks[STATUSBAR_RIGHT_INDEX]);
    }

    let cursor = match greeter.mode {
      Mode::Command => self::command::draw(greeter, &mut f).ok(),
      Mode::Sessions => self::sessions::draw(greeter, &mut f).ok(),
      Mode::Power => self::power::draw(greeter, &mut f).ok(),
      _ => self::prompt::draw(greeter, &mut f).ok(),
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

fn get_time() -> String {
  Local::now().format("%a, %d %h %Y - %H:%M").to_string()
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
  Span::from(format!(" {} ", text.into()))
}

fn prompt_value<'s, S>(text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::styled(text.into(), Style::default().add_modifier(Modifier::BOLD))
}
