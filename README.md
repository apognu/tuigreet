# tuigreet

Graphical console greeter for [greetd](https://git.sr.ht/~kennylevinsen/greetd).

![Screenshot of tuigreet](https://github.com/apognu/tuigreet/blob/master/contrib/screenshot.png)

```
Usage: tuigreet [OPTIONS]

Options:
    -h, --help          show this usage information
    -c, --cmd COMMAND   command to run
    -w, --width WIDTH   width of the main prompt (default: 80)
    -i, --issue         show the host's issue file
    -g, --greeting GREETING
                        show custom text above login prompt
    -t, --time          display the current date and time
```

## Usage

The default configuration tends to be as minimal as possible, visually speaking, only showing the authentication prompts and some minor information in the status bar. You may print your system's `/etc/issue` at the top of the prompt with `--issue` and the current date and time with `--time`. You may include a custom one-line greeting message instead of `/etc/issue` with `--greeting`.

The initial prompt container will be 80 column wide. You may change this with `--width` in case you need more space (for example, to account for large PAM challenge messages).

You may change the command that will be executed after opening a session by prefixing it with `!` in the username input box, for example `!bash`.

## Install

### From source

Building from source requires an installation of Rust's `stable` toolchain, including `cargo`.

```
$ git clone https://github.com/apognu/tuigreet && cd tuigreet
$ cargo build --release
# mv target/release/tuigreet /usr/local/bin/tuigreet
```

### Pre-built binaries

Pre-built binaries of `tuigreet` for several architectures can be found in the [releases](https://github.com/apognu/tuigreet/releases) section of this repository. The [tip prerelease](https://github.com/apognu/tuigreet/releases/tag/tip) is continuously built and kept in sync with the `master` branch.

Actual tag releases will be created when the project stabilizes.

## Configuration

Edit `/etc/greetd/config.toml` and set the `command` setting to use `tuigreet`:

```
[terminal]
vt = 1

[default_session]
command = "tuigreet --cmd sway"
user = "greeter"
```

Please refer to [greetd's wiki](https://man.sr.ht/~kennylevinsen/greetd/) for more information on setting up `greetd`.
