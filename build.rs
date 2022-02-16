use std::{env, error::Error, path::Path, process::Command};

fn main() {
  let version = if Path::new(".git").exists() {
    get_git_version().unwrap_or_else(|_| String::from("unknown"))
  } else {
    env!("CARGO_PKG_VERSION").to_string()
  };

  println!("cargo:rustc-env=VERSION={version}");
  println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
}

fn get_git_version() -> Result<String, Box<dyn Error>> {
  Ok(String::from_utf8(Command::new("./contrib/git-version.sh").output()?.stdout)?)
}
