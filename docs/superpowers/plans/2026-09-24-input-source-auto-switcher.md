# Input Source Auto Switcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete ReadyType macOS input-source auto switcher with event-driven focus detection, editable rules, diagnostics, tray residency, and login startup.

**Architecture:** Keep matching and switching policy in pure Rust domain/application layers, place macOS AX/TIS integrations behind ports, and expose state-changing use cases through thin Tauri commands. React owns forms and presentation only; the latency-critical focus-to-switch path stays native and emits versioned state snapshots after processing.

**Tech Stack:** Rust 2021, Tauri 2, `objc2` framework bindings, Carbon TIS FFI, serde/JSON, React 19, TypeScript 6, Jotai, TanStack Router, Panda CSS, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-24-input-source-auto-switcher-design.md`

## Global Constraints

- Initial platform support is macOS 13 and later; Windows/Linux must keep compiling with an explicit unsupported adapter.
- `domain` and `application` must not import Tauri or OS APIs; dependencies point inward only.
- Tauri commands remain thin and command names exactly match frontend `invoke` calls.
- No React Query, axios, form library, drag-and-drop library, or alternate CSS stack is added.
- TypeScript uses no `any` or `unknown`; generated `styled-system/` and `routeTree.gen.ts` are never hand-edited.
- Normal focus processing never round-trips through the WebView before selecting the input source.
- Detailed AX strings are retained only while debug capture/logging is enabled and are never sent externally.
- The hot path has no time-based debounce; event-receipt-to-TIS-completion target is p95 <= 50 ms on the development machine.

## Review Focus

- An AX event from the previously active PID after an app switch must be discarded; Task 4 pins this with a stale-generation test.
- A rule whose conditions match different ancestors must still match because each condition independently searches its configured scope; Task 1 pins this behavior.
- Saving a duplicate rule ID or empty comparison value must fail without altering the active settings; Tasks 1 and 4 test validation and rollback.
- A configured input source removed from macOS must remain visible as unavailable and a failed selection must refresh the cache exactly once; Tasks 5 and 9 test both sides.
- Permission revocation while running must stop the observer without terminating ReadyType and permission restoration must restart monitoring; Tasks 6 and 7 test the state transitions.

---

### Task 1: Domain types and rule matcher

**Files:**
- Create: `tauri-desktop/rust-native/src/domain/input_source/mod.rs`
- Create: `tauri-desktop/rust-native/src/domain/input_source/model.rs`
- Create: `tauri-desktop/rust-native/src/domain/input_source/matcher.rs`
- Modify: `tauri-desktop/rust-native/src/domain/mod.rs`
- Modify: `tauri-desktop/rust-native/src/domain/errors.rs`

**Interfaces:**
- Consumes: only `std` and existing `DomainError`.
- Produces: `InputSourceId::parse`, `FocusRule::validate`, `RuleSet::new`, `RuleSet::first_match`, and the serializable-neutral domain enums `AxAttribute`, `MatchOperator`, `SearchScope`, `RuleTarget`, `Platform`.

- [ ] **Step 1: Write matcher and validation tests**

Add adjacent unit tests that construct snapshots explicitly:

```rust
#[test]
fn all_conditions_may_match_different_ancestors() {
    let rule = rule_with_conditions(vec![
        condition(AxAttribute::Role, MatchOperator::Equals, SearchScope::FocusedOnly, "AXTextArea"),
        condition(AxAttribute::DomIdentifier, MatchOperator::Contains, SearchScope::FocusedAndAncestors, "aichat"),
    ]);
    let snapshot = FocusSnapshot::new(
        app("com.todesktop.230313mzl4w4u92", 42),
        7,
        vec![element("AXTextArea", ""), element("AXGroup", "workbench.panel.aichat")],
    );

    assert!(rule.matches(&snapshot));
}

#[test]
fn rejects_empty_condition_value_and_duplicate_rule_ids() {
    assert!(FocusRule::validate(rule_with_value(" ")).is_err());
    assert!(RuleSet::new(vec![rule_with_id("same"), rule_with_id("same")]).is_err());
}
```

Also cover case-insensitive equals/contains, focused-only exclusion, zero-condition app match, disabled rules, priority, invalid input-source ID, and wrong platform/app ID.

- [ ] **Step 2: Run the focused tests and confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test domain::input_source -- --nocapture`  
Expected: FAIL because `domain::input_source` and its types do not exist.

- [ ] **Step 3: Implement the immutable domain model**

Use enums rather than stringly typed matching:

```rust
pub enum AxAttribute { Role, Subrole, RoleDescription, Title, Description, Identifier, DomIdentifier, Help }
pub enum MatchOperator { Equals, Contains }
pub enum SearchScope { FocusedOnly, FocusedAndAncestors }
pub enum RuleTarget { PreferredEnglish, PreferredJapanese, Exact(InputSourceId) }

impl RuleCondition {
    pub fn matches(&self, elements: &[AxElementSnapshot]) -> bool {
        let candidates = match self.scope {
            SearchScope::FocusedOnly => &elements[..elements.len().min(1)],
            SearchScope::FocusedAndAncestors => elements,
        };
        candidates.iter().any(|element| self.matches_element(element))
    }
}
```

