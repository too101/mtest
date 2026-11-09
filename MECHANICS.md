# MTEST: mechanics sheet

Original: `ntest.exe`, a 16-bit Windows 3.x MFC program (NE format, expected Windows 3.10, module
`DISPTEST`). Craft Anything's built-in engine and its Ghidra import only see the 417-byte DOS stub,
so the code was read with Ghidra's own NE loader in headless mode (`checks/ghidra/`: `asm.txt`,
`tests.txt`, `t2.txt`, `card.txt`), with code Ghidra skipped forced into instructions and a linear
capstone sweep (`sweep.txt`) to find it. Addresses are Ghidra `segment:offset`; segment 1000 is the
first code segment, 1008 the second, 1018 the data segment. Constant tables live in code segment 1000.

## Commands (switch at 1008:1efd, ids 1000-1014)

| id | Button | Handler | Class (vtable) | Alternatives (+0x1a) |
|----|--------|---------|----------------|----------------------|
| 1000 | Geometry | 1008:73ce | 1008:903e, draw 1008:741c | 4 |
| 1001 | Brightness and contrast | 1008:75fa | 1008:8f9e, draw 1008:766a | 2 |
| 1002 | High voltage | 1008:7c4e | 1008:8fee, draw 1008:7d12 | 2 |
| 1003 | www | 1008:1e81 | opened string 1003 (its maker's site); here labelled "GitHub" and opens https://github.com/too101/mtest/ at the person's request | - |
| 1004 | Colors | 1008:7eda | 1008:902a, draw 1008:7f28, bg 1008:7ffa | 5 |
| 1005 | To control panel/display | 1008:1bdd | WinExec `Rundll32 Shell32.dll,Control_RunDLL Desk.cpl @0,3` (1008:4044); here the system's display settings | - |
| 1006 | Help | 1008:1c1c | WinHelp | - |
| 1007 | Convergence | 1008:8032 | 1008:8f8a, draw 1000:1c5c | 5 |
| 1008 | Focus | 1008:80a2 | 1008:8fc6, draw 1000:1f24, bg 1008:8568 | 3 |
| 1009 | Resolution | 1008:859a | 1008:9016, draw 1008:866c, bg 1008:878a | 2 |
| 1010 | Moire | 1008:87bc | 1008:9052, draw 1000:249e, bg 1000:2646 | 6 |
| 1011 | Readability | 1008:882c | 1008:8fb2, draw 1008:8bfa, bg 1008:8d06 | 2 |
| 1012 | Jitter | 1008:8d38 | 1008:9002, draw 1008:8d86 | 1 |
| 1013 | Sound | 1008:1ddb | sndPlaySound(WAVE 142, SND_ASYNC or SND_MEMORY) | - |
| 1014 | Quit | 1008:1e6b | WM_CLOSE | - |

Default background of a test (1008:71b4): black.

## The grid (window-size handler 1000:1284)

- Columns `nx`: the first of 12, 13, 14, 15, 16 that divides the window width `W` exactly and
  gives `ny = H * nx / W >= 10` rows; otherwise `nx = 16, ny = 12`.
- Line `i` at `x = i * W / nx` (1008:0b48), line `j` at `y = j * H / ny` (1008:0bb4).
- Grid lines (1008:0c20): every line from 0 to the far edge; with the "half" flag every line but the
  first also gets one half a cell before it (`x - (x(1) >> 1)`).
- Layout offsets: `ox = (nx - 12) / 2 + 1`, `oy = (ny - 10) >> 1`.
- Circles (1008:0dda): diameter `d = min(W, H)`, at `((W - d) >> 1, (H - d) >> 1)`; with the
  "corners" flag, ellipses in the boxes `(0, 0, x(2), y(2))` and the other three corners.

## Main screen (test card 1000:09ca)

Black background, white grid, then (`s = x(1) >> 4`, `half = x(1) >> 1`):

- Name strip: cells `ox+3 .. ox+7`, row `oy` (the original draws its maker's logo, bitmap 131;
  this app writes "MONITOR TEST").
- 8 colour bars: cells `ox+1 .. ox+9`, rows `oy+1 .. oy+3`, inset `s`, gaps `s`; colours from
  1000:0048: blue, cyan, magenta, (255, 0, 128), red, orange, yellow, green.
- 3 ramps red, green, blue: cells `ox-1 .. ox+11`, rows `oy+3 .. oy+5`, inset `half` sideways and
  `s` up and down; 255 at the ends, 95 in the middle (1000:001c-0033, drawn by 1008:0ff2).
- 11 grey steps: cells `ox-1 .. ox+11`, rows `oy+5 .. oy+7`; percentages 3, 10, 20 ... 100
  (1000:003c), grey `MulDiv(p, 255, 100)`; label `%d%%` (1008:403e) in 10 pt MS Sans Serif, white
  below 50 %, black from 50 % (1000:0fd1).
- Screen mode: cells `ox+2 .. ox+8`, row `oy+9`: `"%d x %d pixels, %s"` with the window size.
- Edge: the grid's last line is at `x(nx) = W` / `y(ny) = H`, just outside the window; this app
  draws it on the last pixel row and column (also over the screen-mode box) so the card is framed.
- Circles: orange pen 0x0200_80FF (1008:3a46), with corner ellipses (the original puts its round
  logo, bitmap 133, inside them; this app leaves them empty).
- Buttons: button `k` (command 1000+k) at table cell `(cx, cy)` from 1000:0146, column
  `cx + ox + 1`, row `cy + oy + 7`, one cell (two for "www"), inset one pixel (1000:13b8-14a5).
  Table: (0,0) (1,0) (2,0) (3,0) (5,0) (6,0) (7,0) (0,1) ... (7,1).
- Button glow: the main window's create handler (1008:21b0) loads bitmap 152 (64 x 65 greys,
  0 at the corners to 205 in the middle) at 1008:2218; it is stretched over each button cell under
  the button's picture. Copied bit for bit into `src/bitmaps.rs` (a plain gradient).
