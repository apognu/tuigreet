#![allow(unused_must_use)]

/*
  Copied and adapted from the codebase of ratatui.

  Repository: https://github.com/ratatui-org/ratatui
  License: https://github.com/ratatui-org/ratatui/blob/main/LICENSE
  File: https://github.com/ratatui-org/ratatui/blob/f4637d40c35e068fd60d17c9a42b9114667c9861/src/backend/test.rs

  The MIT License (MIT)

  Copyright (c) 2016-2022 Florian Dehau
  Copyright (c) 2023-2024 The Ratatui Developers

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.
*/
use std::{
  fmt::Write,
  io,
  sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

use tui::{
  backend::{Backend, ClearType, WindowSize},
  buffer::{Buffer, Cell},
  layout::{Rect, Size},
};

#[derive(Clone)]
pub struct TestBackend {
  tick: mpsc::Sender<bool>,
  width: u16,
  buffer: Arc<Mutex<Buffer>>,
  height: u16,
  cursor: bool,
  pos: (u16, u16),
}

pub fn output(buffer: &Arc<Mutex<Buffer>>) -> String {
  let buffer = buffer.lock().unwrap();

  let mut view = String::with_capacity(buffer.content.len() + buffer.area.height as usize * 3);
  for cells in buffer.content.chunks(buffer.area.width as usize) {
    let mut overwritten = vec![];
    let mut skip: usize = 0;
    view.push('"');
    for (x, c) in cells.iter().enumerate() {
      if skip == 0 {
        view.push_str(c.symbol());
      } else {
        overwritten.push((x, c.symbol()));
      }
      skip = std::cmp::max(skip, c.symbol().width()).saturating_sub(1);
    }
    view.push('"');
    if !overwritten.is_empty() {
      write!(&mut view, " Hidden by multi-width symbols: {overwritten:?}").unwrap();
    }
    view.push('\n');
  }
  view
}

impl TestBackend {
  pub fn new(width: u16, height: u16) -> (Self, Arc<Mutex<Buffer>>, mpsc::Receiver<bool>) {
    let buffer = Arc::new(Mutex::new(Buffer::empty(Rect::new(0, 0, width, height))));
    let (tx, rx) = mpsc::channel::<bool>(10);

    let backend = Self {
      tick: tx,
      width,
      height,
      buffer: buffer.clone(),
      cursor: false,
      pos: (0, 0),
    };

    (backend, buffer, rx)
  }
}

impl Backend for TestBackend {
  fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
  where
    I: Iterator<Item = (u16, u16, &'a Cell)>,
  {
    let mut buffer = self.buffer.lock().unwrap();

    for (x, y, c) in content {
      let cell = buffer.get_mut(x, y);
      *cell = c.clone();
    }

    let sender = self.tick.clone();

    std::thread::spawn(move || {
      sender.blocking_send(true);
    });

    Ok(())
  }

  fn hide_cursor(&mut self) -> io::Result<()> {
    self.cursor = false;
    Ok(())
  }

  fn show_cursor(&mut self) -> io::Result<()> {
    self.cursor = true;
    Ok(())
  }

  fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
    Ok(self.pos)
  }

  fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
    self.pos = (x, y);
    Ok(())
  }

  fn clear(&mut self) -> io::Result<()> {
    self.buffer.lock().unwrap().reset();
    Ok(())
  }

  fn clear_region(&mut self, clear_type: tui::backend::ClearType) -> io::Result<()> {
    let buffer = self.buffer.clone();
    let mut buffer = buffer.lock().unwrap();

    match clear_type {
      ClearType::All => self.clear()?,
      ClearType::AfterCursor => {
        let index = buffer.index_of(self.pos.0, self.pos.1) + 1;
        buffer.content[index..].fill(Cell::default());
      }
      ClearType::BeforeCursor => {
        let index = buffer.index_of(self.pos.0, self.pos.1);
        buffer.content[..index].fill(Cell::default());
      }
      ClearType::CurrentLine => {
        let line_start_index = buffer.index_of(0, self.pos.1);
        let line_end_index = buffer.index_of(self.width - 1, self.pos.1);
        buffer.content[line_start_index..=line_end_index].fill(Cell::default());
      }
      ClearType::UntilNewLine => {
        let index = buffer.index_of(self.pos.0, self.pos.1);
        let line_end_index = buffer.index_of(self.width - 1, self.pos.1);
        buffer.content[index..=line_end_index].fill(Cell::default());
      }
    }
    Ok(())
  }

  fn append_lines(&mut self, n: u16) -> io::Result<()> {
    let (cur_x, cur_y) = self.get_cursor()?;

    let new_cursor_x = cur_x.saturating_add(1).min(self.width.saturating_sub(1));

    let max_y = self.height.saturating_sub(1);
    let lines_after_cursor = max_y.saturating_sub(cur_y);
    if n > lines_after_cursor {
      let rotate_by = n.saturating_sub(lines_after_cursor).min(max_y);

      if rotate_by == self.height - 1 {
        self.clear()?;
      }

      self.set_cursor(0, rotate_by)?;
      self.clear_region(ClearType::BeforeCursor)?;
      self.buffer.lock().unwrap().content.rotate_left((self.width * rotate_by).into());
    }

    let new_cursor_y = cur_y.saturating_add(n).min(max_y);
    self.set_cursor(new_cursor_x, new_cursor_y)?;

    Ok(())
  }

  fn size(&self) -> io::Result<Rect> {
    Ok(Rect::new(0, 0, self.width, self.height))
  }

  fn window_size(&mut self) -> io::Result<WindowSize> {
    static WINDOW_PIXEL_SIZE: Size = Size { width: 640, height: 480 };
    Ok(WindowSize {
      columns_rows: (self.width, self.height).into(),
      pixels: WINDOW_PIXEL_SIZE,
    })
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
