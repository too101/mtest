#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bitmaps;
mod help;
mod i18n;
mod icons;
mod patterns;
mod platform;
mod sound;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, ColorImage, FontId, Key, Modifiers, PointerButton, Rect, Sense, Stroke, TextureHandle,
    TextureOptions,
};
use i18n::{Labels, Lang};
use icons::Icon;
use patterns::Test;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Address the "www" button opens: this project's page (the original opened its maker's site,
/// string 1003).
const WEB_ADDRESS: &str = "https://github.com/too101/mtest/";
/// Demo mode steps every 5000 ms (SetTimer 0x1388, 1008:2002).
const DEMO_STEP_SECS: f32 = 5.0;
/// The original hides all buttons while a test shows (1008:1f6e); the bar stays off.
const SHOW_BAR: bool = false;
/// The test-screen button bar hides after the mouse rests this long (estimated).
const BAR_HIDE_SECS: f32 = 2.5;
/// Version as a date code, from Cargo.toml's major number: "v.20261109".
const VERSION: &str = concat!("v.", env!("CARGO_PKG_VERSION_MAJOR"));
/// Frames drawn before --screenshot takes its picture.
const SCREENSHOT_FRAMES: u64 = 120;
/// Text sizes: the original's 10 pt and 8 pt MS Sans Serif, in points at 96 dpi.
const TEXT_10PT: f32 = 10.0 * 96.0 / 72.0;
const TEXT_8PT: f32 = 8.0 * 96.0 / 72.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Button {
    Test(Test),
    Sound,
    Help,
    ControlPanel,
    Web,
    Quit,
}

/// The 15 buttons in command order 1000-1014 (switch at 1008:1efd), which is also the order of
/// their cells in the table at code 1000:0146.
const BUTTONS: [(Button, Icon); 15] = [
    (Button::Test(Test::Geometry), Icon::Geometry),
    (Button::Test(Test::Brightness), Icon::Brightness),
    (Button::Test(Test::HighVoltage), Icon::HighVoltage),
    (Button::Web, Icon::Web),
    (Button::Test(Test::Colors), Icon::Colors),
    (Button::ControlPanel, Icon::ControlPanel),
    (Button::Help, Icon::Help),
    (Button::Test(Test::Convergence), Icon::Convergence),
    (Button::Test(Test::Focus), Icon::Focus),
    (Button::Test(Test::Resolution), Icon::Resolution),
    (Button::Test(Test::Moire), Icon::Moire),
    (Button::Test(Test::Readability), Icon::Readability),
    (Button::Test(Test::Jitter), Icon::Jitter),
    (Button::Sound, Icon::Sound),
    (Button::Quit, Icon::Quit),
];

/// Demo order: commands 1000, 1001, 1002, 1004, 1007-1013 (word table at code 1000:0068).
const DEMO: [Button; 11] = [
    Button::Test(Test::Geometry),
    Button::Test(Test::Brightness),
    Button::Test(Test::HighVoltage),
    Button::Test(Test::Colors),
    Button::Test(Test::Convergence),
    Button::Test(Test::Focus),
    Button::Test(Test::Resolution),
    Button::Test(Test::Moire),
    Button::Test(Test::Readability),
    Button::Test(Test::Jitter),
    Button::Sound,
];

fn button_label(b: Button, l: &Labels) -> String {
    match b {
        Button::Test(t) => test_name(t, l).to_string(),
        Button::Sound => l.sound.to_string(),
        Button::Help => l.help.to_string(),
        Button::ControlPanel => l.control_panel.to_string(),
        Button::Web => WEB_ADDRESS.to_string(),
        Button::Quit => l.quit.to_string(),
    }
}

fn test_name(t: Test, l: &Labels) -> &'static str {
    match t {
        Test::Geometry => l.geometry,
        Test::Brightness => l.brightness,
        Test::HighVoltage => l.high_voltage,
        Test::Colors => l.colors,
        Test::Convergence => l.convergence,
        Test::Focus => l.focus,
        Test::Resolution => l.resolution,
        Test::Moire => l.moire,
        Test::Readability => l.readability,
        Test::Jitter => l.jitter,
    }
}