- Pens (1008:38fc): black, white, red, green, blue, orange. Brushes: black and white.
  Font: MS Sans Serif, 10 pt.

## Tests

- **Geometry** (1008:741c): white lines for alternatives 0 and 2, green for 1 and 3; half-cell
  lines from alternative 2; circles with corners. The window's edge is traced twice
  (1008:74fc-75c6): first with the black pen, then with a PS_DASH pen (style 1, 1008:74e6) in the
  line colour, so the outer frame is dashed (GDI dash: 18 pixels on, 6 off).
- **Brightness** alt 0 (1008:7854): `step = min(H/8/11, W/8/11)`; 11 nested rectangles inset
  `i*step`, grey `MulDiv(128, 10-i, 10)` (128 down to 0); inside, 9 columns (`s` gaps): top row
  1..9 %, bottom row 20..100 %, grey `MulDiv(p, 255, 100)`, labelled `%d%%` (top white; bottom
  white below 50 %). Alt 1 (1008:769b): five white squares `MulDiv(4, dpi, 25)` pixels (4/25 inch),
  in the middle and at `W/16, H/16` from each corner.
- **High voltage** (1008:7d12): frame `H/100` in, white; inside another `H/100`: alt 0 three bands,
  band `i` white when `(i & 1) xor phase`; alt 1 the whole area white when `phase`. `phase` flips
  every 1000 ms (SetTimer 0x3e8, 1008:7c8c; 1008:7e92).
- **Colors** (1008:7f28, 1008:7ffa): alt 0-3 the whole screen white, red, green, blue (1000:1be0,
  `alt % 4`, 1000:1c24); alt 4 black with a white square `2 * dpi` (2 inches) in the middle.