`RuleSet::new` validates then stable-sorts by `priority`; `first_match` filters enabled/platform/app before matching conditions. Add `DomainError::DuplicateId` and precise invalid-field messages.

- [ ] **Step 4: Run tests and lint**

Run: `cd tauri-desktop/rust-native && cargo test domain::input_source && cargo clippy --all-targets -- -D warnings`  
Expected: all Task 1 tests pass and Clippy reports no warnings.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/src/domain
git commit -m "入力ソースルールのドメインモデルを追加"
```

### Task 2: Default presets and settings schema

**Files:**
- Create: `tauri-desktop/rust-native/src/domain/input_source/presets.rs`
- Create: `tauri-desktop/rust-native/src/application/settings.rs`
- Modify: `tauri-desktop/rust-native/src/domain/input_source/mod.rs`
- Modify: `tauri-desktop/rust-native/src/application/mod.rs`

**Interfaces:**
- Consumes: Task 1 domain types.
- Produces: `default_rules() -> Vec<FocusRule>`, `SwitcherSettings::defaults()`, `SwitcherSettings::validate()`, and `CURRENT_SCHEMA_VERSION: u32`.

- [ ] **Step 1: Add preset and settings tests**

```rust
#[test]
fn cursor_chat_wins_before_editor() {
    let rules = RuleSet::new(default_rules()).expect("valid presets");
    let snapshot = cursor_snapshot_with("workbench.panel.aichat", "editor");
    assert_eq!(rules.first_match(&snapshot).map(FocusRule::name), Some("Cursor Chat"));
}

#[test]
fn defaults_include_terminal_cursor_and_vscode() {
    let settings = SwitcherSettings::defaults();
    assert!(settings.rules.iter().any(|rule| rule.app_id() == "com.apple.Terminal"));
    assert!(settings.rules.iter().any(|rule| rule.app_id() == "com.microsoft.VSCode"));
}
```

Pin terminal > chat > rename > editor priority and the PoC identifiers/keywords in separate named cases.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test presets settings::tests`  
Expected: FAIL because presets/settings do not exist.

- [ ] **Step 3: Implement schema and presets**

Define:

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub struct SwitcherSettings {
    pub schema_version: u32,
    pub enabled: bool,
    pub preferred_english: InputSourceId,
    pub preferred_japanese: InputSourceId,
    pub rules: Vec<FocusRule>,
    pub debug_logging: bool,
    pub snoozed_until_unix_ms: Option<u64>,
}
```

Use stable literal rule IDs such as `preset-terminal`, `preset-cursor-chat`, and `preset-vscode-editor` so reset and migration are deterministic.

- [ ] **Step 4: Verify GREEN**

Run: `cd tauri-desktop/rust-native && cargo test presets settings::tests`  
Expected: all preset/schema tests pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/src/domain/input_source tauri-desktop/rust-native/src/application
git commit -m "既定入力ソースルールと設定スキーマを追加"
```

### Task 3: Settings repository and atomic JSON persistence

**Files:**
- Create: `tauri-desktop/rust-native/src/application/ports/settings_repository.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/persistence/mod.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/persistence/json_settings_repository.rs`
- Modify: `tauri-desktop/rust-native/src/application/ports/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/mod.rs`
- Modify: `tauri-desktop/rust-native/src/domain/input_source/model.rs`
- Modify: `tauri-desktop/rust-native/src/application/settings.rs`

**Interfaces:**
- Consumes: `SwitcherSettings` from Task 2.
- Produces: trait `SettingsRepository { fn load(&self) -> Result<LoadOutcome, SettingsError>; fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError>; }`, serde wire structs, and `JsonSettingsRepository::new(PathBuf)`.

- [ ] **Step 1: Write repository tests with unique temporary directories**

```rust
#[test]
fn save_then_load_round_trips() {
    let dir = test_directory("round-trip");
    let repository = JsonSettingsRepository::new(dir.join("settings.json"));
    let expected = SwitcherSettings::defaults();
    repository.save(&expected).expect("save");
    assert_eq!(repository.load().expect("load").settings(), &expected);
}

#[test]
fn corrupt_json_is_backed_up_and_defaults_are_returned() {
    let repository = repository_containing("{not-json");
    let outcome = repository.load().expect("recover");
    assert!(matches!(outcome, LoadOutcome::Recovered { .. }));
    assert!(outcome.backup_path().expect("backup").exists());
}
```

