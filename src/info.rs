use std::{env, fs};

use nix::sys::utsname;

pub fn get_hostname() -> String {
    utsname::uname().nodename().to_string()
}

pub fn get_issue() -> Option<String> {
    let vtnr: usize = env::var("XDG_VTNR")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("unable to parse VTNR");

    let uts = utsname::uname();

    if let Ok(issue) = fs::read_to_string("/etc/issue") {
        return Some(
            issue
                .replace("\\S", "Linux")
                .replace("\\l", &format!("tty{}", vtnr))
                .replace("\\s", &uts.sysname())
                .replace("\\r", &uts.release())
                .replace("\\v", &uts.version())
                .replace("\\n", &uts.nodename())
                .replace("\\m", &uts.machine())
                .replace("\\\\", "\\"),
        );
    }

    None
}
