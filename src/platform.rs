//! What differs between Windows, macOS and Linux: where settings live, how the display settings
//! and web pages are opened, and which system font draws Thai.

use std::path::PathBuf;
use std::process::Command;

/// The folder the language choice is saved in.
pub fn settings_dir() -> Option<PathBuf> {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from).or_else(home);
    #[cfg(target_os = "macos")]
    let base = home().map(|h| h.join("Library").join("Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home().map(|h| h.join(".config")));
    #[cfg(not(any(windows, unix)))]
    let base = home();
    base.map(|b| b.join("MTEST"))
}

/// Opens a web address or system location with the desktop's default handler.
pub fn open(target: &str) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("cmd").args(["/C", "start", "", target]).creation_flags(CREATE_NO_WINDOW).spawn().is_ok()
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(target).spawn().is_ok()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(target).spawn().is_ok()
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = target;
        false
    }
}

/// Opens the system's display settings (the original's "To control panel/display").
pub fn open_display_settings() -> bool {
    #[cfg(windows)]
    {
        open("ms-settings:display")
    }
    #[cfg(target_os = "macos")]
    {
        open("x-apple.systempreferences:com.apple.Displays-Settings.extension")
            || open("/System/Library/PreferencePanes/Displays.prefPane")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // GNOME, KDE Plasma, Xfce, Cinnamon, MATE, then the generic X tool.
        let tries: [(&str, &[&str]); 6] = [
            ("gnome-control-center", &["display"]),
            ("systemsettings", &["kcm_kscreen"]),
            ("xfce4-display-settings", &[]),
            ("cinnamon-settings", &["display"]),
            ("mate-display-properties", &[]),
            ("arandr", &[]),
        ];
        tries.iter().any(|(prog, args)| Command::new(prog).args(*args).spawn().is_ok())
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

/// Plays a WAV file kept in memory, without waiting. On macOS and Linux it's written to a
/// temporary file and handed to the system's player.
#[cfg(unix)]
pub fn play_wav(wav: &[u8]) -> bool {
    let path = std::env::temp_dir().join("mtest-sound.wav");
    if std::fs::write(&path, wav).is_err() {
        return false;
    }
    #[cfg(target_os = "macos")]
    let players: [(&str, &[&str]); 1] = [("afplay", &[])];
    #[cfg(not(target_os = "macos"))]
    let players: [(&str, &[&str]); 4] = [("paplay", &[]), ("pw-play", &[]), ("aplay", &["-q"]), ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"])];
    players.iter().any(|(prog, args)| Command::new(prog).args(*args).arg(&path).spawn().is_ok())
}

/// Font files that can draw Thai, per system, best first.
fn thai_font_candidates() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        let fonts = std::env::var_os("WINDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\Windows")).join("Fonts");
        for f in ["LeelawUI.ttf", "leelawui.ttf", "tahoma.ttf", "Tahoma.ttf", "cordia.ttc", "angsana.ttc"] {
            v.push(fonts.join(f));
        }
    }
    #[cfg(target_os = "macos")]
    {
        for f in [
            "/System/Library/Fonts/Supplemental/Ayuthaya.ttf",
            "/System/Library/Fonts/Supplemental/Krungthep.ttf",
            "/System/Library/Fonts/Supplemental/Silom.ttf",
            "/System/Library/Fonts/Supplemental/Sathu.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
        ] {
            v.push(PathBuf::from(f));
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for f in [
            "/usr/share/fonts/truetype/noto/NotoSansThai-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansThai-Regular.ttf",
            "/usr/share/fonts/google-noto/NotoSansThai-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansThai-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansThaiUI-Regular.ttf",
            "/usr/share/fonts/truetype/tlwg/Loma.ttf",
            "/usr/share/fonts/truetype/tlwg/Garuda.ttf",
            "/usr/share/fonts/tlwg/Loma.otf",
            "/usr/share/fonts/TTF/Loma.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ] {
            v.push(PathBuf::from(f));
        }
    }
    v
}

/// Adds a system font that draws Thai behind egui's own fonts. Returns its path when found.
pub fn install_thai_font(ctx: &eframe::egui::Context) -> Option<PathBuf> {
    use eframe::egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;
    let (path, bytes) = thai_font_candidates().into_iter().find_map(|p| std::fs::read(&p).ok().map(|b| (p, b)))?;
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("thai".to_owned(), Arc::new(FontData::from_owned(bytes)));
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push("thai".to_owned());
    }
    ctx.set_fonts(fonts);
    Some(path)
}
