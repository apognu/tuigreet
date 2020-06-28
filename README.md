# tuigreet

Graphical console greter for [greetd](https://git.sr.ht/~kennylevinsen/greetd).

![Screenshot of tuigreet](https://github.com/apognu/tuigreet/blob/master/contrib/screenshot.png)

## Usage

```
Usage: tuigreet [OPTIONS]

Options:
    -h, --help          show this usage information
    -c, --cmd COMMAND   command to run
        --width WIDTH   width of the main prompt (default: 80)
    -i, --issue         show the host's issue file
    -g, --greeting GREETING
                        show custom text above login prompt
    -t, --time          display the current date and time
```

## Configuration

Edit `/etc/greetd/config.toml` and set the `command` setting to use `tuigreet`:

```
[terminal]
vt = 1

[default_session]
command = "tuigreet --cmd sway"
user = "greeter"
```
