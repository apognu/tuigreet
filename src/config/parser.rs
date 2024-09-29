use std::{env, error::Error};

use chrono::format::{Item, StrftimeItems};

use crate::{
  info::{get_issue, get_min_max_uids, get_users},
  power::PowerOption,
  ui::{
    common::menu::Menu,
    power::Power,
    sessions::{SessionSource, SessionType},
  },
  Greeter, SecretDisplay,
};

const DEFAULT_LOG_FILE: &str = "/tmp/tuigreet.log";
const DEFAULT_ASTERISKS_CHARS: &str = "*";
// `startx` wants an absolute path to the executable as a first argument.
// We don't want to resolve the session command in the greeter though, so it should be additionally wrapped with a known noop command (like `/usr/bin/env`).
pub const DEFAULT_XSESSION_WRAPPER: &str = "startx /usr/bin/env";

impl Greeter {
  pub fn parse_debug(&mut self) {
    let debug = self.config.defaults.debug.clone();

    if self.config().opt_present("debug") || debug.is_some() {
      self.debug = true;

      self.logfile = match self.config().opt_str("debug").or(debug) {
        Some(file) => file.to_string(),
        None => DEFAULT_LOG_FILE.to_string(),
      };
    }
  }

  pub fn parse_greeting(&mut self) -> Result<(), Box<dyn Error>> {
    let has_greeting = self.config().opt_present("greeting") || self.config.ui.greeting.is_some();
    let has_issue = self.config().opt_present("issue") || self.config.ui.use_issue;

    if has_greeting && has_issue {
      return Err("Only one of --issue and --greeting may be used at the same time".into());
    }

    self.greeting = self.option("greeting").or_else(|| self.config.ui.greeting.clone());

    if has_issue {
      self.greeting = get_issue();
    }

    Ok(())
  }

  pub fn parse_asterisks(&mut self) -> Result<(), Box<dyn Error>> {
    let has_asterisks = self.config().opt_present("asterisks") || self.config.ui.show_asterisks;

    if has_asterisks {
      let asterisk = if let Some(value) = self.config().opt_str("asterisks-char").or_else(|| self.config.ui.asterisks_char.map(|c| c.to_string())) {
        if value.chars().count() < 1 {
          return Err("--asterisks-char must have at least one character as its value".into());
        }

        value
      } else {
        DEFAULT_ASTERISKS_CHARS.to_string()
      };

      self.secret_display = SecretDisplay::Character(asterisk);
    }

    Ok(())
  }

  pub fn parse_default_command(&mut self) -> Result<(), Box<dyn Error>> {
    // If the `--cmd` argument is provided, it will override the selected session.
    if let Some(command) = self.option("cmd").or_else(|| self.config.defaults.command.clone()) {
      let envs = self.options_multi("env").or_else(|| self.config.defaults.env.clone());

      if let Some(envs) = &envs {
        for env in envs {
          if !env.contains('=') {
            return Err(format!("malformed environment variable definition for '{env}'").into());
          }
        }
      }

      self.session_source = SessionSource::DefaultCommand(command, envs);
    }

    Ok(())
  }

  pub fn parse_sessions(&mut self) {
    if let Some(dirs) = self.option("sessions") {
      self.session_paths.extend(env::split_paths(&dirs).map(|dir| (dir, SessionType::Wayland)));
    } else if let Some(dirs) = self.config.sessions.wayland_paths.clone() {
      self.session_paths.extend(dirs.into_iter().map(|dir| (dir, SessionType::Wayland)));
    }

    if let Some(dirs) = self.option("xsessions") {
      self.session_paths.extend(env::split_paths(&dirs).map(|dir| (dir, SessionType::X11)));
    } else if let Some(dirs) = self.config.sessions.x11_paths.clone() {
      self.session_paths.extend(dirs.into_iter().map(|dir| (dir, SessionType::X11)));
    }

    if self.option("session-wrapper").is_some() || self.config.sessions.wayland_wrapper.is_some() {
      self.session_wrapper = self.option("session-wrapper").or_else(|| self.config.sessions.wayland_wrapper.clone());
    }

    if !self.config().opt_present("no-xsession-wrapper") && !self.config.sessions.x11_wrapper_disabled {
      self.xsession_wrapper = self
        .option("xsession-wrapper")
        .or_else(|| self.config.sessions.x11_wrapper.clone())
        .or_else(|| Some(DEFAULT_XSESSION_WRAPPER.to_string()));
    }
  }

