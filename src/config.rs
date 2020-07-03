use std::{
  env,
  error::Error,
  fmt::{self, Display},
  os::unix::net::UnixStream,
  process,
};

use getopts::{Matches, Options};
use greetd_ipc::Request;
use zeroize::Zeroize;

use crate::info::get_issue;

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
}

impl Default for Mode {
  fn default() -> Mode {
    Mode::Username
  }
}

#[derive(Default)]
pub struct Greeter {
  pub config: Option<Matches>,
  pub stream: Option<UnixStream>,
  pub request: Option<Request>,

  pub mode: Mode,
  pub previous_mode: Mode,
  pub cursor_offset: i16,

  pub command: Option<String>,
  pub new_command: String,
  pub sessions: Vec<(String, String)>,
  pub selected_session: usize,

  pub username: String,
  pub prompt: String,
  pub answer: String,
  pub secret: bool,

  pub greeting: Option<String>,
  pub message: Option<String>,

  pub working: bool,
  pub done: bool,
}

impl Drop for Greeter {
  fn drop(self: &mut Self) {
    self.prompt.zeroize();
    self.username.zeroize();
    self.answer.zeroize();
    self.message.zeroize();
  }
}

impl Greeter {
  pub fn new() -> Self {
    let mut greeter = Self::default();

    greeter.parse_options();
    greeter.sessions = crate::info::get_sessions().unwrap_or_default();
    greeter.selected_session = greeter.sessions.iter().position(|(_, command)| Some(command) == greeter.command.as_ref()).unwrap_or(0);
    greeter
  }

  pub fn config(&self) -> &Matches {
    self.config.as_ref().unwrap()
  }

  pub fn stream(&self) -> &UnixStream {
    self.stream.as_ref().unwrap()
  }

  pub fn option(&self, name: &str) -> Option<String> {
    match self.config().opt_str(name) {
      Some(value) => Some(value),
      None => None,
    }
  }

  pub fn width(&self) -> u16 {
    if let Some(value) = self.option("width") {
      if let Ok(width) = value.parse::<u16>() {
        return width;
      }
    }

    80
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

  pub fn parse_options(&mut self) {
    let mut opts = Options::new();

    opts.optflag("h", "help", "show this usage information");
    opts.optflag("v", "version", "print version information");
    opts.optopt("c", "cmd", "command to run", "COMMAND");
    opts.optopt("w", "width", "width of the main prompt (default: 80)", "WIDTH");
    opts.optflag("i", "issue", "show the host's issue file");
    opts.optopt("g", "greeting", "show custom text above login prompt", "GREETING");
    opts.optflag("t", "time", "display the current date and time");
    opts.optopt("", "container-padding", "padding inside the main prompt container (default: 1)", "PADDING");
    opts.optopt("", "prompt-padding", "padding between prompt rows (default: 1)", "PADDING");

    self.config = match opts.parse(&env::args().collect::<Vec<String>>()) {
      Ok(matches) => Some(matches),

      Err(usage) => {
        println!("{}", usage);
        print_usage(opts);
        process::exit(1);
      }
    };

    if self.config().opt_present("help") {
      print_usage(opts);
      std::process::exit(0);
    }
    if self.config().opt_present("version") {
      print_version();
      std::process::exit(0);
    }

    let socket = env::var("GREETD_SOCK");
    if socket.is_err() {
      eprintln!("GREETD_SOCK must be defined");
      process::exit(1);
    }

    match UnixStream::connect(socket.unwrap()) {
      Ok(stream) => self.stream = Some(stream),

      Err(err) => {
        eprintln!("{}", err);
        process::exit(1);
      }
    }

    if self.config().opt_present("issue") && self.config().opt_present("greeting") {
      eprintln!("Only one of --issue and --greeting may be used at the same time");
      print_usage(opts);
      std::process::exit(0);
    }

    self.greeting = self.option("greeting");
    self.command = self.option("cmd");

    if self.config().opt_present("issue") {
      self.greeting = get_issue();
    }
  }
}

fn print_usage(opts: Options) {
  eprint!("{}", opts.usage("Usage: tuigreet [OPTIONS]"));
}

fn print_version() {
  println!("tuigreet {} ({})", env!("VERSION"), env!("TARGET"));
  println!("Copyright (C) 2020 Antoine POPINEAU <https://github.com/apognu/tuigreet>.");
  println!("Licensed under GPLv3 (GNU GPL version 3 or later).");
}
