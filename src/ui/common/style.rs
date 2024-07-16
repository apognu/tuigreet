use std::str::FromStr;

use tui::style::{Color, Style};

#[derive(Clone)]
enum Component {
  Bg,
  Fg,
}

pub enum Themed {
  Container,
  Time,
  Text,
  Border,
  Title,
  Greet,
  Prompt,
  Input,
  Action,
  ActionButton,
}

#[derive(Default)]
pub struct Theme {
  container: Option<(Component, Color)>,
  time: Option<(Component, Color)>,
  text: Option<(Component, Color)>,
  border: Option<(Component, Color)>,
  title: Option<(Component, Color)>,
  greet: Option<(Component, Color)>,
  prompt: Option<(Component, Color)>,
  input: Option<(Component, Color)>,
  action: Option<(Component, Color)>,
  button: Option<(Component, Color)>,
}

impl Theme {
  pub fn parse(spec: &str) -> Theme {
    use Component::*;

    let directives = spec.split(';').filter_map(|directive| directive.split_once('='));
    let mut style = Theme::default();

    for (key, value) in directives {
      if let Ok(color) = Color::from_str(value) {
        match key {
          "container" => style.container = Some((Bg, color)),
          "time" => style.time = Some((Fg, color)),
          "text" => style.text = Some((Fg, color)),
          "border" => style.border = Some((Fg, color)),
          "title" => style.title = Some((Fg, color)),
          "greet" => style.greet = Some((Fg, color)),
          "prompt" => style.prompt = Some((Fg, color)),
          "input" => style.input = Some((Fg, color)),
          "action" => style.action = Some((Fg, color)),
          "button" => style.button = Some((Fg, color)),
          _ => {}
        }
      }
    }

    if style.time.is_none() {
      style.time.clone_from(&style.text);
    }
    if style.greet.is_none() {
      style.greet.clone_from(&style.text);
    }
    if style.title.is_none() {
      style.title.clone_from(&style.border);
    }
    if style.button.is_none() {
      style.button.clone_from(&style.action);
    }

    style
  }

  pub fn of(&self, targets: &[Themed]) -> Style {
    targets.iter().fold(Style::default(), |style, target| self.apply(style, target))
  }

  fn apply(&self, style: Style, target: &Themed) -> Style {
    use Themed::*;

    let color = match target {
      Container => &self.container,
      Time => &self.time,
      Text => &self.text,
      Border => &self.border,
      Title => &self.title,
      Greet => &self.greet,
      Prompt => &self.prompt,
      Input => &self.input,
      Action => &self.action,
      ActionButton => &self.button,
    };

    match color {
      Some((component, color)) => match component {
        Component::Fg => style.fg(*color),
        Component::Bg => style.bg(*color),
      },

      None => style,
    }
  }
}
