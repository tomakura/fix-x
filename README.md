# fix-x

Windowsで常駐し、クリップボードにコピーした `x.com` の投稿URLを `fxtwitter.com` または `vxtwitter.com` に自動変換する軽量アプリです。

## 機能

- タスクトレイ常駐
- `x.com/<user>/status/<id>` 形式のURLを自動変換
- 変換先の切り替え
  - `fxtwitter.com`
  - `vxtwitter.com`
- 自動変換のオン・オフ
- Windows起動時の自動起動オン・オフ
- 設定を `%APPDATA%\\fix-x\\config.json` に保存

## 対象

変換対象はURL文字列そのものがクリップボードに入っている場合のみです。

- 対象: `https://x.com/example/status/1234567890`
- 対象外: `https://x.com/example`
- 対象外: `メモ https://x.com/example/status/1234567890`
- 対象外: すでに `fxtwitter.com` / `vxtwitter.com` のURL

クエリ文字列とフラグメントは維持されます。

## 使い方

1. アプリを起動します。
2. タスクトレイに `fix-x` のアイコンが表示されます。
3. トレイアイコンを左クリック、または右クリックして `Open Settings` を選ぶと設定画面を開けます。
4. `x.com` の投稿URLをコピーすると、自動で設定先ドメインへ書き換えられます。

## ビルド

Rust ツールチェーンが必要です。

```powershell
cargo build --release
```

生成物:

- `target\release\fix-x.exe`

## テスト

```powershell
cargo test
```

## 実装メモ

- Rust
- Win32 API (`windows` crate)
- レジストリの `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` を使って自動起動を制御

## ライセンス

MIT License
