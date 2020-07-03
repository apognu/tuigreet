use std::{error::Error, io};

use termion::raw::RawTerminal;
use tui::{
  backend::TermionBackend,
  layout::Rect,
  style::{Modifier, Style},
  widgets::{Block, BorderType, Borders, Paragraph, Text},
  Frame,
};

use crate::Greeter;

const CHANGE_SESSION: &str = "Change session";

pub fn draw(greeter: &mut Greeter, f: &mut Frame<TermionBackend<RawTerminal<io::Stdout>>>) -> Result<(u16, u16), Box<dyn Error>> {
  let size = f.size();

  let width = greeter.width();
  let height: u16 = greeter.sessions.len() as u16 + 4;
  let x = (size.width - width) / 2;
  let y = (size.height - height) / 2;

  let container = Rect::new(x, y, width, height);

  let title = format!(" {} ", CHANGE_SESSION);
  let block = Block::default().title(&title).borders(Borders::ALL).border_type(BorderType::Plain);

  for (index, (name, _)) in greeter.sessions.iter().enumerate() {
    let frame = Rect::new(x + 2, y + 2 + index as u16, width, 1);
    let option_text = [get_option(&greeter, name, index)];
    let option = Paragraph::new(option_text.iter());

    f.render_widget(option, frame);
  }

  f.render_widget(block, container);

  Ok((1, 1))
}

fn get_option<'g, S>(greeter: &Greeter, name: S, index: usize) -> Text<'g>
where
  S: Into<String>,
{
  if greeter.selected_session == index {
    Text::styled(name.into(), Style::default().modifier(Modifier::REVERSED))
  } else {
    Text::raw(name.into())
  }
}
