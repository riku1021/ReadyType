# macOS コード署名（TCC / Accessibility / ログイン起動）

ReadyType は Accessibility（macOS 27+ では Device Control）権限と、ログイン起動（`SMAppService.mainApp`）に依存する。  
TCC はアプリのパスや Bundle ID ではなく、**designated requirement（DR）** で許可を照合する。  
SMAppService も **有効なコード署名** を前提とし、ad-hoc / 未署名では `kSMErrorInvalidSignature` などで失敗し得る。

| 署名 | DR のアンカー | 再ビルド後 |
|------|----------------|------------|
| ad-hoc（`-`） | バイナリの cdhash | 毎回不一致 → 設定上 ON でも AX 拒否 |
| 同一証明書 | certificate leaf + identifier | 許可が存続 |

ローカル開発の既定は Keychain 上の安定 **self-signed** 証明書 `ReadyType Code Signing`。  
配線は `APPLE_SIGNING_IDENTITY` 経由なので、将来同じ仕組みで Developer ID に差し替えられる。

**証明書は一度作ったら再生成しない。** leaf が変わると過去の TCC 許可がすべて無効になる。

## 一度きりの証明書作成

1. **Keychain Access.app** を開く
2. メニュー: Keychain Access → Certificate Assistant → **Create a Certificate…**
3. 次を設定する:
   - Name: `ReadyType Code Signing`
   - Identity Type: **Self Signed Root**
   - Certificate Type: **Code Signing**
4. Create で完了

### 存在確認

self-signed は信頼チェーンに載らないため、`-v`（valid only）だと一覧に出ないことがある。**`-v` は使わない。**

```bash
security find-identity -p codesigning | grep 'ReadyType Code Signing'
```

またはリポジトリの ensure スクリプト（証明書は**作成しない**。無いときは失敗する）:

```bash
bash tauri-desktop/rust-native/scripts/macos/ensure-signing-identity.sh
```

## 開発・ビルドでの使い方

`mise run dev` / `mise run build` は macOS 上で ensure を実行し、`APPLE_SIGNING_IDENTITY` を export してから Tauri を起動する。

- **`tauri dev`**: macOS 専用 runner（`scripts/macos/dev-runner.sh`）が `cargo build` 後に `tauri-desktop` バイナリへ codesign してから起動する  
  （Tauri 本体は開発モードで署名しないため。参照: [tauri#7930](https://github.com/tauri-apps/tauri/issues/7930)）
- **`tauri build`**: 同じ env を Tauri bundler が読み、`.app` を署名する
- **`mise run build`**: macOS では `--bundles app` のみ（TCC 確認に十分）。DMG は `mise run build:dmg`

`bundle_dmg.sh` が `リソースが使用中` / `couldn't unmount` で落ちる場合は、Finder が前の作業用ボリュームを掴んでいることが多い。`/Volumes/dmg.*` を取り外してから再実行する。

```bash
ls /Volumes | grep '^dmg\.'
# 例:
hdiutil detach "/Volumes/dmg.XXXX" -force
```

identity を上書きする場合:

```bash
export APPLE_SIGNING_IDENTITY='Developer ID Application: Your Name (TEAMID)'
mise run build
```

## 初回移行（ad-hoc からの切り替え）

ad-hoc 時代の許可は DR が違うため無効になる。一度リセットしてから、署名付きバイナリで再付与する。

```bash
tccutil reset Accessibility com.readytype
```

1. `mise run dev` で署名付き起動
2. System Settings → Privacy & Security → Accessibility（または Device Control）で一度だけ許可
3. アプリを完全終了して再起動し、再ビルドしても許可が残ることを確認

## 検証

開発バイナリ（例。`CARGO_TARGET_DIR` を設定している場合はその配下を見る）:

```bash
codesign -d -r- "${CARGO_TARGET_DIR:-tauri-desktop/rust-native/target}/debug/tauri-desktop"
```

リリース `.app`（例）:

```bash
codesign -d -r- tauri-desktop/rust-native/target/release/bundle/macos/ReadyType.app
```

成功時は `designated =>` に **`certificate leaf = H"..."`** が含まれる。  
`cdhash H"..."` **だけ**の場合は ad-hoc のままなので、identity / runner / env を見直す。

## ログイン起動（SMAppService）

ログイン項目の登録は OS API（`SMAppService.mainApp`）が正本。署名配線は TCC と同じ（`ReadyType Code Signing` / `APPLE_SIGNING_IDENTITY`）。

| 環境 | 期待 |
|------|------|
| `mise run dev`（ensure + codesign） | トグル操作・System Settings 承認までは試せる。実行ファイルが素の `target/...` の場合、アプリは register を拒否する（開発パスを Login Items に刺さない） |
| `mise run build` の署名付き `.app` | ログイン起動の本確認（推奨） |
| `cargo run` 直叩き / ad-hoc / ensure なし | 期待しない。失敗してよい |

アプリ側のガード:

- **`.app` バンドル内でない**ときの register → エラー（手順を docs へ誘導）
- **`kSMErrorInvalidSignature`** → 署名手順（本ドキュメント / `ensure-signing-identity.sh`）へ誘導するメッセージ

未署名向けの LaunchAgent フォールバックは行わない。

## 次段（配布・CI）

このドキュメントの配線を再利用する。

1. Apple Developer Program で **Developer ID Application** 証明書を作成し Keychain に入れる
2. `APPLE_SIGNING_IDENTITY='Developer ID Application: … (TEAMID)'` を設定
3. 公証: `APPLE_API_KEY` / `APPLE_API_ISSUER` / `APPLE_API_KEY_PATH`、または `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID`（[Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/)）
4. CI では `.p12` を base64 化した `APPLE_CERTIFICATE` と `APPLE_CERTIFICATE_PASSWORD` を使う
5. 必要なら `hardenedRuntime` と entitlements を有効化する（公証要件に応じて）

ローカルの TCC / ログイン起動確認だけが目的なら、self-signed のままでよい（Apple TN2206: 安定性は DR で決まり、CA が Apple である必要はない）。配布・公証では Developer ID に切り替える。
