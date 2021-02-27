use std::{
  env,
  error::Error,
  fs::{self, File},
  io,
  os::unix::io::AsRawFd,
  path::Path,
};

use ini::Ini;
use nix::sys::utsname;
use nix::unistd::isatty;
use nix::ioctl_read_bad;

const X_SESSIONS: &str = "/usr/share/xsessions";
const WAYLAND_SESSIONS: &str = "/usr/share/wayland-sessions";
const LAST_USERNAME: &str = "/var/cache/tuigreet/lastuser";

pub fn get_hostname() -> String {
  utsname::uname().nodename().to_string()
}

pub fn get_issue() -> Option<String> {
  let vtnr: usize = env::var("XDG_VTNR").as_deref().unwrap_or("0").parse().expect("unable to parse VTNR");
  let uts = utsname::uname();

  fs::read_to_string("/etc/issue").ok()
    .map(|issue| {
      let mut ret = String::new();
      let mut itr = issue.chars();
      while let Some(c) = itr.next() {
        if c != '\\' {
          ret.push(c);
        } else {
          match itr.next() {
            Some('S') => ret.push_str("Linux"),
            Some('l') => ret.push_str(&format!("tty{}", vtnr)),
            Some('s') => ret.push_str(uts.sysname()),
            Some('r') => ret.push_str(uts.release()),
            Some('v') => ret.push_str(uts.version()),
            Some('n') => ret.push_str(uts.nodename()),
            Some('m') => ret.push_str(uts.machine()),
            Some('\\') => ret.push('\\'),
            Some(c) => ret.push(c),
            _ => ret.push('\\'),
          }
        }
      }
      ret
    })
}

pub fn get_last_username() -> Result<String, io::Error> {
  fs::read_to_string(LAST_USERNAME)
}

pub fn write_last_username(username: &str) {
  let _ = fs::write(LAST_USERNAME, username);
}

pub fn delete_last_username() {
  let _ = fs::remove_file(LAST_USERNAME);
}

pub fn get_sessions() -> Result<Vec<(String, String)>, Box<dyn Error>> {
  let directories = vec![X_SESSIONS, WAYLAND_SESSIONS];

  let files = directories
    .iter()
    .flat_map(|directory| fs::read_dir(&directory))
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
  for f in &["/proc/self/fd/0", "/dev/tty", "/dev/tty0", "/dev/vc/0", "/dev/systty", "/dev/console"] {
    if let Ok(f) = File::open(f) {
      let mut arg = 0;
      if
        isatty(f.as_raw_fd()) == Ok(true) &&
        unsafe { kdgkbtype(f.as_raw_fd(), &mut arg) } == Ok(0) &&
        (arg == KB_101 || arg == KB_84) &&
        unsafe { kdgkbled(f.as_raw_fd(), &mut arg) } == Ok(0)
      {
        return arg & K_CAPSLOCK != 0;
      }
    }
  }

  return false;
}

const KDGKBTYPE: u64 = 0x4B33;
const KB_84: u8 = 0x01;
const KB_101: u8 = 0x02;

const KDGKBLED: u64 = 0x4B64;  /* get led flags (not lights) */
const K_CAPSLOCK: u8 = 0x04;

ioctl_read_bad!(kdgkbtype, KDGKBTYPE, u8);
ioctl_read_bad!(kdgkbled, KDGKBLED, u8);
