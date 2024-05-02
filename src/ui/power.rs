use std::borrow::Cow;

use crate::{power::PowerOption, ui::common::menu::MenuItem};

#[derive(SmartDefault, Clone)]
pub struct Power {
  pub action: PowerOption,
  pub label: String,
  pub command: Option<String>,
}

impl MenuItem for Power {
  fn format(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.label)
  }
}
