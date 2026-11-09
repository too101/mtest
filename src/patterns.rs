//! Every screen of the program, drawn pixel by pixel at the window's real resolution with the
//! rules decoded from the original (addresses are Ghidra `segment:offset`; see MECHANICS.md).

use crate::bitmaps;
use eframe::egui::{Color32, ColorImage};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Test {
    Geometry,
    Brightness,
    HighVoltage,
    Colors,
    Convergence,
    Focus,
    Resolution,
    Moire,
    Readability,
    Jitter,
}

pub const PATTERN_TESTS: [Test; 10] = [
    Test::Geometry,
    Test::Brightness,
    Test::HighVoltage,
    Test::Colors,
    Test::Convergence,
    Test::Focus,
    Test::Resolution,
    Test::Moire,
    Test::Readability,
    Test::Jitter,
];

/// High voltage swaps black and white every 1000 ms (SetTimer 0x3e8, 1008:7c8c).
pub const HV_SWITCH_SECS: f64 = 1.0;

const BLACK: Color32 = Color32::BLACK;
const WHITE: Color32 = Color32::WHITE;
const RED: Color32 = Color32::from_rgb(255, 0, 0);
const GREEN: Color32 = Color32::from_rgb(0, 255, 0);
const BLUE: Color32 = Color32::from_rgb(0, 0, 255);
/// Pen 0x0200_80FF (1008:3a46): the circles of the main screen.
pub const ORANGE: Color32 = Color32::from_rgb(255, 128, 0);

/// Colour bars of the main screen, COLORREF table at code 1000:0048.
const CARD_BARS: [Color32; 8] = [
    Color32::from_rgb(0, 0, 255),
    Color32::from_rgb(0, 255, 255),
    Color32::from_rgb(255, 0, 255),
    Color32::from_rgb(255, 0, 128),
    Color32::from_rgb(255, 0, 0),
    Color32::from_rgb(255, 128, 0),
    Color32::from_rgb(255, 255, 0),
    Color32::from_rgb(0, 255, 0),
];
/// Grey steps of the main screen in percent, byte table at code 1000:003c.
pub const CARD_PERCENTS: [i64; 11] = [3, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
/// Colour ramps of the main screen: bright 255 at the ends, 95 in the middle (code 1000:001c-0033).
const RAMP_HIGH: i64 = 255;
const RAMP_LOW: i64 = 95;
/// Colours test: white, red, green, blue (code 1000:1be0, indexed alt % 4 by 1000:1c24).
const SOLID: [Color32; 4] = [WHITE, RED, GREEN, BLUE];
/// One-colour convergence pens: white, magenta, yellow, cyan (code 1000:1bf0).
const CONVERGENCE: [Color32; 4] = [WHITE, Color32::from_rgb(255, 0, 255), Color32::from_rgb(255, 255, 0), Color32::from_rgb(0, 255, 255)];
/// The 15 buttons' cells on the main screen, (column, row) byte pairs at code 1000:0146.
pub const BUTTON_CELLS: [(i64, i64); 15] =
    [(0, 0), (1, 0), (2, 0), (3, 0), (5, 0), (6, 0), (7, 0), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)];
/// Line groups of the resolution figure: (line width, count), widths 3, 2, 1 with counts from code 1000:1c10.
const GRATING: [(i64, i64); 3] = [(3, 4), (2, 5), (1, 10)];
/// Side of a resolution figure (0x7a, 1008:86d7) and length of its lines (0x3d, 1000:242f).
pub const RES_FIGURE: i64 = 122;
const RES_LINE: i64 = 61;

impl Test {
    /// Alternatives per test: field +0x1a set by each test's constructor.
    pub fn alternatives(self) -> usize {
        match self {
            Test::Geometry => 4,    // 1008:73fd
            Test::Brightness => 2,  // 1008:7629
            Test::HighVoltage => 2, // 1008:7c7d
            Test::Colors => 5,      // 1008:7f09
            Test::Convergence => 5, // 1008:8061
            Test::Focus => 3,       // 1008:80d1
            Test::Resolution => 2,  // 1008:85c9
            Test::Moire => 6,       // 1008:87eb
            Test::Readability => 2, // 1008:885b
            Test::Jitter => 1,      // 1008:8d67
        }
    }

