use std::{
  env,
  error::Error,
  fs::{self, File},
  io::{self, BufRead, BufReader},
  path::{Path, PathBuf},
  process::Command,
};

use ini::Ini;
use nix::sys::utsname;

use crate::Greeter;

const X_SESSIONS: &str = "/usr/share/xsessions";
const WAYLAND_SESSIONS: &str = "/usr/share/wayland-sessions";
const LAST_USER_USERNAME: &str = "/var/cache/tuigreet/lastuser";
const LAST_USER_NAME: &str = "/var/cache/tuigreet/lastuser-name";
const LAST_SESSION: &str = "/var/cache/tuigreet/lastsession";

const DEFAULT_MIN_UID: u16 = 1000;
const DEFAULT_MAX_UID: u16 = 60000;

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

pub fn get_last_user_username() -> Result<String, io::Error> {
  fs::read_to_string(LAST_USER_USERNAME)
}

pub fn get_last_user_name() -> Option<String> {
  fs::read_to_string(LAST_USER_NAME).ok()
}

pub fn write_last_username(username: &str, name: Option<&str>) {
  let _ = fs::write(LAST_USER_USERNAME, username);

  if let Some(name) = name {
    let _ = fs::write(LAST_USER_NAME, name);
  } else {
    let _ = fs::remove_file(LAST_USER_NAME);
  }
}

pub fn get_last_session() -> Result<String, io::Error> {
  fs::read_to_string(LAST_SESSION)
}

pub fn write_last_session(session: &str) {
  let _ = fs::write(LAST_SESSION, session);
}

pub fn get_last_user_session(username: &str) -> Result<String, io::Error> {
  fs::read_to_string(format!("{LAST_SESSION}-{username}"))
}

pub fn write_last_user_session(username: &str, session: &str) {
  let _ = fs::write(format!("{LAST_SESSION}-{username}"), session);
}

pub fn get_users(min_uid: u16, max_uid: u16) -> Vec<(String, Option<String>)> {
  match File::open("/etc/passwd") {
    Err(_) => vec![],
    Ok(file) => {
      let file = BufReader::new(file);

      let users: Vec<(String, Option<String>)> = file
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
            .map(|(_, username, name)| (username, name))
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

pub fn get_sessions(greeter: &Greeter) -> Result<Vec<(String, String)>, Box<dyn Error>> {
  let sessions = match greeter.sessions_path {
    Some(ref dirs) => env::split_paths(&dirs).collect(),
    None => vec![PathBuf::from(X_SESSIONS), PathBuf::from(WAYLAND_SESSIONS)],
  };

  let mut files = sessions
    .iter()
    .flat_map(fs::read_dir)
    .flat_map(|directory| directory.flat_map(|entry| entry.map(|entry| load_desktop_file(entry.path()))).flatten())
    .collect::<Vec<_>>();

  if let Some(command) = &greeter.command {
    files.insert(0, (command.clone(), command.clone()));
  }

  Ok(files)
}

fn load_desktop_file<P>(path: P) -> Result<(String, String), Box<dyn Error>>
where
  P: AsRef<Path>,
{
  let desktop = Ini::load_from_file(path)?;
  let section = desktop.section(Some("Desktop Entry")).ok_or("no Desktop Entry section in desktop file")?;

  let name = section.get("Name").ok_or("no Name property in desktop file")?;
  let exec = section.get("Exec").ok_or("no Exec property in desktop file")?;

  Ok((name.to_string(), exec.to_string()))
}

pub fn capslock_status() -> bool {
  let mut command = Command::new("kbdinfo");
  command.args(["gkbled", "capslock"]);

  match command.output() {
    Ok(output) => output.status.code() == Some(0),
    Err(_) => false,
  }
}
