use std::error::Error;

use tui::{
  layout::Rect,
  style::{Modifier, Style},
  text::Span,
  widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
  ui::{util::*, Frame},
  Greeter,
};

pub fn draw(greeter: &mut Greeter, f: &mut Frame) -> Result<(u16, u16), Box<dyn Error>> {
  let size = f.size();
  let (x, y, width, height) = get_rect_bounds(greeter, size, greeter.sessions.len());

  let container = Rect::new(x, y, width, height);

  let title = Span::from(titleize(&fl!("title_session")));
  let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Plain);

  for (index, session) in greeter.sessions.iter().enumerate() {
    let name = format!("{:1$}", session.name, greeter.width() as usize - 4);

    let frame = Rect::new(x + 2, y + 2 + index as u16, width - 4, 1);
    let option_text = get_option(greeter, name, index);
    let option = Paragraph::new(option_text);

    f.render_widget(option, frame);
  }

  f.render_widget(block, container);

  Ok((1, 1))
}

fn get_option<'g, S>(greeter: &Greeter, name: S, index: usize) -> Span<'g>
where
  S: Into<String>,
{
  if greeter.selected_session == index {
    Span::styled(name.into(), Style::default().add_modifier(Modifier::REVERSED))
  } else {
    Span::from(name.into())
  }
}
