use std::path::PathBuf;

use super::common::menu::MenuItem;

#[derive(SmartDefault, Debug, Copy, Clone, PartialEq)]
pub enum SessionType {
  X11,
  Wayland,
  TTY,
  #[default]
  None,
}

impl SessionType {
  pub fn to_xdg_session_type(&self) -> &'static str {
    match self {
      SessionType::X11 => "x11",
      SessionType::Wayland => "wayland",
      SessionType::TTY => "tty",
      SessionType::None => "unspecified",
    }
  }
}

#[derive(SmartDefault, Clone)]
pub struct Session {
  pub name: String,
  pub command: String,
  pub session_type: SessionType,
  pub path: Option<PathBuf>,
}

impl MenuItem for Session {
  fn format(&self) -> String {
    self.name.clone()
  }
}
