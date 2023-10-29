use std::{
  env,
  error::Error,
  fs::{self, File},
  io::{self, BufRead, BufReader},
  path::{Path, PathBuf},
  process::Command,
};

use ini::Ini;
use lazy_static::lazy_static;
use nix::sys::utsname;

use crate::{
  ui::{
    sessions::{Session, SessionType},
    users::User,
  },
  Greeter,
};

const LAST_USER_USERNAME: &str = "/var/cache/tuigreet/lastuser";
const LAST_USER_NAME: &str = "/var/cache/tuigreet/lastuser-name";
const LAST_SESSION: &str = "/var/cache/tuigreet/lastsession";
const LAST_SESSION_PATH: &str = "/var/cache/tuigreet/lastsession-path";

const DEFAULT_MIN_UID: u16 = 1000;
const DEFAULT_MAX_UID: u16 = 60000;

lazy_static! {
  static ref XDG_DATA_DIRS: Vec<PathBuf> = {
    let value = env::var("XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share".to_string());
    env::split_paths(&value).filter(|p| p.is_absolute()).collect()
  };
  static ref DEFAULT_SESSION_PATHS: Vec<(PathBuf, SessionType)> = XDG_DATA_DIRS
    .iter()
    .map(|p| (p.join("wayland-sessions"), SessionType::Wayland))
    .chain(XDG_DATA_DIRS.iter().map(|p| (p.join("xsessions"), SessionType::X11)))
    .collect();
}

pub fn get_hostname() -> String {
  match utsname::uname() {
    Ok(uts) => uts.nodename().to_str().unwrap_or("").to_string(),
    _ => String::new(),
  }
}

pub fn get_issue() -> Option<String> {
  let vtnr: usize = env::var("XDG_VTNR").unwrap_or_else(|_| "0".to_string()).parse().expect("unable to parse VTNR");
  let uts = utsname::uname();

  if let Ok(issue) = fs::read_to_string("/etc/issue") {
    let issue = issue.replace("\\S", "Linux").replace("\\l", &format!("tty{vtnr}"));

    return match uts {
      Ok(uts) => Some(
        issue
          .replace("\\s", uts.sysname().to_str().unwrap_or(""))
          .replace("\\r", uts.release().to_str().unwrap_or(""))
          .replace("\\v", uts.version().to_str().unwrap_or(""))
          .replace("\\n", uts.nodename().to_str().unwrap_or(""))
          .replace("\\m", uts.machine().to_str().unwrap_or(""))
          .replace("\\\\", "\\"),
      ),

      _ => Some(issue),
    };
  }

  None
}

pub fn get_last_user_username() -> Option<String> {
  match fs::read_to_string(LAST_USER_USERNAME).ok() {
    None => None,
    Some(username) => {
      let username = username.trim();

      if username.is_empty() {
        None
      } else {
        Some(username.to_string())
      }
    }
  }
}

pub fn get_last_user_name() -> Option<String> {
  match fs::read_to_string(LAST_USER_NAME).ok() {
    None => None,
    Some(name) => {
      let name = name.trim();

      if name.is_empty() {
        None
      } else {
        Some(name.to_string())
      }
    }
  }
}

pub fn write_last_username(username: &str, name: Option<&str>) {
  let _ = fs::write(LAST_USER_USERNAME, username);

  if let Some(name) = name {
    let _ = fs::write(LAST_USER_NAME, name);
  } else {
    let _ = fs::remove_file(LAST_USER_NAME);
  }
}

pub fn get_last_session_path() -> Result<PathBuf, io::Error> {
  Ok(PathBuf::from(fs::read_to_string(LAST_SESSION_PATH)?.trim()))
}

pub fn get_last_session() -> Result<String, io::Error> {
  Ok(fs::read_to_string(LAST_SESSION)?.trim().to_string())
}

pub fn write_last_session_path<P>(session: &P)
where
  P: AsRef<Path>,
{
  let _ = fs::write(LAST_SESSION_PATH, session.as_ref().to_string_lossy().as_bytes());
}

pub fn write_last_session(session: &str) {
  let _ = fs::write(LAST_SESSION, session);
}

pub fn get_last_user_session_path(username: &str) -> Result<PathBuf, io::Error> {
  Ok(PathBuf::from(fs::read_to_string(format!("{LAST_SESSION_PATH}-{username}"))?.trim()))
}

pub fn get_last_user_session(username: &str) -> Result<String, io::Error> {
  Ok(fs::read_to_string(format!("{LAST_SESSION}-{username}"))?.trim().to_string())
}

