mod event;
mod info;
mod ipc;
mod keyboard;
mod ui;

use std::{env, error::Error, io, os::unix::net::UnixStream, process};

use getopts::{Matches, Options};
use greetd_ipc::{codec::SyncCodec, Request};
use termion::raw::IntoRawMode;
use tui::{backend::TermionBackend, Terminal};

use self::{event::Events, info::get_issue};

pub enum AuthStatus {
    Success,
    Failure,
}

pub enum Mode {
    Username,
    Password,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Username
    }
}

#[derive(Default)]
pub struct Greeter {
    config: Option<Matches>,
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
    pub fn config(&self) -> Matches {
        self.config.clone().unwrap()
    }

    pub fn width(&self) -> u16 {
        if let Some(value) = self.config().opt_str("width") {
            if let Ok(width) = value.parse::<u16>() {
                return width;
            }
        }

        80
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut greeter = Greeter::default();

    let mut opts = Options::new();
    opts.optflag("h", "help", "show this usage information");
    opts.optopt("c", "cmd", "command to run", "COMMAND");
    opts.optopt(
        "",
        "width",
        "width of the main prompt (default: 80)",
        "WIDTH",
    );
    opts.optflag("i", "issue", "show the host's issue file");
    opts.optopt(
        "g",
        "greeting",
        "show custom text above login prompt",
        "GREETING",
    );
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

    if greeter.config().opt_present("cmd") {
        greeter.command = greeter.config().opt_str("cmd");
    }

    let mut stream = UnixStream::connect(env::var("GREETD_SOCK")?)?;

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let events = Events::new();

    if greeter.config().opt_present("issue") {
        greeter.greeting = get_issue();
    }
    if greeter.config().opt_present("greeting") {
        greeter.greeting = greeter.config().opt_str("greeting");
    }

    loop {
        ui::draw(&mut terminal, &mut greeter)?;
        ipc::handle(&mut greeter, &mut stream)?;
        keyboard::handle(&mut greeter, &events, &mut stream)?;
    }
}

fn print_usage(opts: Options) {
    eprint!("{}", opts.usage("Usage: greetd-tui [OPTIONS]"));
}

pub fn exit(status: AuthStatus, stream: &mut UnixStream) {
    match status {
        AuthStatus::Success => process::exit(0),

        AuthStatus::Failure => {
            Request::CancelSession.write_to(stream).unwrap();
            process::exit(1);
        }
    }
}