    pub fn animated(self) -> bool {
        matches!(self, Test::HighVoltage)
    }

    /// The help topic of a pattern.
    pub fn topic(self, alt: usize) -> crate::help::Topic {
        use crate::help::Topic;
        match (self, alt) {
            (Test::Geometry, _) => Topic::Geometry,
            (Test::Brightness, 0) => Topic::Brightness,
            (Test::Brightness, _) => Topic::Luminosity,
            (Test::HighVoltage, _) => Topic::HighVoltage,
            (Test::Colors, 4) => Topic::ColorSquare,
            (Test::Colors, _) => Topic::Colors,
            (Test::Convergence, 4) => Topic::Convergence3,
            (Test::Convergence, _) => Topic::Convergence1,
            (Test::Focus, _) => Topic::Focus,
            (Test::Resolution, _) => Topic::Resolution,
            (Test::Moire, _) => Topic::Moire,
            (Test::Readability, _) => Topic::Readability,
            (Test::Jitter, _) => Topic::Jitter,
        }
    }
}

/// Windows' MulDiv: a * b / c, rounded.
pub fn mul_div(a: i64, b: i64, c: i64) -> i64 {
    if c == 0 {
        return 0;
    }
    ((a * b) as f64 / c as f64).round() as i64
}

/// The grid every screen is laid out on (window-size handler 1000:1284).
#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub w: i64,
    pub h: i64,
    pub nx: i64,
    pub ny: i64,
}

impl Grid {
    /// Columns: the first of 12..=16 that divides the width exactly and gives at least
    /// 10 rows (rows = height * columns / width); otherwise 16 x 12.
    pub fn new(w: i64, h: i64) -> Grid {
        for nx in 12..=16 {
            if w % nx == 0 {
                let ny = h * nx / w;
                if ny >= 10 {
                    return Grid { w, h, nx, ny };
                }
            }
        }
        Grid { w, h, nx: 16, ny: 12 }
    }
    /// Vertical line i (1008:0b48).
    pub fn x(&self, i: i64) -> i64 {
        i * self.w / self.nx
    }
    /// Horizontal line j (1008:0bb4).
    pub fn y(&self, j: i64) -> i64 {
        j * self.h / self.ny
    }
    /// Column offset of the main screen layout (field 0x36c).
    pub fn ox(&self) -> i64 {
        (self.nx - 12) / 2 + 1
    }
    /// Row offset of the main screen layout (field 0x36e).
    pub fn oy(&self) -> i64 {
        (self.ny - 10) >> 1
    }
}

/// A rectangle in pixels, right and bottom exclusive (like GDI).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct R {
    pub l: i64,
    pub t: i64,
    pub r: i64,
    pub b: i64,
}

impl R {
    pub fn new(l: i64, t: i64, r: i64, b: i64) -> R {
        R { l, t, r, b }
    }
    pub fn w(&self) -> i64 {
        self.r - self.l
    }
    pub fn h(&self) -> i64 {
        self.b - self.t
    }
    pub fn inset(&self, d: i64) -> R {
        R::new(self.l + d, self.t + d, self.r - d, self.b - d)
    }
}

/// A plain RGBA framebuffer.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Color32>,
}

impl Canvas {
    pub fn new(w: usize, h: usize, bg: Color32) -> Self {
        Canvas { w, h, px: vec![bg; w * h] }
    }

