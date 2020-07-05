use std::{env, error::Error, path::Path, process::Command};

fn main() {
  let version = if Path::new(".git").exists() {
    get_git_version().unwrap_or_else(|_| String::from("unknown"))
  } else {
    env!("CARGO_PKG_VERSION").to_string()
  };

  println!("cargo:rustc-env=VERSION={}", version);
  println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
}

fn get_git_version() -> Result<String, Box<dyn Error>> {
  let tag = Command::new("git").args(&["describe", "--abbrev=0"]).output()?;
  let tag = match tag.status.code() {
    Some(0) => String::from_utf8(tag.stdout)?,
    _ => "0.0.0".to_string(),
  };
  let count = String::from_utf8(Command::new("git").args(&["rev-list", "--count", "HEAD"]).output()?.stdout)?;
  let commit = String::from_utf8(Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?.stdout)?;
  let version = format!("{}.r{}.{}", tag.trim(), count.trim(), commit.trim());

  match version.as_str() {
    "0.0.0.r." => Err("could not retrieve version".into()),
    version => Ok(version.to_string()),
  }
}
