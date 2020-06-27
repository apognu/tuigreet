# greetd-tui

Graphical console greter for [greetd](https://git.sr.ht/~kennylevinsen/greetd).

![Screenshot of greetd-tui](https://github.com/apognu/greetd-tui/blob/master/contrib/screenshot.png)

## Usage

```
Usage: greetd-tui [options]

Options:
    -c, --cmd COMMAND   command to run
    -i, --issue         show the host's issue file
    -g, --greeting GREETING
                        show custom text above login prompt
    -h, --help          show this usage information
```

## Configuration

Edit `/etc/greetd/config.toml` and set the `command` setting to use `greetd-tui`:

```
[terminal]
vt = 1

[default_session]
command = "greetd-tui --cmd sway"
user = "greeter"
```