Also test missing file, schema v1 decode, unsupported future schema, validation failure, and absence of a leftover `.tmp` file after success.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test json_settings_repository -- --nocapture`  
Expected: FAIL because the repository is absent.

- [ ] **Step 3: Add serde DTOs and repository implementation**

Keep serde out of domain by converting through infrastructure wire DTOs. Save using `OpenOptions`, `write_all`, `sync_all`, then `rename`:

```rust
let temporary_path = self.path.with_extension("json.tmp");
let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&temporary_path)?;
file.write_all(&bytes)?;
file.sync_all()?;
fs::rename(&temporary_path, &self.path)?;
```

On parse/validation failure, rename the original to `settings.corrupt-<unix-ms>.json` and return defaults in `LoadOutcome::Recovered` with a user-visible warning.

- [ ] **Step 4: Verify persistence tests**

Run: `cd tauri-desktop/rust-native && cargo test json_settings_repository`  
Expected: all persistence tests pass without touching the real app config directory.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/src/application tauri-desktop/rust-native/src/domain tauri-desktop/rust-native/src/infrastructure
git commit -m "入力ソース設定の永続化を追加"
```

### Task 4: Application switching engine

**Files:**
- Create: `tauri-desktop/rust-native/src/application/ports/clock.rs`
- Create: `tauri-desktop/rust-native/src/application/ports/input_source.rs`
- Create: `tauri-desktop/rust-native/src/application/ports/switcher_events.rs`
- Create: `tauri-desktop/rust-native/src/application/input_source_switcher.rs`
- Create: `tauri-desktop/rust-native/src/application/switcher_state.rs`
- Modify: `tauri-desktop/rust-native/src/application/ports/mod.rs`
- Modify: `tauri-desktop/rust-native/src/application/mod.rs`

**Interfaces:**
- Consumes: Tasks 1-3 domain/settings and repository port.
- Produces: `InputSourceGateway`, `Clock`, `SwitcherEventPublisher`, `InputSourceSwitcher<G, R, C, E>`, `handle_focus(FocusSnapshot)`, configuration mutations, log ring buffer, and `SwitcherState`.

- [ ] **Step 1: Write fake-driven use-case tests**

```rust
#[test]
fn stale_generation_is_ignored() {
    let mut switcher = fixture_for_app(42, 8);
    switcher.handle_focus(snapshot(42, 7, "chat")).expect("handle");
    assert!(switcher.gateway().selections().is_empty());
}

#[test]
fn manual_change_is_respected_until_focus_changes() {
    let mut switcher = fixture_for_editor();
    switcher.handle_focus(editor_snapshot(1)).expect("first");
    switcher.gateway_mut().set_current("jp");
    switcher.handle_focus(editor_snapshot(1)).expect("duplicate");
    assert_eq!(switcher.gateway().selections(), &["en"]);
    switcher.handle_focus(editor_snapshot(2)).expect("new focus");
    assert_eq!(switcher.gateway().selections(), &["en", "en"]);
}

#[test]
fn failed_settings_save_leaves_active_rules_unchanged() {
    let mut switcher = fixture_with_failing_repository();
    let before = switcher.state().rules_version;
    assert!(switcher.replace_rules(valid_rules()).is_err());
    assert_eq!(switcher.state().rules_version, before);
}
```

Also test disabled state, active/expired snooze, same-source no-op, unmatched rule, ring buffer truncation at 200, retry exactly once after cache refresh, and emitted monotonic state versions.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test input_source_switcher -- --nocapture`  
Expected: FAIL because ports and service do not exist.

- [ ] **Step 3: Implement ports, state, and service**

The core event API is synchronous so the native worker can keep ordering explicit:

```rust
pub trait InputSourceGateway {
    fn list(&mut self, refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError>;
    fn current(&self) -> Result<InputSourceId, GatewayError>;
    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError>;
}

pub fn handle_focus(&mut self, snapshot: FocusSnapshot) -> Result<SwitchDecision, ApplicationError> {
    if self.is_stale_or_duplicate(&snapshot) { return Ok(SwitchDecision::Ignored); }
    let decision = self.decide(&snapshot)?;
    self.apply(decision, snapshot)
}
```

Persist candidate settings before replacing the in-memory settings/rule index. Record `Instant` durations separately from wall-clock log timestamps.

- [ ] **Step 4: Verify engine tests**

Run: `cd tauri-desktop/rust-native && cargo test application::input_source_switcher`  
Expected: all use-case tests pass with fakes and no OS access.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/src/application
git commit -m "入力ソース切替ユースケースを追加"
```

### Task 5: macOS TIS input-source adapter

**Files:**
- Create: `tauri-desktop/rust-native/src/infrastructure/platform/input_source.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/platform/macos/tis.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/macos/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/mod.rs`
- Modify: `tauri-desktop/rust-native/Cargo.toml`

**Interfaces:**
- Consumes: Task 4 `InputSourceGateway`.
- Produces: `MacOsInputSourceGateway`, cross-platform `UnsupportedInputSourceGateway`, and a cached map from `InputSourceId` to retained native TIS handles.

- [ ] **Step 1: Write adapter-policy tests around a fake TIS backend**

```rust
#[test]
fn missing_target_refreshes_once_then_retries_selection() {
    let backend = FakeTis::with_target_after_refresh("jp");
    let mut gateway = MacOsInputSourceGateway::with_backend(backend);
    gateway.select(&input_id("jp")).expect("select after refresh");
    assert_eq!(gateway.backend().list_calls(), 2);
    assert_eq!(gateway.backend().select_calls(), 1);
}
```

Test display-name/ID conversion, enabled/selectable filtering, current-source equality, OSStatus propagation, and one-refresh-only failure.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test infrastructure::platform::input_source`  
Expected: FAIL because the adapter is absent.

- [ ] **Step 3: Add the narrow Carbon TIS wrapper and cache**

Add macOS-only dependencies/bindings and contain symbols in `tis.rs`:

```rust
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> *const c_void;
    fn TISCreateInputSourceList(properties: *const c_void, include_all_installed: u8) -> *const c_void;
    fn TISSelectInputSource(source: *const c_void) -> i32;
}
```

Wrap every owned Core Foundation value in an RAII type with documented ownership rules. Cache only enabled/selectable keyboard sources, expose name plus ID, and rebuild on explicit refresh or failed lookup.

- [ ] **Step 4: Verify unit tests and macOS compilation**

Run: `cd tauri-desktop/rust-native && cargo test infrastructure::platform::input_source && cargo check --all-targets`  
Expected: policy tests pass; macOS symbols link without executing a real switch.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/Cargo.toml tauri-desktop/rust-native/Cargo.lock tauri-desktop/rust-native/src/infrastructure/platform
git commit -m "macOS入力ソースアダプターを追加"
```