    #[inline]
    pub fn set(&mut self, x: i64, y: i64, c: Color32) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }

    #[inline]
    fn get(&self, x: i64, y: i64) -> Option<Color32> {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            Some(self.px[y as usize * self.w + x as usize])
        } else {
            None
        }
    }

    /// MoveTo/LineTo for straight lines: the end point is not drawn.
    pub fn line(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, c: Color32) {
        if y0 == y1 {
            let step = if x1 >= x0 { 1 } else { -1 };
            let mut x = x0;
            while x != x1 {
                self.set(x, y0, c);
                x += step;
            }
        } else if x0 == x1 {
            let step = if y1 >= y0 { 1 } else { -1 };
            let mut y = y0;
            while y != y1 {
                self.set(x0, y, c);
                y += step;
            }
        }
    }

    pub fn fill(&mut self, r: R, c: Color32) {
        let (x0, x1) = (r.l.max(0), r.r.min(self.w as i64));
        let (y0, y1) = (r.t.max(0), r.b.min(self.h as i64));
        for y in y0..y1 {
            for x in x0..x1 {
                self.px[y as usize * self.w + x as usize] = c;
            }
        }
    }

    /// GDI Rectangle with a hollow brush: the outline of r.
    pub fn frame(&mut self, r: R, c: Color32) {
        self.line(r.l, r.t, r.r - 1, r.t, c);
        self.line(r.r - 1, r.t, r.r - 1, r.b - 1, c);
        self.line(r.r - 1, r.b - 1, r.l, r.b - 1, c);
        self.line(r.l, r.b - 1, r.l, r.t, c);
    }

    /// The outline of r with a GDI PS_DASH pen: 18 pixels on, 6 off, continuing round the corners.
    pub fn dashed_frame(&mut self, r: R, c: Color32) {
        let (l, t, rr, b) = (r.l, r.t, r.r - 1, r.b - 1);
        let mut path: Vec<(i64, i64)> = Vec::new();
        for x in l..rr {
            path.push((x, t));
        }
        for y in t..b {
            path.push((rr, y));
        }
        for x in (l + 1..=rr).rev() {
            path.push((x, b));
        }
        for y in (t + 1..=b).rev() {
            path.push((l, y));
        }
        for (n, (x, y)) in path.into_iter().enumerate() {
            if n % 24 < 18 {
                self.set(x, y, c);
            }
        }
    }

    /// GDI Ellipse with a hollow brush: the outline of the ellipse inside r.
    pub fn ellipse(&mut self, r: R, c: Color32) {
        let cx = (r.l + r.r - 1) as f64 / 2.0;
        let cy = (r.t + r.b - 1) as f64 / 2.0;
        let rx = ((r.r - r.l - 1) as f64 / 2.0).max(0.5);
        let ry = ((r.b - r.t - 1) as f64 / 2.0).max(0.5);
        for x in r.l..r.r {
            let u = (x as f64 - cx) / rx;
            let d = ry * (1.0 - u * u).max(0.0).sqrt();
            self.set(x, (cy - d).round() as i64, c);
            self.set(x, (cy + d).round() as i64, c);
        }
        for y in r.t..r.b {
            let v = (y as f64 - cy) / ry;
            let d = rx * (1.0 - v * v).max(0.0).sqrt();
            self.set((cx - d).round() as i64, y, c);
            self.set((cx + d).round() as i64, y, c);
        }
    }

    /// PatBlt with a pattern brush made from a mono bitmap: set bits get `one`, clear bits `zero`.
    /// The pattern is aligned to the window's origin.
    pub fn pattern(&mut self, r: R, bits: &[&str], one: Color32, zero: Color32) {
        let bh = bits.len() as i64;
        let bw = bits[0].len() as i64;
        let rows: Vec<&[u8]> = bits.iter().map(|s| s.as_bytes()).collect();
        for y in r.t.max(0)..r.b.min(self.h as i64) {
            let row = rows[y.rem_euclid(bh) as usize];
            for x in r.l.max(0)..r.r.min(self.w as i64) {
                let c = if row[x.rem_euclid(bw) as usize] == b'#' { one } else { zero };
                self.px[y as usize * self.w + x as usize] = c;
            }
        }
    }

    /// Copies a mono bitmap at (x, y): set bits get `one`, clear bits `zero`.
    pub fn blit(&mut self, x: i64, y: i64, bits: &[&str], one: Color32, zero: Color32) {
        for (j, row) in bits.iter().enumerate() {
            for (i, b) in row.bytes().enumerate() {
                self.set(x + i as i64, y + j as i64, if b == b'#' { one } else { zero });
            }
        }
    }

    /// Copies a mono bitmap with SRCINVERT: set bits invert the screen.
    pub fn blit_invert(&mut self, x: i64, y: i64, bits: &[&str]) {
        for (j, row) in bits.iter().enumerate() {
            for (i, b) in row.bytes().enumerate() {
                if b == b'#' {
                    let (px, py) = (x + i as i64, y + j as i64);
                    if let Some(c) = self.get(px, py) {
                        self.set(px, py, Color32::from_rgb(255 - c.r(), 255 - c.g(), 255 - c.b()));
                    }
                }
            }
        }
    }

    pub fn into_image(self) -> ColorImage {
        ColorImage::new([self.w, self.h], self.px)
    }
}

