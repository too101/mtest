# MTEST

Version v.20261109 (versions are date codes, v.YYYYMMDD).

A monitor test for today's Windows, Linux and macOS, rebuilt from an old 16-bit Windows monitor test
program: geometry, brightness and contrast, high voltage, colours, convergence, focus, resolution,
moiré, readability and jitter patterns, a left-to-right speaker test and a demo mode. Full screen,
in 10 languages (English, Thai, Finnish, German, French, Spanish, Dutch, Swedish, Norwegian, Italian).

## Use

- Main screen: click a button (Tab and Enter work too). F1 for help, Alt+F4 or EXIT to quit.
- In a test: click or Page Down for the next pattern (after the last one you return), right click or
  Page Up for the previous one, Esc to go back.
- Ctrl+Alt+D starts the demo (Esc stops it). Ctrl+Alt+1 to 9 narrows the window to 10-90 % of the
  screen width, Ctrl+Alt+0 makes it full screen again.

## Download and build

Programs for every system are built by GitHub Actions (`.github/workflows/build.yml`): see the
Actions run's artifacts, or the release for a version tag. To build yourself, see `BUILDING.md`.
How each pattern is drawn, with the original program's code addresses, is in `MECHANICS.md`.
