# Flutter（macOS）単体アプリ版：入力ソース自動切替 要件定義

本ドキュメントは、現状の Hammerspoon 設定（`.hammerspoon/modules/switch_input_source_on_focus.lua`）と同等の挙動を、**Flutter をUI/ロジックの中心**として **macOSデスクトップアプリ（.app）単体**で実現するための要件定義である。

---

## 1. 背景 / 現状の挙動（As-Is）

現状は Hammerspoon が以下を行っている。

- **前面アプリ切替の検知**
  - Terminal.app がアクティブになったら **英数**へ切替
- **Cursor / VS Code 内のフォーカス判定**
  - 統合ターミナルにフォーカス → **英数**
  - エディタ（テキスト編集領域）にフォーカス → **英数**
  - Chat入力欄にフォーカス → **日本語**
  - ファイル名リネーム入力欄にフォーカス → **英数**
- **判定手段**
  - macOS アクセシビリティ（AX）で `AXFocusedUIElement` を取得し、親要素を辿って role/description/DOM identifier 等から文脈（terminal/chat/rename/editor）を推定
- **入力ソース切替**
  - 入力ソースID（例：`com.apple.keylayout.ABC` / `com.google.inputmethod.Japanese.base`）を指定して切替
- **デバッグ**
  - フォーカス要素ツリーをダンプするホットキー（開発・調整用）

---

## 2. 目的（To-Be）

- Hammerspoon を不要にし、**Flutter製のmacOSアプリ単体**で上記の自動切替を実現する
- GUIから **ON/OFF、状態確認、ルール編集、ログ確認** ができるようにする
- AX依存のため、**アクセシビリティ権限の取得/誘導**をアプリ側で完結させる

---

## 3. スコープ

### 3.1 対象（In scope）

- macOS デスクトップアプリ（Flutter for macOS）
- アクセシビリティAPI（AX）を用いたフォーカス要素の取得と判定
- 入力ソースの切替（TIS系API / Carbon等）
- ルールエンジン（アプリ/文脈→入力ソースのマッピング）
- UI（設定、メニューバー/ステータス、ログ、デバッグ支援）

### 3.2 対象外（Out of scope / Non-goals）

- iOS/Android/Windows/Linux での同等機能（将来検討は可）
- IME固有の辞書操作など、入力メソッド自体の高度な制御
- アプリ配布形態（App Store審査通過保証など）は要件外（ただし制約は考慮）

---

## 4. 用語

- **入力ソースID**: macOS が持つ入力ソース識別子（例：`com.apple.keylayout.ABC`）
- **AX**: macOS Accessibility API。`AXUIElement` 等でUI階層・フォーカスを取得
- **文脈（Focus Context）**: `terminal` / `chat` / `rename` / `editor` / `unknown` など

---

## 5. ユースケース（User Stories）

- アプリを起動すると常駐し、対象アプリでフォーカスが変わると入力ソースが自動で切り替わる
- いま自動切替が **有効/無効** かをメニューバーから一目で確認できる
- ルール（どのアプリのどの文脈で何に切替えるか）をGUIで編集できる
- うまく判定できない場合、デバッグ画面で **フォーカス要素情報（role/desc/DOM id 等）**を確認できる
- 不要な時は一時停止（スヌーズ）でき、再開も簡単にできる

---

## 6. 機能要件（Functional Requirements）

### 6.1 常駐/実行形態

- **FR-001**: アプリは常駐（バックグラウンド動作）できること
- **FR-002**: 起動時に自動実行（Login Item）を設定できること（任意/ユーザー選択）

### 6.2 前面アプリの監視

- **FR-010**: 前面アプリ（frontmost application）が切り替わったことを検知できること
- **FR-011**: 検知時にアプリの `bundleId` と名称を取得できること

### 6.3 フォーカス要素の取得（AX）

- **FR-020**: システム全体の `AXFocusedUIElement` を取得できること
- **FR-021**: 取得した要素から親要素を一定深さまで辿れること（深さは設定可能、既定値例：14）
- **FR-022**: 各要素から以下の属性を取得できること（取得失敗を安全に扱う）
  - `AXRole`, `AXSubrole`, `AXRoleDescription`, `AXTitle`, `AXDescription`, `AXIdentifier`, `AXDOMIdentifier`, `AXHelp`, `AXParent`

### 6.4 文脈判定（Focus Context Detection）

#### 6.4.1 判定対象アプリ

- **FR-030**: ルールは `bundleId` ベースで適用できること
- **FR-031**: 最低限の既定サポートとして以下を提供すること
  - Terminal.app: `com.apple.Terminal`
  - Cursor: `com.todesktop.230313mzl4w4u92`
  - VS Code: `com.microsoft.VSCode`

#### 6.4.2 文脈種別

- **FR-032**: 文脈を最低限以下に分類できること
  - `terminal`（統合ターミナル）
  - `chat`（Chat入力）
  - `rename`（ファイル名リネーム入力）
  - `editor`（エディタ本体）
  - `unknown`