fn grey(v: i64) -> Color32 {
    Color32::from_gray(v.clamp(0, 255) as u8)
}

/// Grey of a light-output percentage: MulDiv(p, 255, 100).
pub fn percent_grey(p: i64) -> Color32 {
    grey(mul_div(p, 255, 100))
}

/// The grid (1008:0c20): line i at x(i), full height; with `half`, every line but the first also
/// gets one half a cell before it.
fn grid_lines(c: &mut Canvas, g: &Grid, half: bool, col: Color32) {
    let (hx, hy) = (g.x(1) >> 1, g.y(1) >> 1);
    for i in 0..=g.nx {
        let x = g.x(i).min(g.w - 1);
        c.line(x, 0, x, g.h, col);
        if half && i != 0 {
            c.line(x - hx, 0, x - hx, g.h, col);
        }
    }
    for j in 0..=g.ny {
        let y = g.y(j).min(g.h - 1);
        c.line(0, y, g.w, y, col);
        if half && j != 0 {
            c.line(0, y - hy, g.w, y - hy, col);
        }
    }
}

/// The circles (1008:0dda): one of diameter min(W, H) in the middle and, with `corners`, an
/// ellipse in each corner's 2 x 2 cells. Returns the corner rectangles.
fn circles(c: &mut Canvas, g: &Grid, corners: bool, col: Color32) -> [R; 4] {
    let d = g.w.min(g.h);
    let (l, t) = ((g.w - d) >> 1, (g.h - d) >> 1);
    c.ellipse(R::new(l, t, l + d, t + d), col);
    let (gx, gy) = (g.x(2), g.y(2));
    let rects = [
        R::new(0, 0, gx, gy),
        R::new(g.w - gx, 0, g.w, gy),
        R::new(g.w - gx, g.h - gy, g.w, g.h),
        R::new(0, g.h - gy, gx, g.h),
    ];
    if corners {
        for r in rects {
            c.ellipse(r, col);
        }
    }
    rects
}

/// Where things go on the main screen (1000:09ca and 1000:1284).
pub struct CardLayout {
    pub grid: Grid,
    pub logo: R,
    pub bars: Vec<(Color32, R)>,
    pub ramps: Vec<(Color32, R)>,
    pub steps: Vec<(i64, R)>,
    pub info: R,
    pub buttons: [R; 15],
}

pub fn card_layout(w: i64, h: i64) -> CardLayout {
    let g = Grid::new(w, h);
    let (ox, oy) = (g.ox(), g.oy());
    let s = g.x(1) >> 4; // a sixteenth of a cell
    let half = g.x(1) >> 1;
    let logo = R::new(g.x(ox + 3) + 1, g.y(oy) + 1, g.x(ox + 7), g.y(oy + 1));

    // Eight colour bars in cells ox+1..ox+9, rows oy+1..oy+3.
    let mut a = R::new(g.x(ox + 1) + 1, g.y(oy + 1) + 1, g.x(ox + 9), g.y(oy + 3));
    a.l += s - 1;
    a.t += s - 1;
    a.r -= s;
    a.b -= s;
    let total = a.w() - 7 * s;
    let bars = (0..8)
        .map(|i| (CARD_BARS[i as usize], R::new(a.l + i * s + mul_div(i, total, 8), a.t, a.l + i * s + mul_div(i + 1, total, 8), a.b)))
        .collect();

    // Three ramps (red, green, blue) in cells ox-1..ox+11, rows oy+3..oy+5.
    let area = R::new(g.x(ox - 1) + 1, g.y(oy + 3) + 1, g.x(ox + 11), g.y(oy + 5));
    let span = (g.y(oy + 5) - g.y(oy + 3)) - 2 * s;
    let top = area.t + s - 1;
    let (l, r) = (area.l + half - 1, area.r - half);
    let ramp_cols = [Color32::from_rgb(255, 0, 0), Color32::from_rgb(0, 255, 0), Color32::from_rgb(0, 0, 255)];
    let ramps = (0..3)
        .map(|k| (ramp_cols[k as usize], R::new(l, top + span * k / 3, r, top + span * (k + 1) / 3)))
        .collect();

    // Eleven grey steps in cells ox-1..ox+11, rows oy+5..oy+7.
    let mut a = R::new(g.x(ox - 1) + 1, g.y(oy + 5) + 1, g.x(ox + 11), g.y(oy + 7));
    a.l += half - 1;
    a.t += s;
    a.r -= half;
    a.b -= s;
    let total = a.w() - 10 * s;
    let steps = (0..11)
        .map(|i| (CARD_PERCENTS[i as usize], R::new(a.l + i * s + mul_div(i, total, 11), a.t, a.l + i * s + mul_div(i + 1, total, 11), a.b)))
        .collect();

    let info = R::new(g.x(ox + 2) + 1, g.y(oy + 9) + 1, g.x(ox + 8), g.y(oy + 10));

    let buttons = std::array::from_fn(|k| {
        let (cx, cy) = BUTTON_CELLS[k];
        let (col, row) = (cx + ox + 1, cy + oy + 7);
        let wide = if k == 3 { 1 } else { 0 };
        R::new(g.x(col) + 1, g.y(row) + 1, g.x(col + 1 + wide), g.y(row + 1))
    });
    CardLayout { grid: g, logo, bars, ramps, steps, info, buttons }
}

