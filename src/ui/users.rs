use super::common::menu::MenuItem;

#[derive(Default, Clone)]
pub struct User {
  pub username: String,
  pub name: Option<String>,
}

impl MenuItem for User {
  fn format(&self) -> String {
    match &self.name {
      Some(name) => format!("{name} ({})", self.username),
      None => self.username.clone(),
    }
  }
}
