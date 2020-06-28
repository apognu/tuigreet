use std::{
  env,
  error::Error,
  fmt::{self, Display},
  os::unix::net::UnixStream,
  process,
};

use getopts::{Matches, Options};
use greetd_ipc::Request;

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

pub enum Mode {
  Username,
  Password,
}

impl Default for Mode {
  fn default() -> Mode {
    Mode::Username
  }
}

pub struct Greeter {
  pub config: Option<Matches>,
  pub stream: UnixStream,
  pub command: Option<String>,
  pub mode: Mode,
  pub request: Option<Request>,
  pub cursor_offset: i16,
  pub username: String,
  pub answer: String,
  pub secret: bool,
  pub prompt: String,
  pub greeting: Option<String>,
  pub message: Option<String>,
  pub working: bool,
  pub done: bool,
}

impl Greeter {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let stream = UnixStream::connect(env::var("GREETD_SOCK")?)?;

    Ok(Self {
      stream,
      config: Default::default(),
      command: Default::default(),
      mode: Default::default(),
      request: Default::default(),
      cursor_offset: Default::default(),
      username: Default::default(),
      answer: Default::default(),
      secret: Default::default(),
      prompt: Default::default(),
      greeting: Default::default(),
      message: Default::default(),
      working: Default::default(),
      done: Default::default(),
    })
  }

  pub fn config<'g>(&'g self) -> &'g Matches {
    self.config.as_ref().unwrap()
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
}

pub fn parse_options(greeter: Greeter) -> Greeter {
  let mut greeter = greeter;
  let mut opts = Options::new();

  opts.optflag("h", "help", "show this usage information");
  opts.optopt("c", "cmd", "command to run", "COMMAND");
  opts.optopt("", "width", "width of the main prompt (default: 80)", "WIDTH");
  opts.optflag("i", "issue", "show the host's issue file");
  opts.optopt("g", "greeting", "show custom text above login prompt", "GREETING");
  opts.optflag("t", "time", "display the current date and time");

  greeter.config = match opts.parse(&env::args().collect::<Vec<String>>()) {
    Ok(matches) => Some(matches),

    Err(usage) => {
      println!("{}", usage);
      print_usage(opts);
      process::exit(1);
    }
  };

  if greeter.config().opt_present("help") {
    print_usage(opts);
    std::process::exit(0);
  }

  if greeter.config().opt_present("issue") && greeter.config().opt_present("greeting") {
    eprintln!("Only one of --issue and --greeting may be used at the same time");
    print_usage(opts);
    std::process::exit(0);
  }

  greeter.greeting = greeter.option("greeting");
  greeter.command = greeter.option("cmd");

  if greeter.config().opt_present("issue") {
    greeter.greeting = get_issue();
  }

  greeter
}

fn print_usage(opts: Options) {
  eprint!("{}", opts.usage("Usage: greetd-tui [OPTIONS]"));
}