### Task 6: macOS accessibility snapshot and event monitor

**Files:**
- Create: `tauri-desktop/rust-native/src/application/ports/focus_monitor.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/platform/focus_monitor.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/platform/macos/accessibility.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/platform/macos/focus_monitor.rs`
- Modify: `tauri-desktop/rust-native/src/application/ports/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/macos/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/mod.rs`
- Modify: `tauri-desktop/rust-native/Cargo.toml`

**Interfaces:**
- Consumes: domain `FocusSnapshot`, app `FocusMonitor` port, and candidate attribute requests from the rule index.
- Produces: `MacOsFocusMonitor::start`, `stop`, `set_debug_capture`, `permission_status`, `request_permission`, `open_settings`, plus an unsupported adapter.

- [ ] **Step 1: Write pure monitor-state tests**

```rust
#[test]
fn observer_failure_enables_fallback_only_for_frontmost_pid() {
    let mut state = MonitorState::default();
    state.activate(app("com.example", 41), ObserverRegistration::Unsupported);
    assert_eq!(state.mode(), MonitoringMode::FallbackPolling { interval_ms: 50 });
    state.activate(app("com.other", 42), ObserverRegistration::Registered);
    assert_eq!(state.mode(), MonitoringMode::AxNotification);
    assert!(!state.accepts(41, state.generation()));
}

#[test]
fn permission_revocation_stops_observer_and_restoration_restarts() {
    let mut state = running_monitor_state();
    state.update_permission(false);
    assert_eq!(state.mode(), MonitoringMode::StoppedNoPermission);
    state.update_permission(true);
    assert!(state.needs_restart());
}
```

Also test generation increments, duplicate signal coalescing, debug/full attribute plans, normal/minimal attribute plans, and parent-depth limit 14.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test focus_monitor -- --nocapture`  
Expected: FAIL because monitor types are absent.

- [ ] **Step 3: Implement AX wrappers and monitor lifecycle**

Add target dependencies:

```toml
objc2-application-services = { version = "0.3.2", features = ["AXError", "AXUIElement", "HIServices", "libc"] }
objc2-core-foundation = { version = "0.3.2", features = ["CFRunLoop", "CFString"] }
```

The dedicated run-loop thread owns all non-`Send` AX objects. Main-thread NSWorkspace notifications send `{ app, generation }` commands; AX callbacks enqueue `{ pid, generation }` evaluation signals. Query only requested attributes in normal mode and all supported attributes in debug mode. Use `AXIsProcessTrustedWithOptions` only after explicit user action.

- [ ] **Step 4: Verify tests and lifecycle cleanup**

Run: `cd tauri-desktop/rust-native && cargo test focus_monitor && cargo clippy --all-targets -- -D warnings`  
Expected: state tests pass, observer/run-loop handles are released on stop, no undocumented unsafe blocks or warnings remain.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/Cargo.toml tauri-desktop/rust-native/Cargo.lock tauri-desktop/rust-native/src/application/ports tauri-desktop/rust-native/src/infrastructure/platform
git commit -m "macOSのイベント駆動フォーカス監視を追加"
```

### Task 7: Runtime controller, Tauri commands, and state events

**Files:**
- Create: `tauri-desktop/rust-native/src/application/switcher_controller.rs`
- Create: `tauri-desktop/rust-native/src/presentation/commands/input_source_switcher.rs`
- Create: `tauri-desktop/rust-native/src/presentation/input_source_dto.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/commands/mod.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/mod.rs`
- Modify: `tauri-desktop/rust-native/src/lib.rs`

**Interfaces:**
- Consumes: Tasks 3-6 repositories, switcher service, focus/input adapters.
- Produces: all spec section 13 commands, `input-source-switcher://state` event with `{ version, state }`, and `Arc<SwitcherController>` managed by Tauri.

- [ ] **Step 1: Write DTO and controller-transition tests**

