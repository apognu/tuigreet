use std::ops::Deref;

pub(in crate::integration) struct Output(pub String);

impl Deref for Output {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[allow(dead_code)]
impl Output {
  pub fn debug_print(&self) {
    for line in self.lines() {
      println!("{}", line);
    }
  }

  pub fn debug_inspect(&self) {
    for line in self.lines() {
      println!("{:?}", line.as_bytes().iter().map(|c| *c as char).collect::<Vec<char>>());
    }
  }
}
