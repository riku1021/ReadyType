//! プロセスが起きている間の更新確認。
//!
//! ログイン起動ではメイン画面が非表示のままなので、確認タイマーは WebView に置かない。

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{
    application::{
        app_update::{AppUpdateSession, UpdatePhase},
        switcher_controller::SwitcherController,
    },
    infrastructure::{desktop::tray, persistence::AppUpdateScheduleStore},
};

pub const STATE_EVENT: &str = "readytype://app-update";
const TICK: Duration = Duration::from_secs(60);
const CLIENT_ACK_WAIT: Duration = Duration::from_secs(5);
const CLIENT_ACK_POLL: Duration = Duration::from_millis(100);
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
/// プラグインは確認用 timeout をダウンロードへコピーしない。取得全体の上限。
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10 * 60);

struct PendingUpdate {
    update: Update,
    bytes: Option<Vec<u8>>,
}

pub struct AppUpdateRuntime {
    app: AppHandle,
    store: AppUpdateScheduleStore,
    session: Mutex<AppUpdateSession>,
    pending: Mutex<Option<PendingUpdate>>,
    operation: tauri::async_runtime::Mutex<()>,
}

impl AppUpdateRuntime {
    pub fn load(app: AppHandle, store: AppUpdateScheduleStore) -> Arc<Self> {
        let loaded = store.load();
        Arc::new(Self {
            app,
            store,
            session: Mutex::new(AppUpdateSession::new(
                loaded.schedule,
                loaded.loaded_from_disk,
            )),
            pending: Mutex::new(None),
            operation: tauri::async_runtime::Mutex::new(()),
        })
    }

    pub fn spawn(self: &Arc<Self>) {
        let runtime = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            runtime.wait_for_client_ack().await;
            runtime.check(CheckKind::Scheduled).await;
            loop {
                sleep_for(TICK).await;
                runtime.check(CheckKind::Scheduled).await;
            }
        });
    }

    pub fn tray_install_ready(&self) -> bool {
        self.session().tray_install_ready()
    }

    pub fn snapshot(&self) -> AppUpdateSnapshot {
        AppUpdateSnapshot::from_session(&self.session())
    }

    pub async fn accept_client(
        &self,
        legacy: Option<LegacyUpdatePreferences>,
    ) -> AppUpdateClientAck {
        let decision = {
            let mut session = self.session();
            let decision = match legacy {
                Some(legacy) => session.accept_client(
                    legacy.auto_check,
                    legacy.auto_download,
                    legacy.last_checked_at_unix_ms,
                ),
                None => session.accept_client(None, None, None),
            };
            if decision.persist {
                self.persist(&session);
            }
            decision
        };
        let snapshot = self.snapshot();
        self.publish(&snapshot);
        AppUpdateClientAck {
            clear_legacy: decision.clear_legacy,
            snapshot,
        }
    }

    pub async fn set_auto_check(&self, enabled: bool) -> AppUpdateSnapshot {
        {
            let mut session = self.session();
            session.set_auto_check(enabled);
            self.persist(&session);
        }
        self.publish(&self.snapshot());
        if enabled {
            self.check(CheckKind::Immediate).await;
        }
        self.snapshot()
    }

    pub async fn set_auto_download(&self, enabled: bool) -> AppUpdateSnapshot {
        let should_download = {
            let mut session = self.session();
            let should_download = session.set_auto_download(enabled);
            self.persist(&session);
            should_download
        };
        self.publish(&self.snapshot());
        if should_download {
            self.download_pending().await;
        }
        self.snapshot()
    }

    pub async fn check_now(&self) -> AppUpdateSnapshot {
        self.check(CheckKind::Immediate).await;
        self.snapshot()
    }

    pub async fn install(&self) -> AppUpdateSnapshot {
        let _operation = self.operation.lock().await;
        if !self.package_is_ready() {
            self.download_pending_locked().await;
        }
        if !self.package_is_ready() {
            return self.snapshot();
        }
        {
            let mut session = self.session();
            session.begin_install();
            drop(session);
            self.publish(&self.snapshot());
            self.refresh_tray();
        }
        let install_result = {
            let mut pending = self.pending();
            match pending.as_mut() {
                Some(pending) => pending
                    .bytes
                    .take()
                    .map(|bytes| (pending.update.clone(), bytes)),
                None => None,
            }
        };
        match install_result {
            Some((update, bytes)) => match update.install(&bytes) {
                Ok(()) => {
                    self.app.restart();
                }
                Err(error) => {
                    eprintln!("app-update: install failed: {error}");
                    if let Some(pending) = self.pending().as_mut() {
                        pending.bytes = Some(bytes);
                    }
                    let mut session = self.session();
                    session.note_install_failed();
                }
            },
            None => {
                let mut session = self.session();
                session.note_install_failed();
            }
        }
        let snapshot = self.snapshot();
        self.publish(&snapshot);
        self.refresh_tray();
        snapshot
    }

    async fn wait_for_client_ack(&self) {
        let started = std::time::Instant::now();
        while started.elapsed() < CLIENT_ACK_WAIT {
            if self.session().sealed {
                return;
            }
            sleep_for(CLIENT_ACK_POLL).await;
        }
        let should_persist = {
            let mut session = self.session();
            session.release_without_client()
        };
        if should_persist {
            self.persist(&self.session());
        }
    }

    async fn check(&self, kind: CheckKind) {
        let _operation = self.operation.lock().await;
        let now = now_unix_ms();
        {
            let session = self.session();
            let allowed = match kind {
                CheckKind::Scheduled => session.scheduled_check_due(now),
                CheckKind::Immediate => true,
            };
            if !allowed {
                return;
            }
        }
        {
            let mut session = self.session();
            session.begin_check();
        }
        *self.pending() = None;
        self.publish(&self.snapshot());

        let checked = match self.app.updater_builder().timeout(CHECK_TIMEOUT).build() {
            Ok(updater) => updater.check().await,
            Err(error) => {
                eprintln!("app-update: updater unavailable: {error}");
                Err(error)
            }
        };
        let now = now_unix_ms();
        match checked {
            Ok(Some(update)) => {
                let version = update.version.clone();
                let auto_download = {
                    let mut session = self.session();
                    session.note_available(now, version);
                    let auto_download = session.schedule.auto_download;
                    self.persist(&session);
                    auto_download
                };
                *self.pending() = Some(PendingUpdate {
                    update,
                    bytes: None,
                });
                self.publish(&self.snapshot());
                if auto_download {
                    self.download_pending_locked().await;
                }
            }
            Ok(None) => {
                {
                    let mut session = self.session();
                    session.note_up_to_date(now);
                    self.persist(&session);
                }
                self.publish(&self.snapshot());
                self.refresh_tray();
            }
            Err(error) => {
                eprintln!("app-update: check failed: {error}");
                {
                    let mut session = self.session();
                    session.note_check_failed(now);
                    self.persist(&session);
                }
                self.publish(&self.snapshot());
            }
        }
    }

    async fn download_pending(&self) {
        let _operation = self.operation.lock().await;
        self.download_pending_locked().await;
    }

    async fn download_pending_locked(&self) {
        let update = {
            let pending = self.pending();
            pending.as_ref().map(|pending| pending.update.clone())
        };
        let Some(mut update) = update else {
            return;
        };
        if self.package_is_ready() {
            return;
        }
        update.timeout = Some(DOWNLOAD_TIMEOUT);
        {
            let mut session = self.session();
            session.begin_download();
        }
        self.publish(&self.snapshot());
        match update.download(|_, _| {}, || {}).await {
            Ok(bytes) => {
                if let Some(pending) = self.pending().as_mut() {
                    pending.bytes = Some(bytes);
                }
                {
                    let mut session = self.session();
                    session.note_downloaded();
                }
                self.publish(&self.snapshot());
                self.refresh_tray();
            }
            Err(error) => {
                eprintln!("app-update: download failed: {error}");
                {
                    let mut session = self.session();
                    session.note_download_failed();
                }
                self.publish(&self.snapshot());
            }
        }
    }

    fn package_is_ready(&self) -> bool {
        self.pending()
            .as_ref()
            .is_some_and(|pending| pending.bytes.is_some())
    }

    fn persist(&self, session: &AppUpdateSession) {
        if let Err(error) = self.store.save(&session.schedule) {
            eprintln!("app-update: save failed: {error}");
        }
    }

    fn publish(&self, snapshot: &AppUpdateSnapshot) {
        let _ = self.app.emit(STATE_EVENT, snapshot);
    }

    fn refresh_tray(&self) {
        let Some(controller) = self.app.try_state::<Arc<SwitcherController>>() else {
            return;
        };
        let Ok(state) = controller.state() else {
            return;
        };
        let locale = controller.resolved_ui_locale();
        let _ = tray::update(&self.app, &state, &locale);
    }

    fn session(&self) -> std::sync::MutexGuard<'_, AppUpdateSession> {
        self.session
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn pending(&self) -> std::sync::MutexGuard<'_, Option<PendingUpdate>> {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