```rust
#[test]
fn permission_restoration_restarts_monitoring_without_process_restart() {
    let fixture = ControllerFixture::permission_denied();
    fixture.controller.refresh_permission().expect("denied state");
    fixture.permission.set_trusted(true);
    fixture.controller.refresh_permission().expect("restart");
    assert_eq!(fixture.monitor.start_calls(), 1);
}

#[test]
fn dto_uses_camel_case_and_preserves_state_version() {
    let json = serde_json::to_value(StateDto::from(state_at_version(9))).expect("serialize");
    assert_eq!(json["version"], 9);
    assert!(json.get("currentApp").is_some());
}
```

Test invalid rule DTO rejection, delete/reorder/reset, input preference changes, log clear, debug capture start/stop, debug logging updates, permission-revoked transition, and automatic snooze expiry while no focus event arrives.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test switcher_controller input_source_dto`  
Expected: FAIL because controller and DTOs do not exist.

- [ ] **Step 3: Implement composition and thin commands**

Commands follow one pattern:

```rust
#[tauri::command]
pub fn set_input_source_switcher_enabled(
    enabled: bool,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller.set_enabled(enabled).map(StateDto::from).map_err(IpcError::from)
}
```

Register every command listed in the spec, construct adapters and repository in `setup`, load settings before monitor start, and emit a complete versioned snapshot after mutations and processed focus events. The controller worker uses `recv_timeout` until the active snooze deadline; timeout calls `resume_if_due` and emits the resumed state even when no focus event occurs.

- [ ] **Step 4: Verify Rust integration tests**

Run: `cd tauri-desktop/rust-native && cargo test switcher_controller input_source_dto && cargo check --all-targets`  
Expected: commands compile, mappings/transitions pass, and no test accesses the real OS.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/src/application tauri-desktop/rust-native/src/presentation tauri-desktop/rust-native/src/lib.rs
git commit -m "入力ソース切替のTauri APIを追加"
```

### Task 8: Tray residency, close behavior, snooze, and autostart

**Files:**
- Create: `tauri-desktop/rust-native/src/infrastructure/desktop/mod.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/desktop/tray.rs`
- Create: `tauri-desktop/rust-native/src/presentation/commands/autostart.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/mod.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/commands/mod.rs`
- Modify: `tauri-desktop/rust-native/src/lib.rs`
- Modify: `tauri-desktop/rust-native/Cargo.toml`
- Modify: `tauri-desktop/rust-native/tauri.conf.json`
- Modify: `tauri-desktop/react-ui/src/hooks/useRevealMainWindow.ts`
- Create: `tauri-desktop/react-ui/src/hooks/revealMainWindow.ts`
- Create: `tauri-desktop/react-ui/src/hooks/revealMainWindow.test.ts`

**Interfaces:**
- Consumes: Task 7 controller/state.
- Produces: native tray menu actions, window hide/show behavior, explicit quit, `get_autostart_enabled`, `set_autostart_enabled`, and `should_show_main_window_on_launch`.

- [ ] **Step 1: Write menu-state tests**

```rust
#[test]
fn tray_model_exposes_resume_only_during_snooze() {
    let model = TrayMenuModel::from(snoozed_state(15));
    assert!(model.items.iter().any(|item| item.id == "resume" && item.enabled));
    assert!(model.status.contains("一時停止"));
}
```

Test enabled/disabled labels, target app/rule label fallback, 5/15/60-minute actions, and login-launch argument parsing.
In Vitest, also pin that manual launch returns `true` while `--autostart` returns `false` from
`shouldRevealMainWindow(args: readonly string[]): boolean`.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/rust-native && cargo test tray`  
Expected: FAIL because tray model does not exist.

- [ ] **Step 3: Implement desktop lifecycle**

Enable Tauri's `tray-icon` feature and add `tauri-plugin-autostart = "2"`. Initialize the plugin with `MacosLauncher::LaunchAgent` and `--autostart`; expose the parsed launch mode through `should_show_main_window_on_launch`. Update `useRevealMainWindow` to ask that command before calling `show()` and remove the explicit `catch (error: unknown)` annotation while touching the file. Intercept main-window close to prevent process exit and hide it. Rebuild tray labels from controller state; menu actions call controller use cases. Only the explicit `quit` item calls `app.exit(0)`.

```typescript
const shouldShow = await invoke<boolean>("should_show_main_window_on_launch");
if (!shouldShow || cancelled) {
  clearBootstrapBackground();
  return;
}
await getCurrentWindow().show();
```

- [ ] **Step 4: Verify tray/autostart tests and build**

Run: `cd tauri-desktop/rust-native && cargo test tray && cargo check --all-targets && cd ../react-ui && pnpm test -- src/hooks/revealMainWindow.test.ts && pnpm typecheck`  
Expected: tray/launch-mode tests pass, desktop APIs compile, and login launch cannot be revealed by the frontend startup hook.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/rust-native/Cargo.toml tauri-desktop/rust-native/Cargo.lock tauri-desktop/rust-native/tauri.conf.json tauri-desktop/rust-native/src tauri-desktop/react-ui/src/hooks
git commit -m "常駐メニューとログイン起動を追加"
```

