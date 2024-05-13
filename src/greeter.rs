use std::{
  convert::TryInto,
  env,
  error::Error,
  fmt::{self, Display},
  path::PathBuf,
  process,
  sync::Arc,
};

use chrono::{
  format::{Item, StrftimeItems},
  Locale,
};
use getopts::{Matches, Options};
use i18n_embed::DesktopLanguageRequester;
use tokio::{
  net::UnixStream,
  sync::{mpsc::Sender, RwLock, RwLockWriteGuard},
};
use zeroize::Zeroize;

use crate::{
  event::Event,
  info::{
    get_issue, get_last_session, get_last_session_path, get_last_user_name, get_last_user_session, get_last_user_session_path, get_last_user_username, get_min_max_uids, get_sessions, get_users,
  },
  power::PowerOption,
  ui::{
    common::{masked::MaskedString, menu::Menu, style::Theme},
    power::Power,
    sessions::{Session, SessionSource, SessionType},
    users::User,
  },
};

const DEFAULT_LOG_FILE: &str = "/tmp/tuigreet.log";
const DEFAULT_LOCALE: Locale = Locale::en_US;
const DEFAULT_ASTERISKS_CHARS: &str = "*";
// `startx` wants an absolute path to the executable as a first argument.
// We don't want to resolve the session command in the greeter though, so it should be additionally wrapped with a known noop command (like `/usr/bin/env`).
const DEFAULT_XSESSION_WRAPPER: &str = "startx /usr/bin/env";

#[derive(Debug, Copy, Clone)]
pub enum AuthStatus {
  Success,
  Failure,
  Cancel,
}

impl Display for AuthStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Error for AuthStatus {}

// A mode represents the large section of the software, usually screens to be
// displayed, or the state of the application.
#[derive(SmartDefault, Debug, Copy, Clone, PartialEq)]
pub enum Mode {
  #[default]
  Username,
  Password,
  Action,
  Users,
  Command,
  Sessions,
  Power,
  Processing,
}

// This enum models how secret values should be displayed on terminal.
#[derive(SmartDefault, Debug, Clone)]
pub enum SecretDisplay {
  #[default]
  // All characters hidden.
  Hidden,
  // All characters are replaced by a placeholder character.
  Character(String),
}

impl SecretDisplay {
  pub fn show(&self) -> bool {
    match self {
      SecretDisplay::Hidden => false,
      SecretDisplay::Character(_) => true,
    }
  }
}

// This enum models text alignment options
#[derive(SmartDefault, Debug, Clone)]
pub enum TextAlign {
  #[default]
  Center,
  Left,
  Right
}

#[derive(SmartDefault)]
pub struct Greeter {
  pub debug: bool,
  pub logfile: String,

  #[default(DEFAULT_LOCALE)]
  pub locale: Locale,
  pub config: Option<Matches>,
  pub socket: String,
  pub stream: Option<Arc<RwLock<UnixStream>>>,
  pub events: Option<Sender<Event>>,

  // Current mode of the application, will define what actions are permitted.
  pub mode: Mode,
  // Mode the application will return to when exiting the current mode.
  pub previous_mode: Mode,
  // Offset the cursor should be at from its base position for the current mode.
  pub cursor_offset: i16,

  // Buffer to be used as a temporary editing zone for the various modes.
  // Previous buffer is saved when a transient screen has to use the buffer, to
  // be able to restore it when leaving the transient screen.
  pub previous_buffer: Option<String>,
  pub buffer: String,

  // Define the selected session and how to resolve it.
  pub session_source: SessionSource,
  // List of session files found on disk.
  pub session_paths: Vec<(PathBuf, SessionType)>,
  // Menu for session selection.
  pub sessions: Menu<Session>,
  // Wrapper command to prepend to non-X11 sessions.
  pub session_wrapper: Option<String>,
  // Wrapper command to prepend to X11 sessions.
  pub xsession_wrapper: Option<String>,

  // Whether user menu is enabled.
  pub user_menu: bool,
  // Menu for user selection.
  pub users: Menu<User>,
  // Current username. Masked to display the full name if available.
  pub username: MaskedString,
  // Prompt that should be displayed to ask for entry.
  pub prompt: Option<String>,