enum CheckKind {
    Scheduled,
    Immediate,
}

async fn sleep_for(duration: Duration) {
    let _ = tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(duration);
    })
    .await;
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[derive(Debug, Clone)]
pub struct LegacyUpdatePreferences {
    pub auto_check: Option<bool>,
    pub auto_download: Option<bool>,
    pub last_checked_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateSnapshot {
    pub auto_check: bool,
    pub auto_download: bool,
    pub last_checked_at_unix_ms: Option<u64>,
    pub phase: &'static str,
    pub version: Option<String>,
    pub install_ready: bool,
    pub error_key: Option<&'static str>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateClientAck {
    pub clear_legacy: bool,
    #[serde(flatten)]
    pub snapshot: AppUpdateSnapshot,
}

impl AppUpdateSnapshot {
    fn from_session(session: &AppUpdateSession) -> Self {
        let (phase, error_key) = match &session.phase {
            UpdatePhase::Idle => ("idle", None),
            UpdatePhase::Checking => ("checking", None),
            UpdatePhase::UpToDate => ("upToDate", None),
            UpdatePhase::Available { .. } => ("available", None),
            UpdatePhase::Downloading { .. } => ("downloading", None),
            UpdatePhase::Downloaded { .. } => ("downloaded", None),
            UpdatePhase::Installing { .. } => ("installing", None),
            UpdatePhase::CheckFailed => ("checkFailed", Some("checkFailed")),
            UpdatePhase::DownloadFailed { .. } => ("downloadFailed", Some("downloadFailed")),
            UpdatePhase::InstallFailed { .. } => ("installFailed", Some("installFailed")),
        };
        Self {
            auto_check: session.schedule.auto_check,
            auto_download: session.schedule.auto_download,
            last_checked_at_unix_ms: session.schedule.last_checked_at_unix_ms,
            phase,
            version: session.phase.version().map(str::to_owned),
            install_ready: session.install_ready(),
            error_key,
        }
    }
}