struct Options {
    screenshot: Option<PathBuf>,
    start_test: Option<(Test, usize)>,
    show_help: bool,
    lang: Option<Lang>,
}

fn parse_args() -> Options {
    let mut o = Options { screenshot: None, start_test: None, show_help: false, lang: None };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    let mut alt = 0;
    let mut test = None;
    while i < args.len() {
        match args[i].as_str() {
            "--screenshot" if i + 1 < args.len() => {
                o.screenshot = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--test" if i + 1 < args.len() => {
                let n = args[i + 1].to_lowercase();
                test = patterns::PATTERN_TESTS.iter().copied().find(|t| format!("{t:?}").to_lowercase() == n);
                i += 1;
            }
            "--alt" if i + 1 < args.len() => {
                alt = args[i + 1].parse().unwrap_or(0);
                i += 1;
            }
            "--help-popup" => o.show_help = true,
            "--lang" if i + 1 < args.len() => {
                o.lang = Lang::from_code(&args[i + 1].to_lowercase());
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    o.start_test = test.map(|t| (t, alt % t.alternatives()));
    o
}

fn settings_path() -> Option<PathBuf> {
    Some(platform::settings_dir()?.join("settings.txt"))
}

fn load_language() -> Option<Lang> {
    let path = settings_path()?;
    // Settings saved before the rename to MTEST lived in a "MonitorTest" folder.
    let old = path.parent()?.with_file_name("MonitorTest").join("settings.txt");
    let text = std::fs::read_to_string(&path).or_else(|_| std::fs::read_to_string(old)).ok()?;
    text.lines().find_map(|l| l.strip_prefix("lang=")).and_then(|c| Lang::from_code(c.trim()))
}

fn save_language(lang: Lang) {
    if let Some(p) = settings_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, format!("lang={}\n", lang.labels().code));
    }
}

enum Screen {
    Main,
    Pattern { test: Test, alt: usize },
}

struct Popup {
    title: String,
    body: String,
}

struct App {
    lang: Lang,
    choosing_language: Option<Lang>,
    screen: Screen,
    demo: Option<(usize, Instant)>,
    popup: Option<Popup>,
    texture: Option<(TextureHandle, (Option<Test>, usize, usize, usize, u64))>,
    last_pointer_move: Option<Instant>,
    last_pointer_pos: Option<egui::Pos2>,
    entered: Instant,
    screenshot: Option<PathBuf>,
    frames: u64,
    shot_requested: bool,
    shot: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<ColorImage>>>>,
    fullscreen: bool,
    /// Button to focus when the main screen shows again.
    focus_button: Option<usize>,
    /// Bitmap 152 as a texture: the glow behind the main-screen buttons.
    glow: Option<TextureHandle>,
}

impl App {
    fn new(opts: Options) -> Self {
        let stored = opts.lang.or_else(load_language);
        let screenshot_mode = opts.screenshot.is_some();
        let mut app = App {
            lang: stored.unwrap_or(Lang::English),
            choosing_language: if stored.is_none() && !screenshot_mode { Some(Lang::English) } else { None },
            screen: Screen::Main,
            demo: None,
            popup: None,
            texture: None,
            last_pointer_move: None,
            last_pointer_pos: None,
            entered: Instant::now(),
            screenshot: opts.screenshot,
            frames: 0,
            shot_requested: false,
            shot: Default::default(),
            fullscreen: true,
            focus_button: None,
            glow: None,
        };
        if let Some((t, alt)) = opts.start_test {
            app.screen = Screen::Pattern { test: t, alt };
            if opts.show_help {
                app.popup = Some(app.help_popup(t.topic(alt)));
            }
        } else if opts.show_help {
            app.popup = Some(app.help_popup(help::Topic::Main));
        }
        app
    }

    fn texts(&self) -> &'static help::Texts {
        help::texts(self.lang)
    }

    /// Help in the chosen language.
    fn help_popup(&self, topic: help::Topic) -> Popup {
        let (title, body) = self.texts().topic(topic);
        let title = if topic == help::Topic::Main { format!("{title} {VERSION}") } else { title.to_string() };
        Popup { title, body: body.into() }
    }

    fn labels(&self) -> &'static Labels {
        self.lang.labels()
    }

    /// The window stays full screen without a frame or title on every screen, as the original
    /// fills the whole display; this only re-asserts it if the system took it away.
    fn set_fullscreen(&mut self, ctx: &egui::Context, on: bool) {
        if on && !self.fullscreen {
            self.fullscreen = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }
    }

    /// Tests draw in the main window at its current size (1008:1f20).
    fn open_test(&mut self, _ctx: &egui::Context, test: Test, alt: usize) {
        self.screen = Screen::Pattern { test, alt };
        self.entered = Instant::now();
        self.popup = None;
    }

    /// Leaves a test for the main screen; `stop_demo` also ends demo mode.
    fn back_to_main(&mut self, ctx: &egui::Context, stop_demo: bool) {
        if let Screen::Pattern { test, .. } = self.screen {
            self.focus_button = BUTTONS.iter().position(|(b, _)| *b == Button::Test(test));
        }
        self.screen = Screen::Main;
        if stop_demo {
            self.demo = None;
        }
        self.popup = None;
        self.set_fullscreen(ctx, false);
    }

    /// Next alternative; at the last one nothing changes (1008:7208). Returns false then.
    fn next_alt(&mut self) -> bool {
        if let Screen::Pattern { test, alt } = self.screen {
            if alt + 1 < test.alternatives() {
                self.screen = Screen::Pattern { test, alt: alt + 1 };
                return true;
            }
        }
        false
    }

    /// Previous alternative; at the first one nothing changes (1008:7257).
    fn prev_alt(&mut self) {
        if let Screen::Pattern { test, alt } = self.screen {
            if alt > 0 {
                self.screen = Screen::Pattern { test, alt: alt - 1 };
            }
        }
    }

    fn press(&mut self, ctx: &egui::Context, b: Button) {
        match b {
            Button::Test(t) => self.open_test(ctx, t, 0),
            Button::Sound => {
                if !sound::play() {
                    self.popup = Some(Popup {
                        title: self.labels().sound.to_string(),
                        body: self.texts().sound_failed.into(),
                    });
                }
            }
            Button::Help => {
                self.popup = Some(match self.screen {
                    Screen::Main => self.help_popup(help::Topic::Main),
                    Screen::Pattern { test, alt } => self.help_popup(test.topic(alt)),
                });
            }
            Button::ControlPanel => {
                if !platform::open_display_settings() {
                    self.popup = Some(Popup {
                        title: self.labels().control_panel.to_string(),
                        body: self.texts().display_failed.into(),
                    });
                }
            }
            Button::Web => {
                if !platform::open(WEB_ADDRESS) {
                    self.popup = Some(Popup { title: "GitHub".into(), body: self.texts().web_failed.replace("{url}", WEB_ADDRESS) });
                }
            }
            Button::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    /// "%d x %d pixels, %s" with the window's size in pixels (1000:1134-1161).
    fn screen_info(&self, w: usize, h: usize) -> String {
        self.labels().info(w as u32, h as u32, 32)
    }

    /// Keyboard (1008:27b0).
    fn handle_keys(&mut self, ctx: &egui::Context) {
        let ctrl_alt = Modifiers { alt: true, ctrl: true, ..Default::default() };
        // In demo mode keys are swallowed; Esc stops it (1008:281b -> 1008:206e).
        if self.demo.is_some() {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                self.back_to_main(ctx, true);
            }
            return;
        }
        // Ctrl+Alt+D (without Shift) starts the demo (1008:2832 -> 1008:1fba).
        if ctx.input_mut(|i| i.consume_key(ctrl_alt, Key::D)) {
            self.demo = Some((0, Instant::now()));
            return;
        }
        // Ctrl+Alt+digit: window width digit*10 % of the screen (0 = 100 %), a multiple of 16,
        // full height, at the top left (1008:2869-28e1, MoveWindow).
        let digits = [Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];
        for (d, key) in digits.iter().enumerate() {
            if ctx.input_mut(|i| i.consume_key(ctrl_alt, *key)) {
                self.set_width_percent(ctx, if d == 0 { 100 } else { d as i64 * 10 });
                return;
            }
        }
        // F1: help for what is on screen; Shift+F1: the contents (1008:28e9).
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
            self.press(ctx, Button::Help);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::F1)) {
            self.popup = Some(self.help_popup(help::Topic::Main));
        }
        if self.popup.is_some() && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.popup = None;
            return;
        }
        // In a test: Page Down next (closing after the last), Page Up previous, Esc close
        // (1008:296c). Space and the arrow keys are this app's additions.
        if let Screen::Pattern { .. } = self.screen {
            let next = ctx.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::PageDown) || i.consume_key(Modifiers::NONE, Key::Space) || i.consume_key(Modifiers::NONE, Key::ArrowRight)
            });
            let prev = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::PageUp) || i.consume_key(Modifiers::NONE, Key::ArrowLeft));
            let close = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            if close {
                self.back_to_main(ctx, true);
            } else if next {
                if !self.next_alt() {
                    self.back_to_main(ctx, true);
                }
            } else if prev {
                self.prev_alt();
            }
        }
    }

    /// Ctrl+Alt+digit: the window's width as a share of the screen (1008:28b7).
    fn set_width_percent(&mut self, ctx: &egui::Context, pct: i64) {
        if self.screenshot.is_some() {
            return;
        }
        let ppp = ctx.pixels_per_point();
        let Some(screen) = ctx.input(|i| i.viewport().monitor_size) else { return };
        let (sw, sh) = ((screen.x * ppp).round() as i64, (screen.y * ppp).round() as i64);
        if pct >= 100 {
            self.fullscreen = false;
            self.set_fullscreen(ctx, true);
            return;
        }
        let w = patterns::mul_div(pct, sw, 100) & !0xf;
        self.fullscreen = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos2(0.0, 0.0)));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(w as f32 / ppp, sh as f32 / ppp)));
    }

    /// Demo tick (1000:11e8, 1008:17bc): on the main screen open the next item of the list;
    /// in a test show its next alternative, or close it after the last one.
    fn step_demo(&mut self, ctx: &egui::Context) {
        let Some((idx, since)) = self.demo else { return };
        ctx.request_repaint_after(Duration::from_millis(200));
        if since.elapsed().as_secs_f32() < DEMO_STEP_SECS {
            return;
        }
        let mut next_idx = idx;
        match self.screen {
            Screen::Pattern { .. } => {
                if !self.next_alt() {
                    self.back_to_main(ctx, false);
                }
            }
            Screen::Main => {
                let item = DEMO[idx];
                next_idx = (idx + 1) % DEMO.len();
                self.press(ctx, item);
            }
        }
        self.demo = Some((next_idx, Instant::now()));
    }

    /// The main screen: the test card of 1000:09ca with the buttons in their grid cells.
    fn main_screen(&mut self, root: &mut egui::Ui) {
        let ctx = &root.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let mut pressed = None;
        let l = self.labels();
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::BLACK)).show(root, |ui| {
            let full = ui.max_rect();
            let pw = (full.width() * ppp).round() as usize;
            let ph = (full.height() * ppp).round() as usize;
            self.ensure_texture(ctx, None, 0, pw, ph, 0);
            let p = ui.painter().clone();
            if let Some((tex, _)) = &self.texture {
                p.image(tex.id(), full, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
            let lay = patterns::card_layout(pw.max(64) as i64, ph.max(64) as i64);
            let to_rect = |r: patterns::R| px_rect(full, ppp, r);
            for (cx, cy, rw, rh) in corner_discs(&lay) {
                let c = pos2(full.left() + cx / ppp, full.top() + cy / ppp);
                p.circle_filled(c, (rw.min(rh) * 0.105) / ppp, Color32::WHITE);
            }
            // Name strip (the original shows its maker's logo, bitmap 131, here).
            let logo = to_rect(lay.logo);
            p.text(logo.center(), Align2::CENTER_CENTER, "MTEST", FontId::proportional((logo.height() * 0.32).max(8.0)), Color32::WHITE);
            // "%d%%" in each grey step: white below 50 %, black from 50 % (1000:0fd1).
            for (pct, r) in &lay.steps {
                let col = if *pct < 50 { Color32::WHITE } else { Color32::BLACK };
                p.text(to_rect(*r).center(), Align2::CENTER_CENTER, format!("{pct}%"), FontId::proportional(TEXT_10PT), col);
            }
            // Screen mode, white, centred (DrawText 0x825).
            p.text(to_rect(lay.info).center(), Align2::CENTER_CENTER, self.screen_info(pw, ph), FontId::proportional(TEXT_10PT), Color32::WHITE);
            let glow = self
                .glow
                .get_or_insert_with(|| ctx.load_texture("button-glow", button_glow_image(), TextureOptions::LINEAR))
                .id();
            for (k, (b, icon)) in BUTTONS.iter().enumerate() {
                let r = to_rect(lay.buttons[k]);
                let resp = ui.interact(r, ui.make_persistent_id(("btn", k)), Sense::click());
                if self.focus_button == Some(k) {
                    resp.request_focus();
                }
                let lift = if resp.is_pointer_button_down_on() { vec2(1.0, 1.0) } else { vec2(0.0, 0.0) };
                draw_button(&p, r, lift, *icon, glow);
                if resp.hovered() || resp.has_focus() {
                    p.rect_stroke(r, 4.0, Stroke::new(2.0, patterns::ORANGE), egui::StrokeKind::Inside);
                }
                // Names show as tooltips, as in the original (its CThtTipWnd class).
                if resp.on_hover_text(button_label(*b, l)).clicked() {
                    pressed = Some(*b);
                }
            }
            // Language choice at the bottom, just left of the bottom-right corner circle's box
            // (2 x 2 cells), so it touches no circle (the original asks only at first start).
            let corner_w = lay.grid.x(2) as f32 / ppp;
            let combo_rect = Rect::from_min_size(pos2(full.right() - corner_w - 150.0, full.bottom() - 28.0), vec2(140.0, 22.0));
            let mut lang = self.lang;
            ui.scope_builder(egui::UiBuilder::new().max_rect(combo_rect), |ui| {
                egui::ComboBox::from_id_salt("lang").selected_text(lang.labels().name).width(134.0).show_ui(ui, |ui| {
                    for cand in i18n::ALL {
                        ui.selectable_value(&mut lang, cand, cand.labels().name);
                    }
                });
            });
            if lang != self.lang {
                self.lang = lang;
                if self.screenshot.is_none() {
                    save_language(lang);
                }
            }
        });
        self.focus_button = None;
        if let Some(b) = pressed {
            self.press(ctx, b);
        }
    }

    /// Draws a screen into the shared texture when it isn't there yet.
    fn ensure_texture(&mut self, ctx: &egui::Context, test: Option<Test>, alt: usize, pw: usize, ph: usize, phase: u64) {
        let key = (test, alt, pw, ph, phase);
        if self.texture.as_ref().map(|(_, k)| *k == key).unwrap_or(false) {
            return;
        }
        let dpi = 96.0 * ctx.pixels_per_point() as f64;
        let img: ColorImage = match test {
            Some(t) => patterns::render(t, alt, pw, ph, phase, dpi),
            None => patterns::render_card(pw, ph),
        };
        match &mut self.texture {
            Some((tex, k)) => {
                tex.set(img, TextureOptions::NEAREST);
                *k = key;
            }
            None => self.texture = Some((ctx.load_texture("screen", img, TextureOptions::NEAREST), key)),
        }
    }

    fn pattern_screen(&mut self, root: &mut egui::Ui, test: Test, alt: usize) {
        let ctx = &root.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let phase = if test.animated() { (self.entered.elapsed().as_secs_f64() / patterns::HV_SWITCH_SECS) as u64 } else { 0 };
        if test.animated() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        let mut action: Option<Button> = None;
        let mut clicked_next = false;
        let mut clicked_prev = false;
        let mut go_back = false;
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::BLACK)).show(root, |ui| {
            let full = ui.max_rect();
            let pw = (full.width() * ppp).round() as usize;
            let ph = (full.height() * ppp).round() as usize;
            self.ensure_texture(ctx, Some(test), alt, pw, ph, phase);
            let p = ui.painter().clone();
            if let Some((tex, _)) = &self.texture {
                p.image(tex.id(), full, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
            overlay(&p, full, test, alt, ppp, self.texts().reading);

            // Pointer: show the button bar when the mouse moves.
            let pos = ctx.input(|i| i.pointer.hover_pos());
            if pos.is_some() && pos != self.last_pointer_pos {
                // The first position seen is where the cursor already was, not a movement.
                let moved = self.last_pointer_pos.is_some();
                self.last_pointer_pos = pos;
                if moved {
                    self.last_pointer_move = Some(Instant::now());
                }
            }
            let bar_visible = SHOW_BAR && self.demo.is_none() && self.screenshot.is_none() && self.last_pointer_move.map(|t| t.elapsed().as_secs_f32() < BAR_HIDE_SECS).unwrap_or(false);
            let bar = bar_rect(full);
            let bg = ui.interact(full, ui.make_persistent_id("pattern"), Sense::click());
            let mut over_bar = false;
            if bar_visible {
                ctx.request_repaint_after(Duration::from_millis(250));
                p.rect_filled(bar, 8.0, Color32::from_rgba_unmultiplied(40, 40, 40, 230));
                let n = BUTTONS.len();
                let bw = (bar.width() - 12.0) / n as f32;
                for (k, (b, icon)) in BUTTONS.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(bar.left() + 6.0 + k as f32 * bw, bar.top() + 6.0), vec2(bw - 4.0, bar.height() - 12.0));
                    let resp = ui.interact(r, ui.make_persistent_id(("bar", k)), Sense::click());
                    if resp.hovered() {
                        over_bar = true;
                        p.rect_stroke(r.expand(2.0), 8.0, Stroke::new(2.0, Color32::from_rgb(90, 160, 255)), egui::StrokeKind::Outside);
                    }
                    if *b == Button::Test(test) {
                        p.rect_stroke(r.expand(2.0), 8.0, Stroke::new(2.0, Color32::from_rgb(255, 200, 0)), egui::StrokeKind::Outside);
                    }
                    icons::draw(&p, r, *icon);
                    if resp.on_hover_text(button_label(*b, self.labels())).clicked() {
                        action = Some(*b);
                    }
                }
                if pos.map(|q| bar.contains(q)).unwrap_or(false) {
                    over_bar = true;
                }
            }
            if !over_bar && self.popup.is_none() {
                let (left_down, right_down) = ctx.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
                if (bg.clicked() && right_down) || (bg.clicked_by(PointerButton::Secondary) && left_down) {
                    go_back = true;
                } else if bg.clicked() {
                    clicked_next = true;
                } else if bg.clicked_by(PointerButton::Secondary) {
                    clicked_prev = true;
                }
            }
            // A short hint when a test opens.
            let since = self.entered.elapsed().as_secs_f32();
            if since < 3.0 && self.demo.is_none() && self.screenshot.is_none() {
                ctx.request_repaint_after(Duration::from_millis(100));
                let tx = self.texts();
                let hint = format!(
                    "{}  ({}/{})   ·   {}   ·   {}   ·   {}",
                    test_name(test, self.labels()),
                    alt + 1,
                    test.alternatives(),
                    tx.hint_next,
                    tx.hint_prev,
                    tx.hint_back
                );
                let r = Rect::from_center_size(pos2(full.center().x, full.top() + 28.0), vec2(760.0, 30.0));
                p.rect_filled(r, 6.0, Color32::from_rgba_unmultiplied(0, 0, 0, 200));
                p.text(r.center(), Align2::CENTER_CENTER, hint, FontId::proportional(14.0), Color32::WHITE);
            }
        });
        if clicked_next && !self.next_alt() {
            go_back = true;
        }
        if clicked_prev {
            self.prev_alt();
        }
        if go_back {
            self.back_to_main(ctx, true);
        } else if let Some(b) = action {
            if let Button::Test(t) = b {
                if t == test {
                    self.next_alt();
                } else {
                    self.open_test(ctx, t, 0);
                }
            } else {
                self.press(ctx, b);
            }
        }
    }

    fn language_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut choice) = self.choosing_language else { return };
        let mut done = None;
        egui::Window::new(format!("MTEST {VERSION}"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(help::texts(choice).choose_language);
                ui.add_space(4.0);
                egui::Grid::new("langs").num_columns(2).spacing(vec2(30.0, 4.0)).show(ui, |ui| {
                    for (k, cand) in i18n::ALL.iter().enumerate() {
                        ui.radio_value(&mut choice, *cand, cand.labels().name);
                        if k % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
                ui.separator();
                ui.label(help::texts(choice).asked_once);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(choice.labels().ok).clicked() {
                        done = Some(true);
                    }
                    if ui.button(choice.labels().quit).clicked() {
                        done = Some(false);
                    }
                });
            });
        self.choosing_language = Some(choice);
        match done {
            Some(true) => {
                self.lang = choice;
                save_language(choice);
                self.choosing_language = None;
            }
            Some(false) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            None => {}
        }
    }

    fn popup_window(&mut self, ctx: &egui::Context) {
        let mut close = false;
        if let Some(pop) = &self.popup {
            let ok = self.labels().ok;
            egui::Window::new(pop.title.as_str())
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_max_width(420.0);
                    ui.label(pop.body.as_str());
                    ui.add_space(8.0);
                    if ui.button(ok).clicked() {
                        close = true;
                    }
                });
        }
        if close {
            self.popup = None;
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames >= SCREENSHOT_FRAMES && !self.shot_requested {
            self.shot_requested = true;
            let slot = self.shot.clone();
            let waker = ctx.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |img| {
                *slot.lock().unwrap() = Some(img);
                waker.request_repaint();
            })));
        }
        let shot = self.shot.lock().unwrap().take();
        if let Some(img) = shot {
            let [w, h] = img.size;
            let mut bytes = Vec::with_capacity(w * h * 4);
            for c in &img.pixels {
                bytes.extend_from_slice(&[c.r(), c.g(), c.b(), 255]);
            }
            if let Some(dir) = path.parent() {
                if !dir.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(dir);
                }
            }
            if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                if let Err(e) = buf.save(&path) {
                    eprintln!("could not save {}: {e}", path.display());
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn bar_rect(full: Rect) -> Rect {
    let w = (full.width() * 0.9).min(15.0 * 62.0);
    let h = w / 15.0 * 0.78 + 12.0;
    Rect::from_center_size(pos2(full.center().x, full.bottom() - h / 2.0 - 16.0), vec2(w, h))
}

/// Centres and sizes (in pixels) of the four corner circles' boxes.
fn corner_discs(lay: &patterns::CardLayout) -> [(f32, f32, f32, f32); 4] {
    let g = &lay.grid;
    let (bw, bh) = (g.x(2) as f32, g.y(2) as f32);
    let (w, h) = (g.w as f32, g.h as f32);
    [
        (bw / 2.0, bh / 2.0, bw, bh),
        (w - bw / 2.0, bh / 2.0, bw, bh),
        (w - bw / 2.0, h - bh / 2.0, bw, bh),
        (bw / 2.0, h - bh / 2.0, bw, bh),
    ]
}

/// Bitmap 152 (64 x 65 greys) as an image.
fn button_glow_image() -> ColorImage {
    let hex = bitmaps::BUTTON_GLOW_HEX.as_bytes();
    let px = (0..bitmaps::BUTTON_GLOW_W * bitmaps::BUTTON_GLOW_H)
        .map(|k| {
            let v = u8::from_str_radix(std::str::from_utf8(&hex[2 * k..2 * k + 2]).unwrap_or("00"), 16).unwrap_or(0);
            Color32::from_gray(v)
        })
        .collect();
    ColorImage::new([bitmaps::BUTTON_GLOW_W, bitmaps::BUTTON_GLOW_H], px)
}

/// One main-screen button: bitmap 152 stretched over the cell (loaded at 1008:2218), the
/// picture small in the middle, and a light grey frame round the cell.
fn draw_button(p: &egui::Painter, r: Rect, lift: egui::Vec2, icon: Icon, glow: egui::TextureId) {
    p.image(glow, r.translate(lift), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    let c = r.center() + lift;
    let ih = r.height() * 0.3;
    let iw = if r.width() > r.height() * 1.5 { r.width() * 0.4 } else { ih * 1.35 };
    icons::draw(p, Rect::from_center_size(c, vec2(iw, ih)), icon);
    p.rect_stroke(r, 0.0, Stroke::new(2.0, Color32::from_gray(160)), egui::StrokeKind::Inside);
}

/// A pixel rectangle of the screen as a rectangle in points.
fn px_rect(full: Rect, ppp: f32, r: patterns::R) -> Rect {
    Rect::from_min_max(
        pos2(full.left() + r.l as f32 / ppp, full.top() + r.t as f32 / ppp),
        pos2(full.left() + r.r as f32 / ppp, full.top() + r.b as f32 / ppp),
    )
}

/// Text drawn over the patterns: step labels and the readability text.
fn overlay(p: &egui::Painter, full: Rect, test: Test, alt: usize, ppp: f32, reading: &str) {
    let pw = (full.width() * ppp).round().max(64.0) as i64;
    let ph = (full.height() * ppp).round().max(64.0) as i64;
    match (test, alt) {
        (Test::Brightness, 0) => {
            // "%d%%": white on the 1-9 % steps; below them white under 50 %, black from 50 % (1008:7af8, 7bdc).
            for (pct, r) in patterns::brightness_blocks(pw, ph) {
                let col = if pct < 50 { Color32::WHITE } else { Color32::BLACK };
                p.text(px_rect(full, ppp, r).center(), Align2::CENTER_CENTER, format!("{pct}%"), FontId::proportional(TEXT_10PT), col);
            }
        }
        (Test::Readability, _) => {
            // Five boxes a quarter of the screen each, filled with 8 pt text (1008:887a).
            let fg = if alt == 0 { Color32::BLACK } else { Color32::WHITE };
            for (r, ha, va) in patterns::readability_boxes(pw, ph) {
                let rect = px_rect(full, ppp, r);
                let font = FontId::proportional(TEXT_8PT);
                // Row height of this language's text (Thai lines are taller than Latin ones).
                let sample: String = reading.chars().take(24).collect();
                let row = p.layout_no_wrap(sample, font.clone(), fg).size().y.max(1.0);
                let mut rows = ((rect.height() / row).floor() as usize).max(1);
                // Thai has no spaces between words, so it may break anywhere.
                let anywhere = reading.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c));
                let mut text = String::new();
                loop {
                    text.push_str(reading);
                    let g = p.layout(text.clone(), font.clone(), fg, rect.width());
                    if g.rows.len() > rows || text.len() > 65000 {
                        break;
                    }
                }
                // Only whole lines, so no line is cut at the box's edge or the screen's.
                let room = rect.height().min(full.bottom() - rect.top());
                let make = |rows: usize| {
                    let mut job = egui::text::LayoutJob::simple(text.clone(), font.clone(), fg, rect.width());
                    job.wrap.max_rows = rows;
                    job.wrap.break_anywhere = anywhere;
                    job.wrap.overflow_character = None;
                    job
                };
                let mut job = make(rows);
                while rows > 1 && p.layout_job(job.clone()).size().y > room {
                    rows -= 1;
                    job = make(rows);
                }
                job.halign = match ha {
                    0 => egui::Align::LEFT,
                    1 => egui::Align::Center,
                    _ => egui::Align::RIGHT,
                };
                let galley = p.layout_job(job);
                let x = match ha {
                    0 => rect.left(),
                    1 => rect.center().x,
                    _ => rect.right(),
                };
                let y = match va {
                    0 => rect.top(),
                    1 => rect.center().y - galley.size().y / 2.0,
                    _ => rect.bottom() - galley.size().y,
                };
                let y = y.max(rect.top()).min((full.bottom() - galley.size().y).max(rect.top()));
                p.galley(pos2(x, y), galley, fg);
            }
        }
        _ => {}
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        if self.choosing_language.is_none() {
            self.handle_keys(ctx);
        }
        self.step_demo(ctx);
        match self.screen {
            Screen::Main => self.main_screen(ui),
            Screen::Pattern { test, alt } => self.pattern_screen(ui, test, alt),
        }
        self.popup_window(ctx);
        self.language_dialog(ctx);
        self.handle_screenshot(ctx);
    }
}

fn main() -> eframe::Result {
    let opts = parse_args();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("MTEST {VERSION}"))
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native("MTEST", options, Box::new(move |cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        platform::install_thai_font(&cc.egui_ctx);
        Ok(Box::new(App::new(opts)))
    }))
}
