# Building MTEST

## Ready-made programs

`dist/` holds:

- `mtest-windows-x86_64.exe`: Windows 10/11, 64-bit.
- `mtest-linux-x86_64`: Linux, 64-bit, glibc 2.31 or newer (Ubuntu 20.04+, Debian 11+, Fedora 32+...).
  Make it runnable with `chmod +x mtest-linux-x86_64`. It finds X11/Wayland and OpenGL on the
  system when it starts; the sound test uses `paplay`, `pw-play`, `aplay` or `ffplay`.

macOS programs can't be linked on Windows (Apple's system frameworks are only on a Mac). They are
built by `.github/workflows/build.yml` on GitHub: push this folder to
https://github.com/too101/mtest and open the run's **Artifacts** for `mtest-macos-arm64` (Apple
Silicon) and `mtest-macos-x86_64` (Intel), with Windows and Linux builds (both 32/64-bit) as well. Pushing a tag such
as `v.20261109` attaches all six to a GitHub release. On a Mac, the first start may need right click →
Open, because the program isn't signed.

## Building yourself

GitHub Actions also builds `mtest-linux-x86` for 32-bit x86 Linux using
`i686-unknown-linux-gnu` on Ubuntu 22.04. The workflow installs the i386
development libraries and a multilib compiler, and selects the i386 pkg-config
directory. The binary needs compatible 32-bit glibc, X11/Wayland and OpenGL
libraries on the destination system; on 64-bit Linux, install those 32-bit
runtime libraries too. Run `chmod +x mtest-linux-x86` before starting it.
This Ubuntu 22.04 build is not guaranteed to run on older distributions.

For Windows 32-bit, GitHub Actions produces `mtest-windows-x86.exe` using
`i686-pc-windows-msvc`. This build targets Windows 10 (32-bit or 64-bit);
32-bit support does not imply support for Windows XP or Windows 7.
To build it locally, install the Visual Studio C++ x86 build tools, then run:

```powershell
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

With Rust (https://rustup.rs): `cargo build --release`, giving `target/release/mtest`
(`mtest.exe` on Windows).

- **Windows** here uses the `x86_64-pc-windows-gnu` toolchain; `.cargo/config.toml` points rustc at
  MSYS2's `dlltool` (`C:/msys64/ucrt64/bin/dlltool.exe`). The MSVC toolchain needs nothing extra.
- **Linux from Windows**: `pip install ziglang`, `cargo install cargo-zigbuild`, then
  `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31`.
- **Linux natively**: install `libxkbcommon-dev libgl1-mesa-dev libwayland-dev libx11-dev
  libxcursor-dev libxrandr-dev libxi-dev` (Debian/Ubuntu names) and a Thai font such as
  `fonts-noto-core` or `fonts-thai-tlwg` if you use Thai.
- **macOS**: install the Xcode command line tools (`xcode-select --install`) and build as above.

## Settings

The language choice is kept in `settings.txt` under `%APPDATA%\MTEST` (Windows),
`~/Library/Application Support/MTEST` (macOS) or `~/.config/MTEST` (Linux). A choice saved by an
earlier version in a `MonitorTest` folder is still read.
