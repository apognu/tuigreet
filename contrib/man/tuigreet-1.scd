tuigreet(1)

# NAME

tuigreet - A graphical console greeter for greetd

# SYNOPSIS

*tuigreet* [OPTIONS]...

# OPTIONS

*-h, --help*
	Show usage and exit.

*-v, --version*
	Print program version and exit.

*-c, --cmd CMD*
	Specify which command to run on successful authentication. This can be
	overriden by manual selection within *tuigreet*.

*-s, --sessions DIR1[:DIR2]...*
	Location of desktop-files to be used as session definitions. By default,
	sessions are fetched from */usr/share/xsessions* and
	*/usr/share/wayland-sessions*.

*-w, --width COLS*
	Number of columns the main prompt area should take on the screen.

*-i, --issue*
	Print the content of */etc/issue* at the top of the prompt area.

	This option is mutually exclusive with *--greeting*.

*-g, --greeting GREETING*
	Specify the text to be displayed at the top of the prompt area.

	This option is mutually exclusive with *--issue*.

*-t, --time*
	Print the current date and time at the top of the screen.

*--time-format FORMAT*
	Configure a custom strftime-compliant format string for the current date
	and time.

*--user-menu*
	Allow selecting a user from a graphical menu.

*--user-menu-min-uid*
	Minimum UID of the users to display in the selection menu.

*--user-menu-max-uid*
	Maximum UID of the users to display in the selection menu.

*-r, --remember*
	Remember the username of the last successfully opened session, so the
	username field will be pre-filled on the next run.

*--remember-session*
	Remember the last selected session, effectively overriding the given *--cmd*
	option on subsequent runs.

*--remember-user-session*
	Remember the last opened session, per user (requires *--remember*).

*--asterisks*
	Add visual feedback when typing secrets, as one asterisk character for every
	keystroke. By default, no feedback is given at all.

*--asterisks-char CHAR*
	Change the default feedback character from an asterisk to the provided
	character.

*--window-padding COLS*
	Add spacing between the edge of the screen area the drawing area.

*--container-padding COLS*
	Add spacing between the border of the main prompt area and its contents.

*--prompt-padding ROWS*
	Add spacing between form fields.

*--power-shutdown CMD [ARGS]...*
	Customize the command run when instructed to shut down the machine. This must
	be a non-interactive command (sudo cannot prompt for a password, for example).

*--power-reboot CMD [ARGS]...*
	Customize the command run when instructed to reboot the machine. This must be
	a non-interactive command (sudo cannot prompt for a password, for example).

*--power-no-setsid*
	Do not prefix power commands with *setsid*, which is used to detach it from
	current TTY.


# AUTHORS

Maintained by Antoine POPINEAU <antoine@popineau.eu>.

Contributed to by great people at
https://github.com/apognu/tuigreet/graphs/contributors.

# DEVELOPMENT

Issue reporting and development discussion should happen at
https://github.com/apognu/tuigreet.

# SEE ALSO

*greetd*(1)