### Task 9: Frontend typed IPC client and reactive state

**Files:**
- Create: `tauri-desktop/react-ui/src/features/inputSource/types.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSource/api.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSource/api.test.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSource/atoms/inputSourceStateAtom.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSource/hooks/useInputSourceState.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSource/index.ts`
- Modify: `tauri-desktop/react-ui/src/main.tsx`

**Interfaces:**
- Consumes: Task 7 command/event DTOs.
- Produces: exact TypeScript DTO unions, one wrapper per command, `inputSourceStateAtom`, and `useInputSourceState()` bootstrap/subscription hook.

- [ ] **Step 1: Write IPC wrapper tests with `mockIPC`**

```typescript
it("sends a validated create_rule payload", async () => {
  mockIPC((command, args) => {
    expect(command).toBe("create_rule");
    expect(args).toEqual({ rule: validRule });
    return stateAtVersion(4);
  });
  await expect(createRule(validRule)).resolves.toMatchObject({ version: 4 });
});

it("marks a configured but absent source unavailable", () => {
  expect(withConfiguredSource(availableSources, "removed.id").at(-1)).toEqual({
    id: "removed.id", name: "removed.id", available: false,
  });
});
```

Test every command name, state-event stale-version rejection, invoke errors, and unavailable input-source merging.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/inputSource/api.test.ts`  
Expected: FAIL because the feature is absent.

- [ ] **Step 3: Implement DTOs, wrappers, and subscription**

Use discriminated unions and no untyped IPC boundary:

```typescript
export type RuleTarget =
  | { kind: "preferredEnglish" }
  | { kind: "preferredJapanese" }
  | { kind: "exact"; inputSourceId: string };

export async function createRule(rule: RuleDto): Promise<SwitcherStateDto> {
  return invoke<SwitcherStateDto>("create_rule", { rule });
}
```

Bootstrap with `get_input_source_switcher_state`, subscribe to the versioned event, ignore events whose version is not newer, and surface initialization failure in state.

- [ ] **Step 4: Verify frontend foundation**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/inputSource/api.test.ts && pnpm typecheck`  
Expected: tests and strict type checking pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/react-ui/src/features/inputSource tauri-desktop/react-ui/src/main.tsx
git commit -m "入力ソース切替のフロント状態層を追加"
```

### Task 10: Home status and permission experience

**Files:**
- Modify: `tauri-desktop/react-ui/src/features/home/HomePage.tsx`
- Modify: `tauri-desktop/react-ui/src/features/home/HomePage.styles.ts`
- Create: `tauri-desktop/react-ui/src/features/home/statusViewModel.ts`
- Create: `tauri-desktop/react-ui/src/features/home/statusViewModel.test.ts`

**Interfaces:**
- Consumes: Task 9 `useInputSourceState` and commands for enable/permission/settings.
- Produces: operational home dashboard and pure `toStatusViewModel` formatting.

- [ ] **Step 1: Write view-model tests**

```typescript
it.each([
  ["running", "自動切替は有効です"],
  ["snoozed", "一時停止中"],
  ["permissionRequired", "アクセシビリティ権限が必要です"],
  ["unsupported", "このOSでは未対応です"],
] as const)("formats %s", (status, expected) => {
  expect(toStatusViewModel(stateWithStatus(status)).headline).toBe(expected);
});
```

Cover empty current app, no matched rule, failed last switch, and timing formatting.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/home/statusViewModel.test.ts`  
Expected: FAIL because the formatter does not exist.

- [ ] **Step 3: Build the dashboard**

Replace the empty page with the ON/OFF card, permission banner/buttons, current app/rule/source card, monitoring mode, last result, and timing. Use existing Button/Toggle/SettingsSection components where their contracts fit; keep styles in `HomePage.styles.ts`.

```tsx
const { state, setEnabled, requestPermission, openAccessibilitySettings } =
  useInputSourceState();
const view = toStatusViewModel(state);

return (
  <div className={styles.page}>
    {!state.accessibilityTrusted && (
      <PermissionBanner onRequest={requestPermission} onOpenSettings={openAccessibilitySettings} />
    )}
    <SwitcherStatusCard view={view} enabled={state.enabled} onEnabledChange={setEnabled} />
  </div>
);
```

- [ ] **Step 4: Verify page code**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/home/statusViewModel.test.ts && pnpm typecheck && pnpm lint`  
Expected: all pass with keyboard-accessible controls and explicit button labels.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/react-ui/src/features/home
git commit -m "入力ソース切替のホーム状態画面を追加"
```

### Task 11: Rule list and editor

