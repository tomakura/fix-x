# fix-x

![fix-x logo](assets/logo.png)

日本語:
`x.com` の投稿URLを、クリップボードにコピーした瞬間に `fxtwitter.com` または `vxtwitter.com` に自動変換する Windows 常駐アプリです。

English:
`fix-x` is a lightweight Windows tray app that automatically rewrites copied `x.com` post URLs to `fxtwitter.com` or `vxtwitter.com`.

## Features

- Tray resident app with a small settings window
- Automatic rewrite for `x.com/<user>/status/<id>` URLs only
- Rewrite target switch: `fxtwitter.com` / `vxtwitter.com`
- Enable / disable automatic rewrite
- Launch on Windows startup toggle
- UI language: `Auto` / `日本語` / `English`
- Native installer executable
- Config stored at `%APPDATA%\fix-x\config.json`

## Behavior

- Rewrites only when the clipboard contains a single URL string
- Keeps query strings and fragments
- Ignores non-post URLs, embedded URLs in text, and already rewritten URLs

## Build

```powershell
cargo build --release
```

Output:

- `target\release\fix-x.exe`

## Tests

```powershell
cargo clippy --all-targets -- -D warnings
cargo test
```

## Logo Assets

- `assets\103d668e-2545-49b6-bdfa-2708d540447e.jpg`
- `assets\logo.png`
- `assets\logo.ico`

Regenerate `logo.png` and `logo.ico` from the source image:

```powershell
python .\tools\generate_logo.py
```

## Installer

Build the native installer executable:

```powershell
powershell -ExecutionPolicy Bypass -File .\installer\build-installer.ps1
```

Output:

- `dist\fix-x-installer.exe`

The installer copies `fix-x.exe` into `%LOCALAPPDATA%\Programs\fix-x`, writes an uninstaller script, creates Start Menu shortcuts, and launches the app after installation.

## License

MIT License