/// The main screen's test card (1000:09ca).
pub fn render_card(w: usize, h: usize) -> ColorImage {
    let (w, h) = (w.max(64), h.max(64));
    let lay = card_layout(w as i64, h as i64);
    let g = lay.grid;
    let mut c = Canvas::new(w, h, BLACK);
    grid_lines(&mut c, &g, false, WHITE);
    c.fill(lay.logo, BLACK);
    let bar_area = R::new(g.x(g.ox() + 1) + 1, g.y(g.oy() + 1) + 1, g.x(g.ox() + 9), g.y(g.oy() + 3));
    c.fill(bar_area, BLACK);
    for (col, r) in &lay.bars {
        c.fill(*r, *col);
    }
    let ramp_area = R::new(g.x(g.ox() - 1) + 1, g.y(g.oy() + 3) + 1, g.x(g.ox() + 11), g.y(g.oy() + 5));
    c.fill(ramp_area, BLACK);
    for (col, r) in &lay.ramps {
        // Bright at both ends, darkest in the middle (1008:0ff2).
        let n = (r.w() - 1).max(1) as f64;
        for x in r.l..r.r {
            let t = (x - r.l) as f64 / n;
            let k = if t < 0.5 { 1.0 - t * 2.0 } else { (t - 0.5) * 2.0 };
            let v = (RAMP_LOW as f64 + (RAMP_HIGH - RAMP_LOW) as f64 * k).round() as u8;
            let px = Color32::from_rgb(
                if col.r() > 0 { v } else { 0 },
                if col.g() > 0 { v } else { 0 },
                if col.b() > 0 { v } else { 0 },
            );
            c.fill(R::new(x, r.t, x + 1, r.b), px);
        }
    }
    let step_area = R::new(g.x(g.ox() - 1) + 1, g.y(g.oy() + 5) + 1, g.x(g.ox() + 11), g.y(g.oy() + 7));
    c.fill(step_area, BLACK);
    for (p, r) in &lay.steps {
        c.fill(*r, percent_grey(*p));
    }
    c.fill(lay.info, BLACK);
    // The grid's last row and column sit on the window's edge; draw them over the boxes so the
    // card is framed all round (the original's last lines fall just outside the window).
    c.line(0, g.h - 1, g.w, g.h - 1, WHITE);
    c.line(g.w - 1, 0, g.w - 1, g.h, WHITE);
    circles(&mut c, &g, true, ORANGE);
    for r in &lay.buttons {
        c.fill(*r, BLACK);
    }
    c.into_image()
}

