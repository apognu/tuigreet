use std::{
  convert::TryInto,
  env,
  error::Error,
  fmt::{self, Display},
  os::unix::net::UnixStream,
  process,
};

use chrono::Locale;
use getopts::{Matches, Options};
use greetd_ipc::Request;
use i18n_embed::DesktopLanguageRequester;
use zeroize::Zeroize;

use crate::info::{get_issue, get_last_username};

pub const DEFAULT_LOCALE: Locale = Locale::en_US;

#[derive(Debug)]
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

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
  Username,
  Password,
  Command,
  Sessions,
  Power,
}

impl Default for Mode {
  fn default() -> Mode {
    Mode::Username
  }
}

#[derive(Default)]
pub struct Greeter {
  pub locale: Option<Locale>,
  pub config: Option<Matches>,
  pub socket: String,
  pub stream: Option<UnixStream>,
  pub request: Option<Request>,

  pub mode: Mode,
  pub previous_mode: Mode,
  pub cursor_offset: i16,

  pub command: Option<String>,
  pub new_command: String,
  pub sessions_path: Option<String>,
  pub sessions: Vec<(String, String)>,
  pub selected_session: usize,

  pub selected_power_option: usize,

  pub username: String,
  pub prompt: String,
  pub answer: String,
  pub secret: bool,

  pub remember: bool,
  pub asterisks: bool,
  pub greeting: Option<String>,
  pub message: Option<String>,

  pub working: bool,
  pub done: bool,
}

impl Drop for Greeter {
  fn drop(&mut self) {
    self.prompt.zeroize();
    self.username.zeroize();
    self.answer.zeroize();
    self.message.zeroize();
  }
}

impl Greeter {
  pub fn new() -> Self {
    let mut greeter = Self::default();

    greeter.set_locale();
    greeter.parse_options();
    greeter.sessions = crate::info::get_sessions(&greeter).unwrap_or_default();
    greeter.selected_session = greeter.sessions.iter().position(|(_, command)| Some(command) == greeter.command.as_ref()).unwrap_or(0);

    if greeter.remember {
      greeter.username = get_last_username().unwrap_or_default().trim().to_string();
    }

    greeter
  }

  pub fn reset(&mut self) {
    self.mode = Mode::Username;
    self.previous_mode = Mode::Username;
    self.username = String::new();
    self.answer = String::new();
    self.working = false;
    self.done = false;

    self.connect();
  }

  pub fn connect(&mut self) {
    match UnixStream::connect(&self.socket) {
      Ok(stream) => self.stream = Some(stream),

      Err(err) => {
        eprintln!("{}", err);
        process::exit(1);
      }
    }
  }

  pub fn config(&self) -> &Matches {
    self.config.as_ref().unwrap()
  }

  pub fn stream(&self) -> &UnixStream {
    self.stream.as_ref().unwrap()
  }

  pub fn option(&self, name: &str) -> Option<String> {
    self.config().opt_str(name)
  }

  pub fn width(&self) -> u16 {
    if let Some(value) = self.option("width") {
      if let Ok(width) = value.parse::<u16>() {
        return width;
      }
    }

    80
  }

  pub fn window_padding(&self) -> u16 {
    if let Some(value) = self.option("window-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding;
      }
    }

    0
  }

  pub fn container_padding(&self) -> u16 {
    if let Some(value) = self.option("container-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding + 1;
      }
    }

    2
  }

  pub fn prompt_padding(&self) -> u16 {
    if let Some(value) = self.option("prompt-padding") {
      if let Ok(padding) = value.parse::<u16>() {
        return padding;
      }
    }

    1
  }

  fn set_locale(&mut self) {
    self.locale = DesktopLanguageRequester::requested_languages()
      .into_iter()
      .next()
      .and_then(|locale| locale.region.map(|region| format!("{}_{}", locale.language, region)))
      .and_then(|id| id.as_str().try_into().ok());
  }

  fn parse_options(&mut self) {
    let mut opts = Options::new();

    opts.optflag("h", "help", "show this usage information");
    opts.optflag("v", "version", "print version information");
    opts.optopt("c", "cmd", "command to run", "COMMAND");
    opts.optopt("s", "sessions", "colon-separated list of session paths", "DIRS");
    opts.optopt("w", "width", "width of the main prompt (default: 80)", "WIDTH");
    opts.optflag("i", "issue", "show the host's issue file");
    opts.optopt("g", "greeting", "show custom text above login prompt", "GREETING");
    opts.optflag("t", "time", "display the current date and time");
    opts.optflag("r", "remember", "remember last logged-in username");
    opts.optflag("", "asterisks", "display asterisks when a secret is typed");
    opts.optopt("", "window-padding", "padding inside the terminal area (default: 0)", "PADDING");
    opts.optopt("", "container-padding", "padding inside the main prompt container (default: 1)", "PADDING");
    opts.optopt("", "prompt-padding", "padding between prompt rows (default: 1)", "PADDING");

    self.config = match opts.parse(&env::args().collect::<Vec<String>>()) {
      Ok(matches) => Some(matches),

      Err(error) => {
        println!("{}", error);
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

    self.connect();

    if self.config().opt_present("issue") && self.config().opt_present("greeting") {
      eprintln!("Only one of --issue and --greeting may be used at the same time");
      print_usage(opts);
      process::exit(0);
    }

    self.remember = self.config().opt_present("remember");
    self.asterisks = self.config().opt_present("asterisks");
    self.greeting = self.option("greeting");
    self.command = self.option("cmd");
    self.sessions_path = self.option("sessions");

    if self.config().opt_present("issue") {
      self.greeting = get_issue();
    }
  }

  pub fn set_prompt(&mut self, prompt: &str) {
    self.prompt = if prompt.ends_with(' ') { prompt.into() } else { format!("{} ", prompt) };
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