pub fn write_last_user_session_path<P>(username: &str, session: P)
where
  P: AsRef<Path>,
{
  let _ = fs::write(format!("{LAST_SESSION_PATH}-{username}"), session.as_ref().to_string_lossy().as_bytes());
}

pub fn delete_last_session_path() {
  let _ = fs::remove_file(LAST_SESSION_PATH);
}

pub fn write_last_user_session(username: &str, session: &str) {
  let _ = fs::write(format!("{LAST_SESSION}-{username}"), session);
}

pub fn delete_last_user_session_path(username: &str) {
  let _ = fs::remove_file(format!("{LAST_SESSION_PATH}-{username}"));
}

pub fn get_users(min_uid: u16, max_uid: u16) -> Vec<User> {
  match File::open("/etc/passwd") {
    Err(_) => vec![],
    Ok(file) => {
      let file = BufReader::new(file);

      let users: Vec<User> = file
        .lines()
        .filter_map(|line| {
          line
            .map(|line| {
              let mut split = line.splitn(7, ':');
              let username = split.next();
              let uid = split.nth(1);
              let name = split.nth(1);

              match uid.map(|uid| uid.parse::<u16>()) {
                Some(Ok(uid)) => match (username, name) {
                  (Some(username), Some("")) => Some((uid, username.to_string(), None)),
                  (Some(username), Some(name)) => Some((uid, username.to_string(), Some(name.to_string()))),
                  _ => None,
                },

                _ => None,
              }
            })
            .ok()
            .flatten()
            .filter(|(uid, _, _)| uid >= &min_uid && uid <= &max_uid)
            .map(|(_, username, name)| User { username, name })
        })
        .collect();

      users
    }
  }
}

pub fn get_min_max_uids(min_uid: Option<u16>, max_uid: Option<u16>) -> (u16, u16) {
  if let (Some(min_uid), Some(max_uid)) = (min_uid, max_uid) {
    return (min_uid, max_uid);
  }

  let overrides = (min_uid, max_uid);
  let default = (min_uid.unwrap_or(DEFAULT_MIN_UID), max_uid.unwrap_or(DEFAULT_MAX_UID));

  match File::open("/etc/login.defs") {
    Err(_) => default,
    Ok(file) => {
      let file = BufReader::new(file);

      let uids: (u16, u16) = file.lines().fold(default, |acc, line| {
        line
          .map(|line| {
            let mut tokens = line.split_whitespace();

            match (overrides, tokens.next(), tokens.next()) {
              ((None, _), Some("UID_MIN"), Some(value)) => (value.parse::<u16>().unwrap_or(acc.0), acc.1),
              ((_, None), Some("UID_MAX"), Some(value)) => (acc.0, value.parse::<u16>().unwrap_or(acc.1)),
              _ => acc,
            }
          })
          .unwrap_or(acc)
      });

      uids
    }
  }
}

pub fn get_sessions(greeter: &Greeter) -> Result<Vec<Session>, Box<dyn Error>> {
  let paths = if greeter.session_paths.is_empty() {
    DEFAULT_SESSION_PATHS.as_ref()
  } else {
    &greeter.session_paths
  };

  let mut files = match &greeter.command {
    Some(command) => vec![Session {
      name: command.clone(),
      command: command.clone(),
      session_type: SessionType::default(),
      path: None,
    }],
    _ => vec![],
  };

  for (path, session_type) in paths.iter() {
    if let Ok(entries) = fs::read_dir(path) {
      files.extend(entries.flat_map(|entry| entry.map(|entry| load_desktop_file(entry.path(), *session_type))).flatten());
    }
  }
  Ok(files)
}

fn load_desktop_file<P>(path: P, session_type: SessionType) -> Result<Session, Box<dyn Error>>
where
  P: AsRef<Path>,
{
  let desktop = Ini::load_from_file(path.as_ref())?;
  let section = desktop.section(Some("Desktop Entry")).ok_or("no Desktop Entry section in desktop file")?;

  let name = section.get("Name").ok_or("no Name property in desktop file")?;
  let exec = section.get("Exec").ok_or("no Exec property in desktop file")?;

  Ok(Session {
    name: name.to_string(),
    command: exec.to_string(),
    session_type,
    path: Some(path.as_ref().into()),
  })
}

pub fn capslock_status() -> bool {
  let mut command = Command::new("kbdinfo");
  command.args(["gkbled", "capslock"]);

  match command.output() {
    Ok(output) => output.status.code() == Some(0),
    Err(_) => false,
  }
}