- **Convergence** (1000:1c5c): alt 0-3 grid and circle in white, magenta, yellow, cyan (1000:1bf0);
  alt 4 pieces with `W' = W-1, H' = H-1`: for `j in 0..=ny`, `i in 0..nx+2`, colour red/green/blue by
  `(i + j) % 3`, a vertical piece at `x = i*W'/nx` from `j*H'/ny - (H'/ny)/2` to the next, and a
  horizontal piece at `y = j*H'/ny` from `i*W'/nx - (W'/nx)/2` to the next; green circle.
- **Focus** (1000:1f24): alt 0 black, frames at 0 and 2 px, then a 5 x 5 field inside 6 px; cells
  (2,1) and (2,3) hold the 1-pixel checker (bitmap 137); the others (1008:8112) alternate white/black
  (even cell number white). Each gets a cross in the contrasting pen: line pairs one pixel either
  side of the middle, half-length `(min(w,h) >> 3) | 1`. Then a row of bitmap 135 is drawn
  inverted (SRCINVERT 0x660046, 1008:729e), centred, in the top band and the bottom band of
  height `h - m`, where `m = h - h/4` (or `h - bmHeight - 4` if that is larger), `w/8` in from each
  side; as many glyphs as fit side by side (1008:843d-84d8). Alt 1 / 2 white, with bitmap 138 / 139 (40 x 40) as 4 x 4
  tiles in the four corners and the middle (1008:84e6).
- **Resolution** (1008:866c, 1008:85e8, 1000:23ae): alt 0 black on white, alt 1 white on black.
  A 122-pixel figure in the corners and the middle: four gratings of lines 61 px long turning round:
  4 lines 3 px wide, 5 lines 2 px, 10 lines 1 px (counts at 1000:1c10), gaps as wide as the lines.
- **Moire** (1000:249e, 1000:2646): alt 0 grey 128, alt 1 grey 160 (`alt*32 + 128`), alt 2 white
  vertical lines at odd x on black, alt 3 horizontal lines at odd y, alt 4 the 2-pixel checker
  (bitmap 140), alt 5 the 1-pixel checker (bitmap 137).
- **Readability** (1008:8bfa, 1008:887a): alt 0 black on white, alt 1 white on black; five boxes
  `W/4 x H/4` (top left, top right, bottom left, bottom right, middle; aligned to their corner)
  filled with 8 pt text (the original repeats its intro sentence, string 1015; this app uses its own).
- **Jitter** (1008:8d86): window inset 10 px, 8 columns between white lines; in each a white block
  from 5 % to 70 % of the height and the 2-pixel checker from 75 % to 95 %, 4 px in from the left
  line and 3 px from the right.

## Controls

- Next/previous alternative (1008:7208 / 1008:7257) stop at the last/first one (no wrap).
- Demo mode (Ctrl+Alt+D, timer 5000 ms at 1008:2002): each tick opens the next item of
  1000, 1001, 1002, 1004, 1007-1013 (table 1000:0068: the ten tests and the sound) or, in an open
  test, shows its next alternative and closes it after the last (1000:11e8, 1008:17bc).
- Main screen keys (help topic): Tab, Enter, Alt+F4, Ctrl+Alt+D, Esc.
- Mouse in a test (main window message map at code 1000:0092): left button (1008:238c) shows
  the next alternative and closes the test after the last one (1008:17bc, 1008:16d8). Right button
  (1008:2426) goes back one (1008:1814 -> 7258) and stays at the first. Both buttons together
  (flags & 3 == 3) close the test.
- While a test shows, the 15 buttons are hidden (1008:1f6e). Closing shows them again and gives the
  closed test's button the focus (1008:1755-17ae). So there is no button bar on test screens.
