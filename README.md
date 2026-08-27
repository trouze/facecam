# facecam

Tiny macOS app for floating your webcam feed on your desktop to capture in screen recordings.

A native Rust clone of [CamBubble](https://github.com/backnotprop/CamBubble): a circular webcam bubble that floats above everything — no menubar, no chrome, just your face in a draggable circle, always on top.

## Why

You want to record a demo or walkthrough with QuickTime (or any screen recorder) and show your face too — but you don't want to buy software. Open FaceCam, start your screen recording, done.

## Features

- 200×200 circular webcam bubble, floating window level, always on top
- Aspect-fill preview with mirroring (like a real mirror)
- Center Stage disabled so the camera uses its full wide field of view
- Drag the bubble anywhere (`movableByWindowBackground`)
- Right-click the bubble → **Quit FaceCam**, or Cmd+Q
- Pure Rust via `objc2` bindings to AppKit/AVFoundation — no WebView, no Electron

## Install

Install the latest release straight from your terminal (Apple Silicon + Intel):

```bash
curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | sh
```

That downloads the latest `FaceCam.app` from GitHub Releases and puts it in `/Applications`. To install elsewhere:

```bash
curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | INSTALL_DIR="$HOME/Applications" sh
```

Prefer doing it by hand?

```bash
curl -fsSL -o /tmp/FaceCam.app.zip https://github.com/trouze/facecam/releases/latest/download/FaceCam.app.zip
unzip -q -o /tmp/FaceCam.app.zip -d /tmp && cp -R /tmp/FaceCam.app /Applications/
```

New releases are published automatically: pushing a tag like `v0.1.1` runs the GitHub Action that builds a universal binary and attaches it to the release.

## Build

Requires macOS 13+, a camera, and Rust (`rustup`).

```bash
./build.sh --release
cp -r target/release/FaceCam.app /Applications/
```

`build.sh` compiles the binary, assembles `FaceCam.app` (with the camera usage description required by TCC), and ad-hoc signs it.

## Run

```bash
open target/release/FaceCam.app
```

Grant camera access when prompted, then drag the bubble wherever you want. Start your QuickTime screen recording and your face gets captured along with everything else.

## License

MIT
