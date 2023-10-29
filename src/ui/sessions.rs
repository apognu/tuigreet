use std::path::{Path, PathBuf};

use crate::Greeter;

use super::common::menu::MenuItem;

#[derive(SmartDefault, Debug, Copy, Clone, PartialEq)]
pub enum SessionType {
  X11,
  Wayland,
  Tty,
  #[default]
  None,
}

impl SessionType {
  pub fn as_xdg_session_type(&self) -> &'static str {
    match self {
      SessionType::X11 => "x11",
      SessionType::Wayland => "wayland",
      SessionType::Tty => "tty",
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

impl Session {
  pub fn from_path<P>(greeter: &Greeter, path: P) -> Option<&Session>
  where
    P: AsRef<Path>,
  {
    greeter.sessions.options.iter().find(|session| session.path.as_deref() == Some(path.as_ref()))
  }

  pub fn get_selected(greeter: &Greeter) -> Option<&Session> {
    greeter.session_path.as_ref()?;
    greeter.sessions.options.get(greeter.sessions.selected)
  }
}

#[cfg(test)]
mod test {
  use crate::{
    ui::{
      common::menu::Menu,
      sessions::{Session, SessionType},
    },
    Greeter,
  };

  #[test]
  fn from_path_existing() {
    let mut greeter = Greeter::default();
    greeter.session_path = Some("/Session2Path".into());

    greeter.sessions = Menu::<Session> {
      title: "Sessions".into(),
      selected: 1,
      options: vec![
        Session {
          name: "Session1".into(),
          command: "Session1Cmd".into(),
          session_type: super::SessionType::Wayland,
          path: Some("/Session1Path".into()),
        },
        Session {
          name: "Session2".into(),
          command: "Session2Cmd".into(),
          session_type: super::SessionType::X11,
          path: Some("/Session2Path".into()),
        },
      ],
    };

    let session = Session::from_path(&greeter, "/Session2Path");

    assert!(matches!(session, Some(_)));
    assert_eq!(session.unwrap().name, "Session2");
    assert_eq!(session.unwrap().session_type, SessionType::X11);
  }

  #[test]
  fn from_path_non_existing() {
    let mut greeter = Greeter::default();
    greeter.session_path = Some("/Session2Path".into());

    greeter.sessions = Menu::<Session> {
      title: "Sessions".into(),
      selected: 1,
      options: vec![Session {
        name: "Session1".into(),
        command: "Session1Cmd".into(),
        session_type: super::SessionType::Wayland,
        path: Some("/Session1Path".into()),
      }],
    };

    let session = Session::from_path(&greeter, "/Session2Path");

    assert!(matches!(session, None));
  }

  #[test]
  fn no_session() {
    let greeter = Greeter::default();

    assert!(matches!(Session::get_selected(&greeter), None));
  }

  #[test]
  fn distinct_session() {
    let mut greeter = Greeter::default();
    greeter.session_path = Some("/Session2Path".into());

    greeter.sessions = Menu::<Session> {
      title: "Sessions".into(),
      selected: 1,
      options: vec![
        Session {
          name: "Session1".into(),
          command: "Session1Cmd".into(),
          session_type: super::SessionType::Wayland,
          path: Some("/Session1Path".into()),
        },
        Session {
          name: "Session2".into(),
          command: "Session2Cmd".into(),
          session_type: super::SessionType::X11,
          path: Some("/Session2Path".into()),
        },
      ],
    };

    let session = Session::get_selected(&greeter);

    assert!(matches!(session, Some(_)));
    assert_eq!(session.unwrap().name, "Session2");
    assert_eq!(session.unwrap().session_type, SessionType::X11);
  }

  #[test]
  fn same_name_session() {
    let mut greeter = Greeter::default();
    greeter.session_path = Some("/Session2Path".into());

    greeter.sessions = Menu::<Session> {
      title: "Sessions".into(),
      selected: 1,
      options: vec![
        Session {
          name: "Session".into(),
          command: "Session1Cmd".into(),
          session_type: super::SessionType::Wayland,
          path: Some("/Session1Path".into()),
        },
        Session {
          name: "Session".into(),
          command: "Session2Cmd".into(),
          session_type: super::SessionType::X11,
          path: Some("/Session2Path".into()),
        },
      ],
    };

    let session = Session::get_selected(&greeter);

    assert!(matches!(session, Some(_)));
    assert_eq!(session.unwrap().name, "Session");
    assert_eq!(session.unwrap().session_type, SessionType::X11);
    assert_eq!(session.unwrap().command, "Session2Cmd");
  }
}