  // Whether the current edition prompt should be hidden.
  pub asking_for_secret: bool,
  // How should secrets be displayed?
  pub secret_display: SecretDisplay,

  // Whether last logged-in user should be remembered.
  pub remember: bool,
  // Whether last launched session (regardless of user) should be remembered.
  pub remember_session: bool,
  // Whether last launched session for the current user should be remembered.
  pub remember_user_session: bool,

  // Style object for the terminal UI
  pub theme: Theme,
  // Greeting message (MOTD) to use to welcome the user.
  pub greeting: Option<String>,
  // Transaction message to show to the user.
  pub message: Option<String>,

  // Menu for power options.
  pub powers: Menu<Power>,
  // Whether to prefix the power commands with `setsid`.
  pub power_setsid: bool,

  #[default(2)]
  pub kb_command: u8,
  #[default(3)]
  pub kb_sessions: u8,
  #[default(12)]
  pub kb_power: u8,

  // The software is waiting for a response from `greetd`.
  pub working: bool,
  // We are done working.
  pub done: bool,
  // Should we exit?
  pub exit: Option<AuthStatus>,
}

impl Drop for Greeter {
  fn drop(&mut self) {
    self.scrub(true, false);
  }
}

impl Greeter {
  pub async fn new(events: Sender<Event>) -> Self {
    let mut greeter = Self::default();

    greeter.events = Some(events);
    greeter.set_locale();

    greeter.powers = Menu {
      title: fl!("title_power"),
      options: Default::default(),
      selected: 0,
    };

    greeter.parse_options().await;

    greeter.sessions = Menu {
      title: fl!("title_session"),
      options: get_sessions(&greeter).unwrap_or_default(),
      selected: 0,
    };

    // If we should remember the last logged-in user.
    if greeter.remember {
      if let Some(username) = get_last_user_username() {
        greeter.username = MaskedString::from(username, get_last_user_name());

        // If, on top of that, we should remember their last session.
        if greeter.remember_user_session {
          if let Ok(ref session_path) = get_last_user_session_path(greeter.username.get()) {
            // Set the selected menu option and the session source.
            greeter.sessions.selected = greeter.sessions.options.iter().position(|Session { path, .. }| path.as_deref() == Some(session_path)).unwrap_or(0);
            greeter.session_source = SessionSource::Session(greeter.sessions.selected);
          }

          // See if we have the last free-form command from the user.
          if let Ok(command) = get_last_user_session(greeter.username.get()) {
            greeter.session_source = SessionSource::Command(command);
          }
        }
      }
    }

    // Same thing, but not user specific.
    if greeter.remember_session {
      if let Ok(ref session_path) = get_last_session_path() {
        greeter.sessions.selected = greeter.sessions.options.iter().position(|Session { path, .. }| path.as_deref() == Some(session_path)).unwrap_or(0);
        greeter.session_source = SessionSource::Session(greeter.sessions.selected);
      }

      if let Ok(command) = get_last_session() {
        greeter.session_source = SessionSource::Command(command.trim().to_string());
      }
    }

    greeter
  }

  // Scrub memory of all data, unless `soft` is true, in which case, we will
  // keep the username (can happen if a wrong password was entered, we want to
  // give the user another chance, as PAM would).
  fn scrub(&mut self, scrub_message: bool, soft: bool) {
    self.buffer.zeroize();
    self.prompt.zeroize();

    if !soft {
      self.username.zeroize();
    }

    if scrub_message {
      self.message.zeroize();
    }
  }

  // Reset the software to its initial state.
  pub async fn reset(&mut self, soft: bool) {
    if soft {
      self.mode = Mode::Password;
      self.previous_mode = Mode::Password;
    } else {
      self.mode = Mode::Username;
      self.previous_mode = Mode::Username;
    }

    self.working = false;
    self.done = false;

    self.scrub(false, soft);
    self.connect().await;
  }

  // Connect to `greetd` and return a stream we can safely write to.
  pub async fn connect(&mut self) {
    match UnixStream::connect(&self.socket).await {
      Ok(stream) => self.stream = Some(Arc::new(RwLock::new(stream))),

      Err(err) => {
        eprintln!("{err}");
        process::exit(1);
      }
    }
  }

