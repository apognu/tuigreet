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
        --time-format FORMAT
                        custom strftime format for displaying date and time
    -r, --remember      remember last logged-in username
        --remember-session
                        remember last selected session
        --remember-user-session
                        remember last selected session for each user
        --user-menu     allow graphical selection of users from a menu
        --user-menu-min-uid UID
                        minimum UID to display in the user selection menu
        --user-menu-max-uid UID
                        maximum UID to display in the user selection menu
        --asterisks     display asterisks when a secret is typed
        --asterisks-char CHAR
                        character to be used to redact secrets (default: *)
        --window-padding PADDING
                        padding inside the terminal area (default: 0)
        --container-padding PADDING
                        padding inside the main prompt container (default: 1)
        --prompt-padding PADDING
                        padding between prompt rows (default: 1)
        --power-shutdown 'CMD [ARGS]...'
                        command to run to shut down the system
        --power-reboot 'CMD [ARGS]...'
                        command to run to reboot the system
        --power-no-setsid
                        do not prefix power commands with setsid
```

## Usage

The default configuration tends to be as minimal as possible, visually speaking, only showing the authentication prompts and some minor information in the status bar. You may print your system's `/etc/issue` at the top of the prompt with `--issue` and the current date and time with `--time` (and possibly customize it with `--time-format`). You may include a custom one-line greeting message instead of `/etc/issue` with `--greeting`.

The initial prompt container will be 80 column wide. You may change this with `--width` in case you need more space (for example, to account for large PAM challenge messages). Please refer to usage information (`--help`) for more customizaton options. Various padding settings are availble through the `*-padding` options.

You can instruct `tuigreet` to remember the last username that successfully opened a session with the `--remember` option (that way, the username field will be pre-filled). Similarly, the command and session configuration can be retained between runs with the `--remember-session` option (when using this, the `--cmd` value is overridden by manual selections). You can also remember the selected session per user with the `--remember-user-session` flag. In this case, the selected session will only be saved on successful authentication.

You may change the command that will be executed after opening a session by hitting `F2` and amending the command. Alternatively, you can list the system-declared sessions (or custom ones) by hitting `F3`. Power options are available through `F12`.

## Install

### From source

Building from source requires an installation of Rust's `stable` toolchain, including `cargo`.

```
$ git clone https://github.com/apognu/tuigreet && cd tuigreet
$ cargo build --release
# mv target/release/tuigreet /usr/local/bin/tuigreet
```

### From AUR

On ArchLinux, `tuigreet` is available on [AUR](https://aur.archlinux.org/packages/greetd-tuigreet) and is installable through your preferred AUR helper:

```
$ yay -S greetd-tuigreet
```

Two more distributions are available: `greetd-tuigreet-bin` is the precompiled release for the latest tagged release of `tuigreet` and `greetd-tuigreet-git` is a rolling release always following the `master` branch of this repository.

### From Gentoo

On Gentoo, `tuigreet` is available as a package `gui-apps/tuigreet`:

```
$ emerge --ask --verbose gui-apps/tuigreet
```

### From NixOS

On NixOS `greetd` and `tuigreet` both available via `<nixpkgs>` main repository.
Please refer to the snippet below for the minimal `tuigreet` configuration:

```nix
{ pkgs, ... }:
{
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd sway";
        user = "greeter";
      };
    };
  };
}
```

[More details](https://search.nixos.org/options?channel=unstable&show=services.greetd.settings&from=0&size=50&sort=relevance&query=greetd)

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

#### Desktop environments

`greetd` only accepts environment-less commands to be used to start a session. Therefore, if your desktop environment requires either arguments or environment variables, you will need to create a wrapper script and refer to it in an appropriate desktop file.

For example, to run X11 Gnome, you may need to start it through `startx` and configure your `~/.xinitrc` (or an external `xinitrc` with a wrapper script):

```
exec gnome-session
```

To run Wayland Gnome, you would need to create a wrapper script akin to the following:

```
XDG_SESSION_TYPE=wayland dbus-run-session gnome-session
```

Then refer to your wrapper script in a custom desktop file (in a directory declared with the `-s/--sessions` option):

```
Name=Wayland Gnome
Exec=/path/to/my/wrapper.sh
```

### Power management

Two power actions are possible from `tuigreet`, shutting down (through `shutdown -h now`) and rebooting (with `shutdown -r now`) the machine. This requires that those commands be executable by regular users, which is not the case on some distros.

To alleviate this, there are two options that can be used to customize the commands that are run: `--power-shutdown` and `--power-reboot`. The provided commands must be non-interactive, meaning they will not be able to print anything or prompt for anything. If you need to use `sudo` or `doas`, they will need to be configured to run passwordless for those specific commands.

An example for `/etc/greetd/config.toml`:

```
[default_session]
command = "tuigreet --power-shutdown 'sudo systemctl poweroff'"
```

Note that, by default, all commands are prefixed with `setsid` to completely detach the command from our TTY. If you would prefer to run the commands as is, or if `setsid` does not exist on your system, you can use `--power-no-setsid`.

### User menu

Optionally, a user can be selected from a menu instead of typing out their name, with the `--user-menu` option, this will present all users present in `/etc/passwd` at the time `tuigreet` was run, with a UID within the acceptable range. The values for the minimum and maximum UIDs are selected as follows, for each value:

 * A user-provided value, through `--user-menu-min-uid` or `--user-menu-max-uid`;
 * **Or**, the available values for `UID_MIN` or `UID_MAX` from `/etc/login.defs`;
 * **Or**, hardcoded `1000` for minimum UID and `60000` for maximum UID.