/// Brightness steps of the first brightness alternative: (percent, rectangle).
pub fn brightness_blocks(w: i64, h: i64) -> Vec<(i64, R)> {
    let g = Grid::new(w, h);
    let step = (h / 8 / 11).min(w / 8 / 11);
    let inner = R::new(0, 0, w, h).inset(11 * step);
    let s = g.x(1) >> 4;
    let total = inner.w() - 8 * s;
    let mid = (inner.t + inner.b) / 2 - s;
    let mut out = Vec::new();
    for i in 0..9 {
        let x0 = inner.l + i * s + mul_div(i, total, 9);
        let x1 = inner.l + i * s + mul_div(i + 1, total, 9);
        out.push((i + 1, R::new(x0, inner.t, x1, mid)));
        out.push((i * 10 + 20, R::new(x0, mid + s, x1, inner.b)));
    }
    out
}

/// Readability boxes: (rectangle, horizontal align, vertical align); 0 start, 1 centre, 2 end (1008:8bfa).
pub fn readability_boxes(w: i64, h: i64) -> [(R, u8, u8); 5] {
    let (bw, bh) = (w / 4, h / 4);
    let at = |x: i64, y: i64| R::new(x, y, x + bw, y + bh);
    [
        (at(0, 0), 0, 0),
        (at(w - bw, 0), 2, 0),
        (at(0, h - bh + 1), 0, 2),
        (at(w - bw, h - bh + 1), 2, 2),
        (at((w - bw) / 2, (h - bh) / 2), 1, 1),
    ]
}

/// Background of each screen (each class's background method).
pub fn background(test: Test, alt: usize) -> Color32 {
    match test {
        Test::Colors if alt < 4 => SOLID[alt % 4],         // 1008:7ffa
        Test::Focus | Test::Readability | Test::Resolution => {
            if alt == 0 {
                if test == Test::Focus { BLACK } else { WHITE } // 1008:8568, 8d06, 878a
            } else if test == Test::Focus {
                WHITE
            } else {
                BLACK
            }
        }
        Test::Moire if alt < 2 => grey(alt as i64 * 32 + 0x80), // 1000:2646
        _ => BLACK,                                         // 1008:71b4
    }
}