- Window: the original fills the whole display; this app opens full screen without frame or title.
- Keyboard (1008:27b0; Shift/Ctrl/Alt read with GetKeyState 0x10/0x11/0x12):
  - in demo mode every key is swallowed; Esc stops it (1008:281b -> 1008:206e);
  - Ctrl+Alt+D without Shift starts the demo (1008:2832 -> 1008:1fba);
  - Ctrl+Alt+0..9 without Shift moves the window to the top left at full screen height with width
    `MulDiv(n, screen width, 100) & 0xfff0`, n = digit*10 and 0 meaning 100 (1008:2869-28e1, MoveWindow);
  - F1 without Ctrl/Alt: Shift+F1 opens the help contents, F1 the help popup of the open test
    (its help id, vtable +0x10) or of the main screen (0x2710) (1008:28e9-2964);
  - in a test: Page Down = next alternative, closing after the last (1008:17bc); Page Up = previous
    (1008:1814); Esc = close (1008:16d8) (1008:296c-29c7);
  - on the main screen Tab moves between the buttons (1008:29cf).
  Space and the arrow keys in a test are this app's additions.

## Cursors

- Sound test (1008:191e): the length of WAVE 142 is worked out as data bytes x 1000 / bytes per
  second. If it is 150 to 10000 ms, a timer of length/3 starts (id 0x8af, 1008:1a3c). The cursor
  shows cursor 0x91 (loudspeaker sounding left), then on each tick 0x8f (both sides), then 0x92 (right).
  After the third tick the normal cursor returns (timer handler 1008:2b23-2b91). This app does the
  same with its 1.71 s tone (570 ms per step) and the three cursor bitmaps copied bit for bit.
- Demo mode: cursor 0x9b, a round "Demo" badge (loaded at 1008:2030 when the demo starts, set by
  the set-cursor handler at 1008:2c4a). Done the same way here.
- With two or more monitors (count from 1008:534c), the main screen shows the hand cursor 0x90
  (loaded at 1008:22c8). Pressing on the card (1008:23ec) and releasing over another monitor
  (1008:2502 -> 1008:350e) moves the window to that monitor. Not rebuilt yet: it couldn't be tried
  here with one monitor.

## Intro picture (not rebuilt)

A second window class (message map at code 1000:14e0; paint 1008:5d1c with StretchDIBits, create
1008:5e6e) shows bitmap 151, the maker's photo-and-logo picture, with its own palette. It closes
on key-up or mouse-up (1008:5e08, 5e2a, 5e4c) and starts a 1000 ms timer (id 0x33bd, 1008:60fc).
It isn't rebuilt: the picture is the maker's artwork. Where it is opened from wasn't traced.

## Settings

The only profile (INI) writes in the original are MFC's standard ones: CWinApp's recent-file list
(4 entries, 1000:73d1-742d, WritePrivateProfileString) and "PreviewPages" (1000:743a via
1000:b68a); the GetPrivateProfileString import (13d8:0090) is MFC's reader for the same. No
language entry was found there. The first-run dialog's "you won't see this dialog any more" note
(string 1020) and the file-writing messages (1031, 1034) suggest the choice is kept in a file the
program writes itself; that path wasn't traced. This app keeps the choice in `settings.txt`
(`lang=<code>`) in its own settings folder.

## Sound

WAVE resource 142: 22050 Hz, 8-bit stereo, 1.71 s, tone about 350 Hz, left level falling
128 to 12 while the right rises 12 to 123. The app synthesises a tone with these numbers.

## Not copied

The photo and logo bitmaps (131, 133, 151), the brand name, the intro sentence and the help texts
aren't reused. The five small pattern bitmaps (135, 137-140) are copied bit for bit in
`src/bitmaps.rs` because they are the test patterns themselves.

## Added in this version

- **Thai** labels (not in the original), drawn with a Thai font found on the system.
- **Linux and macOS**: settings folder, display-settings button, web link and sound playback per
  system (see BUILDING.md).

## Changed at the person's request

- Name **MTEST** (window title, program file `mtest`, name strip at about a third of the strip's
  height).
- Help, messages and the readability sample follow the chosen language (`src/help.rs`, written
  for this app). Readability shows only whole lines in each box.
- The "www" button reads "GitHub" and opens https://github.com/too101/mtest/.
