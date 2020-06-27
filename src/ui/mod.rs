mod prompt;

use std::{
    error::Error,
    io::{self, Write},
};

use termion::{cursor::Goto, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Paragraph, Text},
    Terminal,
};

use crate::Greeter;

const COMMAND: &'static str = "SESSION";
const CHANGE_COMMAND: &'static str = "(use !cmd to change it)";

pub fn draw(
    terminal: &mut Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
    greeter: &mut Greeter,
) -> Result<(), Box<dyn Error>> {
    if greeter.working {
        terminal.hide_cursor()?;
    } else {
        terminal.show_cursor()?;
    }

    let mut cursor: Option<(u16, u16)> = None;

    terminal.draw(|mut f| {
        let size = f.size();
        let chunks = Layout::default()
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(size);

        let status = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[1]);

        let line = format!(
            "ESC Reset | {} {} {}",
            COMMAND,
            greeter.command.clone().unwrap_or("-".to_string()),
            CHANGE_COMMAND
        );
        let t = [Text::raw(&line)];
        let p = Paragraph::new(t.iter());

        f.render_widget(p, status[0]);

        cursor = self::prompt::draw(greeter, &mut f).ok();
    })?;

    if let Some(cursor) = cursor {
        write!(terminal.backend_mut(), "{}", Goto(cursor.0, cursor.1))?;
    }

    io::stdout().flush()?;

    Ok(())
}