  pub fn config(&self) -> &Matches {
    self.config.as_ref().unwrap()
  }

  pub async fn stream(&self) -> RwLockWriteGuard<'_, UnixStream> {
    self.stream.as_ref().unwrap().write().await
  }

  pub fn option(&self, name: &str) -> Option<String> {
    self.config().opt_str(name)
  }

  // Returns the width of the main window where content is displayed from the
  // provided arguments.
  pub fn width(&self) -> u16 {
    if let Some(value) = self.option("width") {
      if let Ok(width) = value.parse::<u16>() {
        return width;
      }
    }

    80
  }

  // Returns the padding of the screen from the provided arguments.
  pub fn window_padding(&self) -> u16 {
    if let Some(value) = self.option("window-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding;
      }
    }

    0
  }

  // Returns the padding of the main window where content is displayed from the
  // provided arguments.
  pub fn container_padding(&self) -> u16 {
    if let Some(value) = self.option("container-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding + 1;
      }
    }

    2
  }

  // Returns the spacing between each prompt from the provided arguments.
  pub fn prompt_padding(&self) -> u16 {
    if let Some(value) = self.option("prompt-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding;
      }
    }

    1
  }

  pub fn text_align(&self) -> TextAlign {
    if let Some(value) = self.option("text-align") {
      match value.to_uppercase().as_str() {
        "LEFT" | "L" => TextAlign::Left,
        "RIGHT" | "R" => TextAlign::Right,
        _ => TextAlign::Center
      }
    } else {
      TextAlign::default()
    }
  }

  // Sets the locale that will be used for this invocation from environment.
  fn set_locale(&mut self) {
    let locale = DesktopLanguageRequester::requested_languages()
      .into_iter()
      .next()
      .and_then(|locale| locale.region.map(|region| format!("{}_{region}", locale.language)))
      .and_then(|id| id.as_str().try_into().ok());

    if let Some(locale) = locale {
      self.locale = locale;
    }
  }

  pub fn options() -> Options {
    let mut opts = Options::new();

    let xsession_wrapper_desc = format!("wrapper command to initialize X server and launch X11 sessions (default: {DEFAULT_XSESSION_WRAPPER})");

    opts.optflag("h", "help", "show this usage information");
    opts.optflag("v", "version", "print version information");
    opts.optflagopt("d", "debug", "enable debug logging to the provided file, or to /tmp/tuigreet.log", "FILE");
    opts.optopt("c", "cmd", "command to run", "COMMAND");
    opts.optopt("s", "sessions", "colon-separated list of Wayland session paths", "DIRS");
    opts.optopt("", "session-wrapper", "wrapper command to initialize the non-X11 session", "'CMD [ARGS]...'");
    opts.optopt("x", "xsessions", "colon-separated list of X11 session paths", "DIRS");
    opts.optopt("", "xsession-wrapper", xsession_wrapper_desc.as_str(), "'CMD [ARGS]...'");
    opts.optflag("", "no-xsession-wrapper", "do not wrap commands for X11 sessions");
    opts.optopt("w", "width", "width of the main prompt (default: 80)", "WIDTH");
    opts.optflag("i", "issue", "show the host's issue file");
    opts.optopt("g", "greeting", "show custom text above login prompt", "GREETING");
    opts.optflag("t", "time", "display the current date and time");
    opts.optopt("", "time-format", "custom strftime format for displaying date and time", "FORMAT");
    opts.optflag("r", "remember", "remember last logged-in username");
    opts.optflag("", "remember-session", "remember last selected session");
    opts.optflag("", "remember-user-session", "remember last selected session for each user");
    opts.optflag("", "user-menu", "allow graphical selection of users from a menu");
    opts.optopt("", "user-menu-min-uid", "minimum UID to display in the user selection menu", "UID");
    opts.optopt("", "user-menu-max-uid", "maximum UID to display in the user selection menu", "UID");
    opts.optopt("", "theme", "define the application theme colors", "THEME");
    opts.optflag("", "asterisks", "display asterisks when a secret is typed");
    opts.optopt("", "asterisks-char", "characters to be used to redact secrets (default: *)", "CHARS");
    opts.optopt("", "window-padding", "padding inside the terminal area (default: 0)", "PADDING");
    opts.optopt("", "container-padding", "padding inside the main prompt container (default: 1)", "PADDING");
    opts.optopt("", "prompt-padding", "padding between prompt rows (default: 1)", "PADDING");
    opts.optopt("", "text-align", "alignment of text in the prompt container (default: 'center')", "[left|center|right]");

    opts.optopt("", "power-shutdown", "command to run to shut down the system", "'CMD [ARGS]...'");
    opts.optopt("", "power-reboot", "command to run to reboot the system", "'CMD [ARGS]...'");
    opts.optflag("", "power-no-setsid", "do not prefix power commands with setsid");

    opts.optopt("", "kb-command", "F-key to use to open the command menu", "[1-12]");
    opts.optopt("", "kb-sessions", "F-key to use to open the sessions menu", "[1-12]");
    opts.optopt("", "kb-power", "F-key to use to open the power menu", "[1-12]");

    opts
  }

  // Parses command line arguments to configured the software accordingly.
  pub async fn parse_options(&mut self) {
    let opts = Greeter::options();

    self.config = match opts.parse(env::args().collect::<Vec<String>>()) {
      Ok(matches) => Some(matches),

      Err(err) => {
        eprintln!("{err}");
        print_usage(opts);
        process::exit(1);
      }
    };

    if self.config().opt_present("help") {
      print_usage(opts);
      process::exit(0);
    }
    if self.config().opt_present("version") {
      print_version();
      process::exit(0);
    }

    match env::var("GREETD_SOCK") {
      Ok(socket) => self.socket = socket,
      Err(_) => {
        eprintln!("GREETD_SOCK must be defined");
        process::exit(1);
      }
    }

    if self.config().opt_present("debug") {
      self.debug = true;

      self.logfile = match self.config().opt_str("debug") {
        Some(file) => file.to_string(),
        None => DEFAULT_LOG_FILE.to_string(),
      }
    }

    if self.config().opt_present("issue") && self.config().opt_present("greeting") {
      eprintln!("Only one of --issue and --greeting may be used at the same time");
      print_usage(opts);
      process::exit(1);
    }

    if self.config().opt_present("theme") {
      if let Some(spec) = self.config().opt_str("theme") {
        self.theme = Theme::parse(spec.as_str());
      }
    }

    if self.config().opt_present("asterisks") {
      let asterisk = if let Some(value) = self.config().opt_str("asterisks-char") {
        if value.chars().count() < 1 {
          eprintln!("--asterisks-char must have at least one character as its value");
          print_usage(opts);
          process::exit(1);
        }

        value
      } else {
        DEFAULT_ASTERISKS_CHARS.to_string()
      };

      self.secret_display = SecretDisplay::Character(asterisk);
    }

    if let Some(format) = self.config().opt_str("time-format") {
      if StrftimeItems::new(&format).any(|item| item == Item::Error) {
        eprintln!("Invalid strftime format provided in --time-format");
        process::exit(1);
      }
    }

    if self.config().opt_present("user-menu") {
      self.user_menu = true;

      let min_uid = self.config().opt_str("user-menu-min-uid").and_then(|uid| uid.parse::<u16>().ok());
      let max_uid = self.config().opt_str("user-menu-max-uid").and_then(|uid| uid.parse::<u16>().ok());
      let (min_uid, max_uid) = get_min_max_uids(min_uid, max_uid);

      tracing::info!("min/max UIDs are {}/{}", min_uid, max_uid);

      if min_uid >= max_uid {
        eprintln!("Minimum UID ({min_uid}) must be less than maximum UID ({max_uid})");
        process::exit(1);
      }

      self.users = Menu {
        title: fl!("title_users"),
        options: get_users(min_uid, max_uid),
        selected: 0,
      };

      tracing::info!("found {} users", self.users.options.len());
    }

    if self.config().opt_present("remember-session") && self.config().opt_present("remember-user-session") {
      eprintln!("Only one of --remember-session and --remember-user-session may be used at the same time");
      print_usage(opts);
      process::exit(1);
    }
    if self.config().opt_present("remember-user-session") && !self.config().opt_present("remember") {
      eprintln!("--remember-session must be used with --remember");
      print_usage(opts);
      process::exit(1);
    }

    self.remember = self.config().opt_present("remember");
    self.remember_session = self.config().opt_present("remember-session");
    self.remember_user_session = self.config().opt_present("remember-user-session");
    self.greeting = self.option("greeting");

    // If the `--cmd` argument is provided, it will override the selected session.
    if let Some(command) = self.option("cmd") {
      self.session_source = SessionSource::Command(command);
    }

    if let Some(dirs) = self.option("sessions") {
      self.session_paths.extend(env::split_paths(&dirs).map(|dir| (dir, SessionType::Wayland)));
    }

    if let Some(dirs) = self.option("xsessions") {
      self.session_paths.extend(env::split_paths(&dirs).map(|dir| (dir, SessionType::X11)));
    }

    if self.option("session-wrapper").is_some() {
      self.session_wrapper = self.option("session-wrapper");
    }

    if !self.config().opt_present("no-xsession-wrapper") {
      self.xsession_wrapper = self.option("xsession-wrapper").or_else(|| Some(DEFAULT_XSESSION_WRAPPER.to_string()));
    }

    if self.config().opt_present("issue") {
      self.greeting = get_issue();
    }

    self.powers.options.push(Power {
      action: PowerOption::Shutdown,
      label: fl!("shutdown"),
      command: self.config().opt_str("power-shutdown"),
    });

    self.powers.options.push(Power {
      action: PowerOption::Reboot,
      label: fl!("reboot"),
      command: self.config().opt_str("power-reboot"),
    });

    self.power_setsid = !self.config().opt_present("power-no-setsid");

    self.kb_command = self.config().opt_str("kb-command").map(|i| i.parse::<u8>().unwrap_or_default()).unwrap_or(2);
    self.kb_sessions = self.config().opt_str("kb-sessions").map(|i| i.parse::<u8>().unwrap_or_default()).unwrap_or(3);
    self.kb_power = self.config().opt_str("kb-power").map(|i| i.parse::<u8>().unwrap_or_default()).unwrap_or(12);

    if self.kb_command == self.kb_sessions || self.kb_sessions == self.kb_power || self.kb_power == self.kb_command {
      eprintln!("keybindings must all be distinct");
      print_usage(opts);
      process::exit(1);
    }

    self.connect().await;
  }

  pub fn set_prompt(&mut self, prompt: &str) {
    self.prompt = if prompt.ends_with(' ') { Some(prompt.into()) } else { Some(format!("{prompt} ")) };
  }

  pub fn remove_prompt(&mut self) {
    self.prompt = None;
  }

  // Computes the size of the prompt to help determine where input should start.
  pub fn prompt_width(&self) -> usize {
    match &self.prompt {
      None => 0,
      Some(prompt) => prompt.chars().count(),
    }
  }
}