#### 6.4.3 判定ロジック（優先順位含む）

- **FR-033**: 文脈判定の優先順位は **terminal > chat > rename > editor** とする（As-Is互換）
- **FR-034**: `terminal` 判定は、親探索中の要素が `AXRole == AXTextField` 等で、`AXDescription` 等に「ターミナル/Terminal」に相当する文字列を含む場合に成立できること（パターンは設定可能）
- **FR-035**: `chat` 判定は、`AXDOMIdentifier`/`AXDescription`/`AXTitle`/`AXRoleDescription` に以下のような識別子が含まれる場合に成立できること（パターンは設定可能）
  - Cursor系: `workbench.panel.aichat` / `aichat`
  - VS Code系: `workbench.panel.chat`
  - フォールバック: `chat` / `チャット` / `composer`
- **FR-036**: `rename` 判定は、`AXRole == AXTextField` かつ `AXDescription` にリネーム入力を示す文言（「ファイル名を入力します」「press enter to confirm」等）を含む場合に成立できること（パターンは設定可能）
- **FR-037**: `editor` 判定は、`AXDOMIdentifier` に `workbench.parts.editor` を含む、または `AXRoleDescription` が「エディター/editor」相当で成立できること（パターンは設定可能）

#### 6.4.4 検知トリガ（ポーリング/イベント）

- **FR-038**: Cursor/VS Code 等、同一アプリ内でのフォーカス変更を検知し文脈判定を更新できること
- **FR-039**: 方式は以下いずれか、または併用とする
  - AXObserver 等のイベント駆動
  - 一定間隔ポーリング（既定例：0.25秒、可変）
- **FR-040**: 前回文脈と同一なら切替処理を行わない（不要な切替抑制）

### 6.5 入力ソース切替

- **FR-050**: 現在の入力ソースIDを取得できること
- **FR-051**: 指定した入力ソースIDへ切り替えられること
- **FR-052**: 切替が失敗した場合、原因が分かる程度のログ（失敗/現在ID/対象ID/理由）を残せること
- **FR-053**: 既定の入力ソースIDを持つこと（ユーザーが変更可能）
  - 英数（例）：`com.apple.keylayout.ABC`
  - 日本語（例）：`com.google.inputmethod.Japanese.base`

### 6.6 ルールエンジン

- **FR-060**: ルールは「bundleId + 文脈 → 入力ソースID」の形式で管理できること
- **FR-061**: 既定ルール（As-Is互換）をプリセットとして提供すること
  - Terminal.app activated → 英数
  - Cursor/VS Code:
    - terminal → 英数
    - editor → 英数
    - rename → 英数
    - chat → 日本語
- **FR-062**: ルールには有効/無効フラグを持てること
- **FR-063**: ルールの適用対象外アプリは何もしないこと

### 6.7 UI（設定/操作）

- **FR-070**: メニューバー（ステータス）を提供し、以下を操作できること
  - 自動切替 ON/OFF
  - 現在の文脈/対象アプリ表示（可能な範囲で）
  - 直近の切替理由表示（任意）
  - 設定画面を開く
- **FR-071**: 設定画面で以下を編集できること
  - 英数/日本語の入力ソースID
  - アプリ別のルール
  - ポーリング間隔（採用時）
  - デバッグログの有効化
- **FR-072**: デバッグ画面で以下を確認できること
  - `AXFocusedUIElement` と親階層の属性一覧（role/desc/dom id 等）
  - 現在の文脈判定結果
  - 入力ソースID（現在/対象）

### 6.8 ログ/診断

- **FR-080**: ログレベル（例：Error/Warn/Info/Debug）を持つこと
- **FR-081**: Debug有効時のみ詳細ログ（要素属性など）を記録できること
- **FR-082**: GUIからログ閲覧・コピーができること（少なくとも直近N件）

---

## 7. 非機能要件（Non-Functional Requirements）

### 7.1 パフォーマンス

- **NFR-001**: ポーリング採用時、CPU使用率が常用で過度に上がらないこと（目安：アイドル時数%以内）
- **NFR-002**: 文脈変化から切替までの遅延が体感で遅すぎないこと（目安：0.5秒以内）

### 7.2 安定性

- **NFR-010**: AX属性取得失敗や例外が起きてもアプリがクラッシュしないこと
- **NFR-011**: 監視開始/停止を繰り返してもリーク・暴走が起きないこと

### 7.3 セキュリティ/プライバシー

- **NFR-020**: AXで取得する情報は判定に必要な範囲に限定し、ログ出力はユーザー同意（Debugオン）時のみ詳細化
- **NFR-021**: 収集データを外部送信しない（既定）※送信機能を追加する場合は明示同意が必須

### 7.4 互換性