  pub fn parse_time(&mut self) -> Result<(), Box<dyn Error>> {
    self.time = self.config().opt_present("time") || self.config.ui.show_time;

    if let Some(format) = self.config().opt_str("time-format").or_else(|| self.config.ui.time_format.clone()) {
      if StrftimeItems::new(&format).any(|item| item == Item::Error) {
        return Err("Invalid strftime format provided in --time-format".into());
      }

      self.time_format = Some(format);
    }

    Ok(())
  }

  pub fn parse_menus(&mut self) -> Result<(), Box<dyn Error>> {
    if self.config().opt_present("user-menu") || self.config.ui.show_user_menu {
      self.user_menu = true;

      let min_uid = self.config().opt_str("user-menu-min-uid").and_then(|uid| uid.parse::<u16>().ok()).or(self.config.defaults.user_min_uid);
      let max_uid = self.config().opt_str("user-menu-max-uid").and_then(|uid| uid.parse::<u16>().ok()).or(self.config.defaults.user_max_uid);
      let (min_uid, max_uid) = get_min_max_uids(min_uid, max_uid);

      tracing::info!("min/max UIDs are {}/{}", min_uid, max_uid);

      if min_uid >= max_uid {
        return Err("Minimum UID ({min_uid}) must be less than maximum UID ({max_uid})".into());
      }

      self.users = Menu {
        title: fl!("title_users"),
        options: get_users(min_uid, max_uid),
        selected: 0,
      };

      tracing::info!("found {} users", self.users.options.len());
    }

    Ok(())
  }

  pub fn parse_remembers(&mut self) -> Result<(), Box<dyn Error>> {
    let has_remember = self.config().opt_present("remember") || self.config.remember.last_user;
    let has_remember_session = self.config().opt_present("remember-session") || self.config.remember.last_session;
    let has_remember_user_session = self.config().opt_present("remember-user-session") || self.config.remember.last_user_session;

    if has_remember_session && has_remember_user_session {
      return Err("Only one of --remember-session and --remember-user-session may be used at the same time".into());
    }
    if has_remember_user_session && !has_remember {
      return Err("--remember-session must be used with --remember".into());
    }

    self.remember = has_remember;
    self.remember_session = has_remember_session;
    self.remember_user_session = has_remember_user_session;

    Ok(())
  }

  pub fn parse_power(&mut self) {
    self.powers.options.push(Power {
      action: PowerOption::Shutdown,
      label: fl!("shutdown"),
      command: self.config().opt_str("power-shutdown").or_else(|| self.config.defaults.shutdown_command.clone()),
    });

    self.powers.options.push(Power {
      action: PowerOption::Reboot,
      label: fl!("reboot"),
      command: self.config().opt_str("power-reboot").or_else(|| self.config.defaults.reboot_command.clone()),
    });

    self.power_setsid = !(self.config().opt_present("power-no-setsid") || self.config.defaults.power_no_setsid);
  }