fn print_usage(opts: Options) {
  eprint!("{}", opts.usage("Usage: tuigreet [OPTIONS]"));
}

fn print_version() {
  println!("tuigreet {} ({})", env!("VERSION"), env!("TARGET"));
  println!("Copyright (C) 2020 Antoine POPINEAU <https://github.com/apognu/tuigreet>.");
  println!("Licensed under GPLv3+ (GNU GPL version 3 or later).");
  println!();
  println!("This is free software, you are welcome to redistribute it under some conditions.");
  println!("There is NO WARRANTY, to the extent provided by law.");
}

#[cfg(test)]
mod test {
  use crate::Greeter;

  #[test]
  fn test_prompt_width() {
    let mut greeter = Greeter::default();
    greeter.prompt = None;

    assert_eq!(greeter.prompt_width(), 0);

    greeter.prompt = Some("Hello:".into());

    assert_eq!(greeter.prompt_width(), 6);
  }

  #[test]
  fn test_set_prompt() {
    let mut greeter = Greeter::default();

    greeter.set_prompt("Hello:");

    assert_eq!(greeter.prompt, Some("Hello: ".into()));

    greeter.set_prompt("Hello World: ");

    assert_eq!(greeter.prompt, Some("Hello World: ".into()));

    greeter.remove_prompt();

    assert_eq!(greeter.prompt, None);
  }
}