**Files:**
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/components/RulesPage/RulesPage.tsx`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/components/RulesPage/RulesPage.styles.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/components/RuleDialog/RuleDialog.tsx`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/components/RuleDialog/RuleDialog.styles.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/ruleForm.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/ruleForm.test.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceRules/index.ts`
- Create: `tauri-desktop/react-ui/src/routes/rules/index.tsx`
- Modify: `tauri-desktop/react-ui/src/features/navigation/navLinks.ts`
- Modify: `tauri-desktop/react-ui/src/features/navigation/navLinks.test.ts`

**Interfaces:**
- Consumes: Task 9 rule DTOs/API/state and existing dialog/form primitives.
- Produces: `/rules`, rule CRUD/duplicate/reorder UI, and `validateRuleForm`/`toRuleDto`.

- [ ] **Step 1: Write form conversion and navigation tests**

```typescript
it("rejects blank condition values", () => {
  expect(validateRuleForm(formWithCondition("   "))).toEqual({
    field: "conditions.0.value", message: "比較値を入力してください",
  });
});

it("duplicates with a fresh id and adjacent priority", () => {
  const copy = duplicateRule(ruleAtPriority(2), "new-id");
  expect(copy).toMatchObject({ id: "new-id", priority: 3, name: "Rule のコピー" });
});
```

Test empty app/name, exact target without ID, add/remove conditions, stable up/down reorder, and `/rules` navigation metadata.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/inputSourceRules src/features/navigation/navLinks.test.ts`  
Expected: FAIL because the feature/route is absent.

- [ ] **Step 3: Implement list and accessible editor dialog**

Use `crypto.randomUUID()` for new IDs. Render rule status, app ID, condition summary, target, enable toggle, up/down, edit, duplicate, and delete. The dialog edits local state, displays field-level errors, and invokes create/update only after validation. Confirm destructive delete/reset via the existing Dialog primitives.

```typescript
export function duplicateRule(rule: RuleDto, id: string): RuleDto {
  return { ...rule, id, name: `${rule.name} のコピー`, priority: rule.priority + 1 };
}

export function moveRule(rules: readonly RuleDto[], id: string, offset: -1 | 1): RuleDto[] {
  const from = rules.findIndex((rule) => rule.id === id);
  const to = from + offset;
  if (from < 0 || to < 0 || to >= rules.length) return [...rules];
  const reordered = [...rules];
  [reordered[from], reordered[to]] = [reordered[to], reordered[from]];
  return reordered.map((rule, priority) => ({ ...rule, priority }));
}
```

- [ ] **Step 4: Generate route tree and verify**

Run: `cd tauri-desktop/react-ui && pnpm exec vite build && pnpm test -- src/features/inputSourceRules src/features/navigation/navLinks.test.ts && pnpm typecheck`  
Expected: TanStack regenerates `routeTree.gen.ts`; tests and type checking pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/react-ui/src/features/inputSourceRules tauri-desktop/react-ui/src/features/navigation tauri-desktop/react-ui/src/routes/rules tauri-desktop/react-ui/src/routeTree.gen.ts
git commit -m "入力ソース切替ルール編集画面を追加"
```

### Task 12: Diagnostics, AX inspection, and condition capture

**Files:**
- Create: `tauri-desktop/react-ui/src/features/inputSourceDiagnostics/components/DiagnosticsPage/DiagnosticsPage.tsx`
- Create: `tauri-desktop/react-ui/src/features/inputSourceDiagnostics/components/DiagnosticsPage/DiagnosticsPage.styles.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceDiagnostics/diagnosticsViewModel.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceDiagnostics/diagnosticsViewModel.test.ts`
- Create: `tauri-desktop/react-ui/src/features/inputSourceDiagnostics/index.ts`
- Create: `tauri-desktop/react-ui/src/routes/diagnostics/index.tsx`
- Modify: `tauri-desktop/react-ui/src/features/navigation/navLinks.ts`
- Modify: `tauri-desktop/react-ui/src/features/navigation/navLinks.test.ts`

**Interfaces:**
- Consumes: Task 9 debug/log commands and Task 11 `/rules` editor route.
- Produces: `/diagnostics`, log copy/clear, live AX tree, timing display, and navigation state for prefilled condition creation.

- [ ] **Step 1: Write diagnostic transformation tests**

```typescript
it("maps an ancestor DOM identifier to a contains condition", () => {
  expect(toCondition(snapshotAttribute(2, "domIdentifier", "workbench.panel.chat"))).toEqual({
    attribute: "domIdentifier",
    operator: "contains",
    scope: "ancestors",
    value: "workbench.panel.chat",
  });
});
```

Test focused depth mapping, empty-attribute exclusion, TSV log copy output, success/failure labels, and p50/p95 formatting.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/inputSourceDiagnostics`  
Expected: FAIL because diagnostics are absent.

- [ ] **Step 3: Implement diagnostics UI and capture lifecycle**

Start debug capture on page mount and stop in effect cleanup. Render logs and AX hierarchy without persisting values. Use `navigator.clipboard.writeText` behind a user action and surface failure. “条件に使用” navigates to `/rules` with a typed prefill state; do not store raw AX text globally after leaving diagnostics.

```typescript
useEffect(() => {
  void startDebugCapture();
  return () => {
    void stopDebugCapture();
  };
}, []);

const useAsCondition = (attribute: SnapshotAttribute) => {
  void navigate({ to: "/rules", state: { conditionPrefill: toCondition(attribute) } });
};
```

- [ ] **Step 4: Generate route tree and verify**

