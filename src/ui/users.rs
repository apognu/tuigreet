use std::borrow::Cow;

use super::common::menu::MenuItem;

#[derive(Default, Clone)]
pub struct User {
  pub username: String,
  pub name: Option<String>,
}

impl MenuItem for User {
  fn format(&self) -> Cow<'_, str> {
    match &self.name {
      Some(name) => Cow::Owned(format!("{name} ({})", self.username)),
      None => Cow::Borrowed(&self.username),
    }
  }
}
