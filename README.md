# tuigreet

Graphical console greeter for [greetd](https://git.sr.ht/~kennylevinsen/greetd).

![Screenshot of tuigreet](https://github.com/apognu/tuigreet/blob/master/contrib/screenshot.png)

```
Usage: tuigreet [OPTIONS]

Options:
    -h, --help          show this usage information
    -v, --version       print version information
    -c, --cmd COMMAND   command to run
    -s, --sessions DIRS colon-separated list of session paths
    -w, --width WIDTH   width of the main prompt (default: 80)
    -i, --issue         show the host's issue file
    -g, --greeting GREETING
                        show custom text above login prompt
    -t, --time          display the current date and time
    -r, --remember      remember last logged-in username
        --asterisks     display asterisks when a secret is typed
        --asterisks-char CHAR
                        character to be used to redact secrets (default: *)
        --window-padding PADDING
                        padding inside the terminal area (default: 0)
        --container-padding PADDING
                        padding inside the main prompt container (default: 1)
        --prompt-padding PADDING
                        padding between prompt rows (default: 1)
```

## Usage

The default configuration tends to be as minimal as possible, visually speaking, only showing the authentication prompts and some minor information in the status bar. You may print your system's `/etc/issue` at the top of the prompt with `--issue` and the current date and time with `--time`. You may include a custom one-line greeting message instead of `/etc/issue` with `--greeting`.

The initial prompt container will be 80 column wide. You may change this with `--width` in case you need more space (for example, to account for large PAM challenge messages). Please refer to usage information (`--help`) for more customizaton options.

You may change the command that will be executed after opening a session by hitting `F2` and amending the command. Alternatively, you can list the system-declared sessions (or custom ones) by hitting `F3`.

## Install

### From source

Building from source requires an installation of Rust's `stable` toolchain, including `cargo`.

```
$ git clone https://github.com/apognu/tuigreet && cd tuigreet
$ cargo build --release
# mv target/release/tuigreet /usr/local/bin/tuigreet
```

### From AUR

On ArchLinux, `tuigreeter` is available on [AUR](https://aur.archlinux.org/packages/greetd-tuigreet) and is installable through your preferred AUR helper:

```
$ yay -S greetd-tuigreet
```

Two more distributions are available: `greetd-tuigreet-bin` is the precompiled release for the latest tagged release of `tuigreet` and `greetd-tuigreet-git` is a rolling release always following the `master` branch of this repository.

### From Gentoo

On Gentoo, `tuigreet` is available as a package `gui-apps/tuigreet`:

```
$ emerge --ask --verbose gui-apps/tuigreet
```


### Pre-built binaries

Pre-built binaries of `tuigreet` for several architectures can be found in the [releases](https://github.com/apognu/tuigreet/releases) section of this repository. The [tip prerelease](https://github.com/apognu/tuigreet/releases/tag/tip) is continuously built and kept in sync with the `master` branch.

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

### Sessions

The available sessions are fetched from `desktop` files in `/usr/share/xsessions` and `/usr/share/wayland-sessions`. If you want to provide custom directories, you can set the `--sessions` arguments with a colon-separated list of directories for `tuigreet` to fetch session definitions some other place.