Run: `cd tauri-desktop/react-ui && pnpm exec vite build && pnpm test -- src/features/inputSourceDiagnostics && pnpm typecheck`  
Expected: route is generated; tests and type checking pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/react-ui/src/features/inputSourceDiagnostics tauri-desktop/react-ui/src/features/navigation tauri-desktop/react-ui/src/routes/diagnostics tauri-desktop/react-ui/src/routeTree.gen.ts
git commit -m "入力ソース診断画面を追加"
```

### Task 13: Functional settings page

**Files:**
- Modify: `tauri-desktop/react-ui/src/features/settings/SettingsPage.tsx`
- Modify: `tauri-desktop/react-ui/src/features/settings/SettingsPage.styles.ts`
- Create: `tauri-desktop/react-ui/src/features/settings/inputSourceOptions.ts`
- Create: `tauri-desktop/react-ui/src/features/settings/inputSourceOptions.test.ts`
- Modify: `tauri-desktop/react-ui/src/features/settings/index.ts`

**Interfaces:**
- Consumes: Task 9 input-source/autostart/settings commands and existing theme controls.
- Produces: preferred English/Japanese selectors, unavailable source warning, autostart toggle, debug logging toggle, and reset action.

- [ ] **Step 1: Write input option tests**

```typescript
it("keeps a removed configured source visible and unavailable", () => {
  expect(buildInputSourceOptions([{ id: "en", name: "ABC" }], "removed")).toContainEqual({
    value: "removed", label: "removed（利用不可）", available: false,
  });
});
```

Test duplicate IDs, localized display-name fallback, selected-value preservation, and deterministic sorting.

- [ ] **Step 2: Confirm RED**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/settings/inputSourceOptions.test.ts`  
Expected: FAIL because the option builder does not exist.

- [ ] **Step 3: Replace placeholder settings with functional sections**

Keep the real theme and About navigation controls. Remove placeholder account/language/week/share/feedback controls. Add input-source selectors showing display name and ID, autostart, debug logging, and confirmed defaults reset. Disable/save-error controls explicitly while commands are pending.

```tsx
<SettingItem
  icon={MdKeyboard}
  title="英数入力ソース"
  type="selection"
  value={selectedEnglishLabel}
  options={englishOptions.map((option) => option.label)}
  onSelectionChanged={selectPreferredEnglish}
/>
<SettingItem
  icon={MdRocketLaunch}
  title="ログイン時に起動"
  type="toggle"
  toggleValue={autostartEnabled}
  onToggleChanged={(enabled) => void updateAutostart(enabled)}
/>
```

- [ ] **Step 4: Verify settings**

Run: `cd tauri-desktop/react-ui && pnpm test -- src/features/settings && pnpm typecheck && pnpm lint`  
Expected: tests, types, and lint pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-desktop/react-ui/src/features/settings
git commit -m "入力ソースと起動設定を設定画面へ追加"
```

### Task 14: Integration documentation and full verification

**Files:**
- Create: `docs/input-source-switcher-manual-test.md`
- Create: `docs/input-source-rule-maintenance.md`
- Modify: `tauri-desktop/react-ui/README.md`

**Interfaces:**
- Consumes: all prior tasks.
- Produces: reproducible permission/setup instructions, manual E2E matrix, matcher maintenance procedure, and recorded performance results.

- [ ] **Step 1: Write the manual E2E matrix before running it**

Include exact rows for Terminal activation; Cursor/VS Code terminal, chat, rename, editor; arbitrary custom app rule; manual override; permission denied/granted/revoked; fallback monitoring; close-to-tray; snooze expiry; login launch; missing input source; and diagnostics copy/capture cleanup. Each row has precondition, action, expected rule/source, actual result, and latency field.

- [ ] **Step 2: Run Rust quality gates**

Run:

```bash
cd tauri-desktop/rust-native
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Expected: all commands exit 0 with no warnings.

- [ ] **Step 3: Run frontend quality gates**

Run:

```bash
cd tauri-desktop/react-ui
pnpm fix
pnpm typecheck
pnpm test
pnpm build
```

Expected: all commands exit 0; only generated files change as expected.

- [ ] **Step 4: Run macOS manual E2E and record evidence**

Run `mise run dev`, grant Accessibility permission, execute every matrix row, and paste actual outcomes plus p50/p95 values into `docs/input-source-switcher-manual-test.md`. If a required external app is unavailable, mark only that row `Not run — <app> unavailable`; do not report it as passing.

- [ ] **Step 5: Document rule maintenance**

Describe how to open Diagnostics, freeze/copy the focused AX hierarchy, identify the smallest stable attribute, create a contains/equals condition, order the rule, and validate it without exposing captured user text. Document the default bundle IDs and reset behavior.

- [ ] **Step 6: Review the final diff and commit**

Run: `git diff --check && git status --short && git diff --stat HEAD~13..HEAD`  
Expected: no whitespace errors or unrelated files; then commit documentation:

```bash
git add docs/input-source-switcher-manual-test.md docs/input-source-rule-maintenance.md tauri-desktop/react-ui/README.md
git commit -m "入力ソース切替の検証手順を追加"
```