- **NFR-030**: 対応macOSバージョンを定義すること（例：macOS 13+ など）
- **NFR-031**: Cursor/VS Code のUI変更に備え、マッチャ（判定パターン）は設定で更新可能にすること

### 7.5 UX

- **NFR-040**: 権限未許可時、原因と解決手順をユーザーに分かりやすく提示すること
- **NFR-041**: 誤判定時に「いま何を見てそう判定したか」を追える導線（デバッグ画面）を用意すること

---

## 8. 権限・OS制約（必須要件）

- **REQ-PERM-001**: アクセシビリティ（Accessibility）権限が必須
  - 未許可の場合、アプリ内で権限付与手順を案内し、必要に応じてシステム設定へ誘導する
- **REQ-PERM-002**: 入力ソース切替API利用に必要な権限/制約を満たすこと
  - サンドボックス/署名/配布方式により制約が出る可能性があるため、技術検証を実施する（後述の受け入れ基準に含める）

---

## 9. アーキテクチャ要件（Flutter単体で完結させるための構成）

### 9.1 全体構成

- Flutter（Dart）
  - UI（設定/メニューバー/ログ/デバッグ）
  - ルール管理・永続化
  - 状態管理（ON/OFF、現在文脈、最終切替理由）
- macOSネイティブ（Swift/Obj-C）
  - 前面アプリ監視（NSWorkspace等）
  - AXアクセス（AXUIElement、必要ならAXObserver）
  - 入力ソース取得/切替（TIS API）
- Flutter ↔ macOS は Platform Channel（MethodChannel / EventChannel）で接続

### 9.2 API（概念）要件

- **API-001**: `getFrontmostApp()` → `{ bundleId, name, pid }`
- **API-002**: `getFocusedElementSnapshot(maxDepth)` → 要素属性の配列
- **API-003**: `detectContext(bundleId, snapshot)` → `context`
  - ※検出ロジックはDart側/Swift側どちらでも良いが、テスト容易性を優先して分離できること
- **API-004**: `getCurrentInputSourceId()` → `string`
- **API-005**: `switchInputSource(targetId)` → `success/failure + reason`
- **API-006**: `startMonitoring(config)` / `stopMonitoring()`
  - 監視イベント（文脈変化、切替実行、エラー）をEventChannelでストリーミング可能にする

---

## 10. データ要件（設定・永続化）

- **DATA-001**: 設定はローカルに永続化（例：`shared_preferences`、ファイル、Keychainは不要）
- **DATA-002**: ルールは人間が壊さず扱える形式（JSON等）で保存できること
- **DATA-003**: 既定設定へ戻す（Reset）機能を持つこと

---

## 11. 受け入れ基準（Acceptance Criteria）

### 11.1 基本

- **AC-001**: Terminal.app をアクティブにすると英数へ切り替わる
- **AC-002**: Cursor/VS Code で統合ターミナルにフォーカスすると英数へ切り替わる
- **AC-003**: Cursor の Chat 入力欄にフォーカスすると日本語へ切り替わる
- **AC-004**: Cursor/VS Code でファイル名リネーム入力欄にフォーカスすると英数へ切り替わる
- **AC-005**: 同一文脈での連続切替が発生しない（不要な再切替抑止）

### 11.2 設定/運用

- **AC-010**: GUIでON/OFFが切り替えられ、OFF中は一切切替が起きない
- **AC-011**: 入力ソースIDをGUIで変更でき、変更が反映される
- **AC-012**: 権限が未許可の場合、アプリが落ちずに案内が表示される

### 11.3 デバッグ

- **AC-020**: デバッグ画面でフォーカス要素ツリー（必要属性）が閲覧できる
- **AC-021**: ログをGUIからコピーでき、切替理由が追跡できる

---

## 12. テスト要件

- **TEST-001**: 文脈判定ロジックはユニットテスト可能な形に分離すること
- **TEST-002**: 手動E2E手順書を用意すること（Terminal/Cursor/VS Code 各文脈で確認）
- **TEST-003**: Cursor/VS Code のUI変更に備え、マッチャ更新手順をドキュメント化すること

---

## 13. リスク/課題（事前に織り込む）

- **RISK-001**: Cursor/VS Code のAX属性（DOM identifier等）がアップデートで変わる可能性がある
  - 対策: マッチャを設定化し、デバッグ画面で確認→更新できるようにする
- **RISK-002**: AXイベント駆動が難しい場合はポーリングが必要（省電力/CPU負荷とのトレードオフ）
  - 対策: ポーリング間隔の調整、アプリ非アクティブ時の停止
- **RISK-003**: サンドボックスや配布方式によりAX/TISが制約される可能性
  - 対策: 早期に技術検証を実施し、配布方針（署名/ノータリゼーション等）を決める

---

## 14. 今後の拡張（任意）

- アプリごとの複数ルール（ウィンドウタイトル/プロセス名など追加条件）
- “一時的にこのアプリだけ固定” のピン留め機能
- ルールのエクスポート/インポート
