# facecam

[![release](https://img.shields.io/github/v/release/trouze/facecam?label=release)](https://github.com/trouze/facecam/releases)
[![build](https://img.shields.io/github/actions/workflow/status/trouze/facecam/release.yml?label=build)](https://github.com/trouze/facecam/actions/workflows/release.yml)
[![license](https://img.shields.io/badge/license-MIT-blue)](https://github.com/trouze/facecam)
[![rust](https://img.shields.io/badge/rust-2021-orange)](https://www.rust-lang.org)
[![platform](https://img.shields.io/badge/platform-macOS-lightgrey)](https://www.apple.com/macos)
[![size](https://img.shields.io/badge/binary-~360KB-success)](https://github.com/trouze/facecam/releases)

tiny macOS app for floating your webcam feed on your desktop to capture in screen recordings. small, minimal, fast, and written in rust.

## why

you want to record a demo or walkthrough with macOS' native screen recorder (cmd+shift+5) and skip the bloat of products like Loom. open facecam, start your screen recording, done.

## features

- displays your webcam feed on desktop in a bubble
- is a 360kb binary
- pure rust via `objc2` bindings to appkit/avfoundation. no webview, no electron
- resizable


## install

install the latest release straight from your terminal (apple silicon + intel):

```bash
curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | sh
```

that downloads the latest `facecam.app` from gitHub releases and puts it in `/Applications`. to install elsewhere:

```bash
curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | INSTALL_DIR="$HOME/Applications" sh
```

prefer doing it by hand?

```bash
curl -fsSL -o /tmp/FaceCam.app.zip https://github.com/trouze/facecam/releases/latest/download/FaceCam.app.zip
unzip -q -o /tmp/FaceCam.app.zip -d /tmp && cp -R /tmp/FaceCam.app /Applications/
```

there is no auto-updating functionality, to update just re-run the install command — it replaces the old version:

```bash
curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | sh
```

or if facecam is running, quit it first (right-click the bubble → quit facecam, or `pkill -x facecam`).


## build

requires macOS 13+, a camera, and rust (`rustup`).

```bash
./build.sh --release
cp -r target/release/FaceCam.app /Applications/
```

`build.sh` compiles the binary, assembles `FaceCam.app` (with the camera usage description required by TCC), and ad-hoc signs it.

## run

```bash
open target/release/FaceCam.app
```

grant camera access when prompted, then drag the bubble wherever you want. start your screen recording (cmd + shift + 5) and your face gets captured along with everything else.

## license

MIT
