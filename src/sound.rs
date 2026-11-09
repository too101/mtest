//! The stereo sound test: a tone that starts in the left speaker and ends in the right one.
//! Numbers measured from the original's WAVE resource 142 (see MECHANICS.md).

pub const SAMPLE_RATE: u32 = 22050;
pub const LENGTH_SECS: f32 = 1.71;
pub const TONE_HZ: f32 = 350.0;

/// A 16-bit stereo WAV file in memory.
pub fn make_wav() -> Vec<u8> {
    let frames = (SAMPLE_RATE as f32 * LENGTH_SECS) as usize;
    let mut data = Vec::with_capacity(frames * 4);
    for n in 0..frames {
        let t = n as f32 / SAMPLE_RATE as f32;
        let pan = n as f32 / frames as f32; // 0 = left, 1 = right
        // Soft start and end so the tone doesn't click.
        let edge = (t / 0.02).min((LENGTH_SECS - t) / 0.02).clamp(0.0, 1.0);
        let s = (2.0 * std::f32::consts::PI * TONE_HZ * t).sin() * 0.8 * edge;
        let left = s * (1.0 - pan * 0.9);
        let right = s * (0.1 + pan * 0.9);
        data.extend_from_slice(&((left * 32767.0) as i16).to_le_bytes());
        data.extend_from_slice(&((right * 32767.0) as i16).to_le_bytes());
    }
    let mut wav = Vec::with_capacity(44 + data.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&2u16.to_le_bytes()); // stereo
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
    wav.extend_from_slice(&data);
    wav
}

/// Plays the test sound without waiting. Returns false when it couldn't be played.
#[cfg(windows)]
pub fn play() -> bool {
    use std::sync::OnceLock;
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
    static WAV: OnceLock<Vec<u8>> = OnceLock::new();
    let wav = WAV.get_or_init(make_wav);
    // SAFETY: the buffer lives for the whole program (static), as SND_ASYNC needs.
    unsafe { PlaySoundW(wav.as_ptr() as *const u16, std::ptr::null_mut(), SND_MEMORY | SND_ASYNC | SND_NODEFAULT) != 0 }
}

#[cfg(unix)]
pub fn play() -> bool {
    crate::platform::play_wav(&make_wav())
}

#[cfg(not(any(windows, unix)))]
pub fn play() -> bool {
    false
}
