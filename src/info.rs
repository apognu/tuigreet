use std::{
  env,
  error::Error,
  fs, io,
  path::{Path, PathBuf},
  process::Command,
};

use ini::Ini;
use nix::sys::utsname;

use crate::Greeter;

const X_SESSIONS: &str = "/usr/share/xsessions";
const WAYLAND_SESSIONS: &str = "/usr/share/wayland-sessions";
const LAST_USERNAME: &str = "/var/cache/tuigreet/lastuser";
const LAST_SESSION: &str = "/var/cache/tuigreet/lastsession";

pub fn get_hostname() -> String {
  utsname::uname().nodename().to_string()
}

pub fn get_issue() -> Option<String> {
  let vtnr: usize = env::var("XDG_VTNR").unwrap_or_else(|_| "0".to_string()).parse().expect("unable to parse VTNR");
  let uts = utsname::uname();

  if let Ok(issue) = fs::read_to_string("/etc/issue") {
    return Some(
      issue
        .replace("\\S", "Linux")
        .replace("\\l", &format!("tty{}", vtnr))
        .replace("\\s", uts.sysname())
        .replace("\\r", uts.release())
        .replace("\\v", uts.version())
        .replace("\\n", uts.nodename())
        .replace("\\m", uts.machine())
        .replace("\\\\", "\\"),
    );
  }

  None
}

pub fn get_last_username() -> Result<String, io::Error> {
  fs::read_to_string(LAST_USERNAME)
}

pub fn write_last_username(username: &str) {
  let _ = fs::write(LAST_USERNAME, username);
}

pub fn get_last_session() -> Result<String, io::Error> {
  fs::read_to_string(LAST_SESSION)
}

pub fn write_last_session(session: &str) {
  let _ = fs::write(LAST_SESSION, session);
}

pub fn get_sessions(greeter: &Greeter) -> Result<Vec<(String, String)>, Box<dyn Error>> {
  let sessions = match greeter.sessions_path {
    Some(ref dirs) => env::split_paths(&dirs).collect(),
    None => vec![PathBuf::from(X_SESSIONS), PathBuf::from(WAYLAND_SESSIONS)],
  };

  let files = sessions
    .iter()
    .flat_map(fs::read_dir)
    .flat_map(|directory| directory.flat_map(|entry| entry.map(|entry| load_desktop_file(entry.path()))).flatten())
    .collect::<Vec<_>>();

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
