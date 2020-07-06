use std::{error::Error, io};

use termion::raw::RawTerminal;
use tui::{
  backend::TermionBackend,
  layout::{Constraint, Direction, Layout, Rect},
  widgets::{Block, BorderType, Borders, Paragraph, Text},
  Frame,
};

use super::prompt_value;
use crate::{ui::util::*, Greeter};

const CHANGE_COMMAND: &str = " Change session command ";
const COMMAND: &str = "New command:";

pub fn draw(greeter: &mut Greeter, f: &mut Frame<TermionBackend<RawTerminal<io::Stdout>>>) -> Result<(u16, u16), Box<dyn Error>> {
  let size = f.size();

  let width = greeter.width();
  let height = get_height(&greeter);
  let container_padding = greeter.container_padding();
  let x = (size.width - width) / 2;
  let y = (size.height - height) / 2;

  let container = Rect::new(x, y, width, height);
  let frame = Rect::new(x + container_padding, y + container_padding, width - container_padding, height - container_padding);

  let block = Block::default().title(CHANGE_COMMAND).borders(Borders::ALL).border_type(BorderType::Plain);

  f.render_widget(block, container);

  let constraints = [
    Constraint::Length(1), // Username
  ];

  let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints.as_ref()).split(frame);
  let cursor = chunks[0];

  let command_label_text = [prompt_value(COMMAND)];
  let command_label = Paragraph::new(command_label_text.iter());
  let command_value_text = [Text::raw(&greeter.new_command)];
  let command_value = Paragraph::new(command_value_text.iter());

  f.render_widget(command_label, chunks[0]);
  f.render_widget(command_value, Rect::new(1 + chunks[0].x + COMMAND.len() as u16, chunks[0].y, get_input_width(greeter, COMMAND), 1));

  let offset = get_cursor_offset(greeter, greeter.new_command.chars().count());

  Ok((2 + cursor.x + COMMAND.len() as u16 + offset as u16, cursor.y + 1))
}