/// `dpi` is the logical dots per inch (96 at 100 % scaling).
pub fn render(test: Test, alt: usize, w: usize, h: usize, phase: u64, dpi: f64) -> ColorImage {
    let (w, h) = (w.max(64), h.max(64));
    let (wi, hi) = (w as i64, h as i64);
    let g = Grid::new(wi, hi);
    let mut c = Canvas::new(w, h, background(test, alt));
    let dpi = dpi.round() as i64;
    match test {
        Test::Geometry => {
            // 1008:741c: white (alt 0, 2) or green (1, 3) lines; half-cell lines from alt 2.
            let col = if alt % 2 == 0 { WHITE } else { GREEN };
            grid_lines(&mut c, &g, alt >= 2, col);
            circles(&mut c, &g, true, col);
            c.frame(R::new(0, 0, wi, hi), BLACK);
            c.dashed_frame(R::new(0, 0, wi, hi), col);
        }
        Test::Brightness if alt == 0 => {
            // 1008:7854: 11 nested frames from 50 % grey to black, then the steps.
            let step = (hi / 8 / 11).min(wi / 8 / 11);
            for i in 0..11 {
                c.fill(R::new(0, 0, wi, hi).inset(i * step), grey(mul_div(128, 10 - i, 10)));
            }
            for (p, r) in brightness_blocks(wi, hi) {
                c.fill(r, percent_grey(p));
            }
        }
        Test::Brightness => {
            // 1008:769b: five white squares of 4/25 inch.
            let (sw, sh) = (mul_div(4, dpi, 25), mul_div(4, dpi, 25));
            let (x0, y0) = (wi >> 4, hi >> 4);
            let (x1, y1) = (wi - x0 - sw + 1, hi - y0 - sh + 1);
            for (x, y) in [((wi - sw) >> 1, (hi - sh) >> 1), (x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
                c.fill(R::new(x, y, x + sw, y + sh), WHITE);
            }
        }
        Test::HighVoltage => {
            // 1008:7d12: a frame 1 % of the height in, then bands that swap every second.
            let m = hi / 100;
            let outer = R::new(0, 0, wi, hi).inset(m);
            c.frame(outer, WHITE);
            let inner = outer.inset(m);
            let on = phase % 2 == 1;
            if alt == 0 {
                for i in 0..3 {
                    let t = inner.t + inner.h() * i / 3;
                    let b = inner.t + inner.h() * (i + 1) / 3;
                    let white = ((i & 1) == 1) ^ on;
                    c.fill(R::new(inner.l, t, inner.r, b), if white { WHITE } else { BLACK });
                }
            } else {
                c.fill(inner, if on { WHITE } else { BLACK });
            }
        }
        Test::Colors => {
            if alt == 4 {
                // 1008:7f28: a 2 x 2 inch square in the middle.
                let (sw, sh) = (dpi * 2, dpi * 2);
                let (l, t) = ((wi - sw) >> 1, (hi - sh) >> 1);
                c.fill(R::new(l, t, l + sw, t + sh), SOLID[0]);
            }
        }
        Test::Convergence => {
            if alt < 4 {
                // 1000:1c5c: grid and circle in white, magenta, yellow or cyan.
                grid_lines(&mut c, &g, false, CONVERGENCE[alt]);
                circles(&mut c, &g, false, CONVERGENCE[alt]);
            } else {
                // Pieces that change colour halfway between crossings: red, green, blue by (i + j) % 3.
                let (w1, h1) = (wi - 1, hi - 1);
                let (hx, hy) = ((w1 / g.nx) / 2, (h1 / g.ny) / 2);
                let rgb = [RED, GREEN, BLUE];
                for j in 0..=g.ny {
                    for i in 0..g.nx + 2 {
                        let col = rgb[((i + j) % 3) as usize];
                        let x = i * w1 / g.nx;
                        c.line(x, j * h1 / g.ny - hy, x, (j + 1) * h1 / g.ny - hy, col);
                        let y = j * h1 / g.ny;
                        c.line(i * w1 / g.nx - hx, y, (i + 1) * w1 / g.nx - hx, y, col);
                    }
                }
                circles(&mut c, &g, false, GREEN);
            }
        }
        Test::Focus => focus(&mut c, wi, hi, alt),
        Test::Resolution => {
            // 1008:866c: five line-grating figures.
            let col = if alt == 0 { BLACK } else { WHITE };
            let f = RES_FIGURE;
            for (x, y) in [(0, 0), (wi - f, 0), (wi - f, hi - f), (0, hi - f), ((wi - f) >> 1, (hi - f) >> 1)] {
                resolution_figure(&mut c, x, y, col);
            }
        }
        Test::Moire => match alt {
            2 => {
                let mut x = 1;
                while x < wi {
                    c.line(x, 0, x, hi, WHITE);
                    x += 2;
                }
            }
            3 => {
                let mut y = 1;
                while y < hi {
                    c.line(0, y, wi, y, WHITE);
                    y += 2;
                }
            }
            4 => c.pattern(R::new(0, 0, wi, hi), &bitmaps::CHECKER_2, BLACK, WHITE),
            5 => c.pattern(R::new(0, 0, wi, hi), &bitmaps::CHECKER_1, BLACK, WHITE),
            _ => {}
        },
        Test::Readability => {}
        Test::Jitter => {
            // 1008:8d86: eight columns, each a white block and a checker block.
            let r = R::new(0, 0, wi, hi).inset(10);
            let ht = r.h();
            for i in 0..8 {
                let x0 = r.l + mul_div(i, r.w(), 8);
                let x1 = r.l + mul_div(i + 1, r.w(), 8);
                c.line(x0, r.t, x0, r.b, WHITE);
                c.line(x1, r.t, x1, r.b, WHITE);
                c.fill(R::new(x0 + 4, r.t + ht / 20, x1 - 3, r.t + mul_div(ht, 70, 100)), WHITE);
                c.pattern(R::new(x0 + 4, r.t + mul_div(ht, 75, 100), x1 - 3, r.t + mul_div(ht, 95, 100)), &bitmaps::CHECKER_2, BLACK, WHITE);
            }
        }
    }
    c.into_image()
}

/// Resolution figure (1008:85e8): four gratings turning round a 122-pixel square.
fn resolution_figure(c: &mut Canvas, x: i64, y: i64, col: Color32) {
    let l = RES_LINE;
    grating(c, x, y, 1, 0, col);
    grating(c, x + l, y, 0, 1, col);
    grating(c, x + 2 * l - 1, y + l, -1, 0, col);
    grating(c, x, y + 2 * l - 1, 0, -1, col);
}

/// One grating (1000:23ae): 4 lines 3 px wide, 5 lines 2 px, 10 lines 1 px, gaps as wide as the lines.
fn grating(c: &mut Canvas, mut x: i64, mut y: i64, dx: i64, dy: i64, col: Color32) {
    let mut first = true;
    for (width, count) in GRATING {
        for _ in 0..count {
            if !first {
                x += dx * width;
                y += dy * width;
            }
            for _ in 0..width {
                c.line(x, y, x + dy.abs() * RES_LINE, y + dx.abs() * RES_LINE, col);
                x += dx;
                y += dy;
            }
            first = false;
        }
    }
}

fn focus(c: &mut Canvas, w: i64, h: i64, alt: usize) {
    if alt == 0 {
        // 1000:1f24: two frames, then a 5 x 5 field of figures.
        c.frame(R::new(0, 0, w, h), WHITE);
        c.frame(R::new(2, 2, w - 2, h - 2), WHITE);
        let (fx, fy, fw, fh) = (6, 6, w - 12, h - 12);
        let mut k = 0;
        for col in 0..5 {
            for row in 0..5 {
                let x = fx + mul_div(fw, col, 5);
                let y = fy + mul_div(fh, row, 5);
                let cell = R::new(x, y, fx + mul_div(fw, col + 1, 5), fy + mul_div(fh, row + 1, 5));
                if col == 2 && (row == 1 || row == 3) {
                    c.pattern(cell, &bitmaps::CHECKER_1, WHITE, BLACK);
                } else {
                    focus_figure(c, cell, k % 2 == 0);
                }
                k += 1;
            }
        }
    } else {
        // 4 x 4 tiles of bitmap 138 (alt 1) or 139 (alt 2) in the corners and the middle.
        let bits: &[&str] = if alt >= 2 { &bitmaps::GLYPHS } else { &bitmaps::SQUARES };
        let (bw, bh) = (bits[0].len() as i64, bits.len() as i64);
        let (tw, th) = (bw * 4, bh * 4);
        for (x, y) in [(0, 0), (w - tw, 0), (w - tw, h - th), (0, h - th), ((w - tw) >> 1, (h - th) >> 1)] {
            for i in 0..4 {
                for j in 0..4 {
                    c.blit(x + i * bw, y + j * bh, bits, WHITE, BLACK);
                }
            }
        }
    }
}

/// One focus figure (1008:8112): the cell white (even cell number) or black, a cross of line
/// pairs one pixel either side of the middle, half-length `(min(w, h) >> 3) | 1`, in the contrasting
/// pen; then a row of the small text bitmap 135 drawn inverted (SRCINVERT, 1008:729e), centred, in the
/// top quarter and the bottom quarter of the cell, an eighth of the width in from each side.
fn focus_figure(c: &mut Canvas, cell: R, white: bool) {
    let (bg, ink) = if white { (WHITE, BLACK) } else { (BLACK, WHITE) };
    c.fill(cell, bg);
    let (x, y, w, h) = (cell.l, cell.t, cell.w(), cell.h());
    let u = (w.min(h) >> 3) | 1;
    let (cx, cy) = (x + (w >> 1), y + (h >> 1));
    for d in [-1, 1] {
        c.line(cx + d, cy - u, cx + d, cy + u, ink);
        c.line(cx - u, cy + d, cx + u, cy + d, ink);
    }
    let (gw, gh) = (bitmaps::ME_TEXT[0].len() as i64, bitmaps::ME_TEXT.len() as i64);
    let mut m = h - (h >> 2);
    if gh + 4 > h - m {
        m = h - gh - 4;
    }
    let a = w >> 3;
    let (bw, bh) = (w - 2 * a, h - m);
    for by in [y, y + m] {
        let count = (bw / gw).max(1);
        let mut gx = x + a + (bw - gw * count) / 2;
        let gy = by + ((bh - gh) >> 1);
        for _ in 0..count {
            c.blit_invert(gx, gy, &bitmaps::ME_TEXT);
            gx += gw;
        }
    }
}