  pub fn parse_keybinds(&mut self) -> Result<(), Box<dyn Error>> {
    self.kb_command = self.config().opt_str("kb-command").and_then(|i| i.parse::<u8>().ok()).or(self.config.ui.command_f_key).unwrap_or(2);
    self.kb_sessions = self.config().opt_str("kb-sessions").and_then(|i| i.parse::<u8>().ok()).or(self.config.ui.sessions_f_key).unwrap_or(3);
    self.kb_power = self.config().opt_str("kb-power").and_then(|i| i.parse::<u8>().ok()).or(self.config.ui.power_f_key).unwrap_or(12);

    if self.kb_command == self.kb_sessions || self.kb_sessions == self.kb_power || self.kb_power == self.kb_command {
      return Err("keybindings must all be distinct".into());
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{config::file::FileConfig, ui::sessions::SessionSource, Greeter, SecretDisplay};

  #[tokio::test]
  async fn test_command_line_arguments() {
    let table: &[(&[&str], _, Option<fn(&Greeter)>)] = &[
      // No arguments
      (&[], true, None),
      // Valid combinations
      (&["--cmd", "hello"], true, None),
      (
        &[
          "--cmd",
          "uname",
          "--env",
          "A=B",
          "--env",
          "C=D=E",
          "--asterisks",
          "--asterisks-char",
          ".",
          "--issue",
          "--time",
          "--prompt-padding",
          "0",
          "--window-padding",
          "1",
          "--container-padding",
          "12",
          "--user-menu",
        ],
        true,
        Some(|greeter| {
          assert!(matches!(&greeter.session_source, SessionSource::DefaultCommand(cmd, Some(env)) if cmd == "uname" && env.len() == 2));

          if let SessionSource::DefaultCommand(_, Some(env)) = &greeter.session_source {
            assert_eq!(env[0], "A=B");
            assert_eq!(env[1], "C=D=E");
          }

          assert!(matches!(&greeter.secret_display, SecretDisplay::Character(c) if c == "."));
          assert_eq!(greeter.prompt_padding(), 0);
          assert_eq!(greeter.window_padding(), 1);
          assert_eq!(greeter.container_padding(), 13);
          assert_eq!(greeter.user_menu, true);
          assert!(matches!(greeter.xsession_wrapper.as_deref(), Some("startx /usr/bin/env")));
        }),
      ),
      (
        &["--xsession-wrapper", "mywrapper.sh"],
        true,
        Some(|greeter| {
          assert!(matches!(greeter.xsession_wrapper.as_deref(), Some("mywrapper.sh")));
        }),
      ),
      (
        &["--no-xsession-wrapper"],
        true,
        Some(|greeter| {
          assert!(matches!(greeter.xsession_wrapper, None));
        }),
      ),
      // Invalid combinations
      (&["--remember-session", "--remember-user-session"], false, None),
      (&["--asterisk-char", ""], false, None),
      (&["--remember-user-session"], false, None),
      (&["--min-uid", "10000", "--max-uid", "5000"], false, None),
      (&["--issue", "--greeting", "Hello, world!"], false, None),
      (&["--kb-command", "2", "--kb-sessions", "2"], false, None),
      (&["--time-format", "%i %"], false, None),
      (&["--cmd", "cmd", "--env"], false, None),
      (&["--cmd", "cmd", "--env", "A"], false, None),
    ];

    for (args, valid, check) in table {
      let mut greeter = Greeter::default();
      let opts = greeter.parse_opts(*args);

      let result = match opts {
        Ok(opts) => {
          greeter.opts = opts;
          greeter.parse_config().await.ok()
        }

        Err(_) => None,
      };

      match valid {
        true => {
          assert!(result.is_some(), "{:?} cannot be parsed", args);
          assert!(matches!(greeter.parse_config().await, Ok(())), "{:?} cannot be parsed", greeter.opts);

          if let Some(check) = check {
            check(&greeter);
          }
        }

        false => assert!(result.is_none(), "{:?} should not have been parsed", args),
      }
    }
  }

  #[tokio::test]
  async fn command_and_env() {
    let table: &[(&[&str], fn(&mut FileConfig), fn(&Greeter))] = &[
      (
        &["--cmd", "mycommand", "--env", "A=b", "--env", "C=d"],
        |file: &mut FileConfig| {
          file.defaults.command = Some("secondcommand".to_string());
          file.defaults.env = Some(vec!["X=y".to_string()]);
        },
        |greeter| {
          assert!(matches!(&greeter.session_source, SessionSource::DefaultCommand(cmd, Some(env)) if cmd == "mycommand" && env.len() == 2));
        },
      ),
      (
        &[],
        |file: &mut FileConfig| {
          file.defaults.command = Some("secondcommand".to_string());
          file.defaults.env = Some(vec!["X=y".to_string()]);
        },
        |greeter| {
          assert!(matches!(&greeter.session_source, SessionSource::DefaultCommand(cmd, Some(env)) if cmd == "secondcommand" && env.len() == 1 && env.first().unwrap() == "X=y"));
        },
      ),
    ];

    for (opts, file, check) in table {
      let mut greeter = Greeter::default();
      greeter.opts = greeter.parse_opts(*opts).unwrap();
      greeter.config = FileConfig::default();

      file(&mut greeter.config);

      assert!(matches!(greeter.parse_config().await, Ok(())), "{:?} cannot be parsed", opts);

      check(&greeter);
    }
  }
}
