# fix-x

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

## Installer

Build the installer with Windows built-in IExpress:

```powershell
powershell -ExecutionPolicy Bypass -File .\installer\build-installer.ps1
```

Output:

- `dist\fix-x-installer.exe`

The installer copies `fix-x.exe` into `%LOCALAPPDATA%\Programs\fix-x`, creates Start Menu shortcuts, and launches the app after installation.

## License

MIT License
