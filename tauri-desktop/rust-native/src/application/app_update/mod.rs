//! 自動アップデートの確認間隔。
//!
//! 成功した確認の次は 24 時間後。失敗した確認は最終確認時刻を変えず、1 時間後に再試行する。
//! 未インストールの更新が残っているプロセス起動時だけ、間隔を待たずに取り直す。

pub const SUCCESS_INTERVAL_MS: u64 = 24 * 60 * 60 * 1000;
pub const FAILURE_RETRY_MS: u64 = 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSchedule {
    pub auto_check: bool,
    pub auto_download: bool,
    pub last_checked_at_unix_ms: Option<u64>,
    pub next_check_at_unix_ms: Option<u64>,
    /// 見つかったが未インストールの版。プロセスをまたいで保持する。
    pub pending_version: Option<String>,
    /// 画面が旧設定を渡した、または自動確認の切り替えを保存した。
    /// 偽の間は、後から来た旧設定のオン／オフをまだ取り込む。
    pub client_acked: bool,
}

impl UpdateSchedule {
    pub fn defaults() -> Self {
        Self {
            auto_check: true,
            auto_download: true,
            last_checked_at_unix_ms: None,
            next_check_at_unix_ms: None,
            pending_version: None,
            client_acked: false,
        }
    }

    pub fn is_due(&self, now_unix_ms: u64, force_pending_redownload: bool) -> bool {
        if !self.auto_check {
            return false;
        }
        if force_pending_redownload && self.pending_version.is_some() {
            return true;
        }
        match self.next_check_at_unix_ms {
            None => true,
            Some(next) => now_unix_ms >= next,
        }
    }

    pub fn record_success(&mut self, now_unix_ms: u64) {
        self.last_checked_at_unix_ms = Some(now_unix_ms);
        self.next_check_at_unix_ms = Some(now_unix_ms.saturating_add(SUCCESS_INTERVAL_MS));
    }

    pub fn record_failure(&mut self, now_unix_ms: u64) {
        self.next_check_at_unix_ms = Some(now_unix_ms.saturating_add(FAILURE_RETRY_MS));
    }

    /// 画面の localStorage にだけ残っている旧設定を、ファイルが無い初回だけ取り込む。
    pub fn adopt_legacy(
        &mut self,
        auto_check: Option<bool>,
        auto_download: Option<bool>,
        last_checked_at_unix_ms: Option<u64>,
    ) {
        if let Some(enabled) = auto_check {
            self.auto_check = enabled;
        }
        if let Some(enabled) = auto_download {
            self.auto_download = enabled;
        }
        if let Some(last) = last_checked_at_unix_ms {
            self.last_checked_at_unix_ms = Some(last);
            self.next_check_at_unix_ms = Some(last.saturating_add(SUCCESS_INTERVAL_MS));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatePhase {
    Idle,
    Checking,
    UpToDate,
    Available { version: String },
    Downloading { version: String },
    Downloaded { version: String },
    Installing { version: String },
    CheckFailed,
    DownloadFailed { version: String },
    InstallFailed { version: String },
}

impl UpdatePhase {
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::Available { version }
            | Self::Downloading { version }
            | Self::Downloaded { version }
            | Self::Installing { version }
            | Self::DownloadFailed { version }
            | Self::InstallFailed { version } => Some(version),
            Self::Idle | Self::Checking | Self::UpToDate | Self::CheckFailed => None,
        }
    }
}

/// 画面とトレイが共有する、ネットワークを持たない更新セッション。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppUpdateSession {
    pub schedule: UpdateSchedule,
    pub phase: UpdatePhase,
    /// このプロセスで、残っていた更新をまだ取り直していない。
    pub force_pending_redownload: bool,
    /// このプロセスのメモリに検証済みパッケージがある。
    pub package_ready: bool,
    /// 設定ファイルがある、またはスケジューラを進めてよい。
    pub sealed: bool,
}

impl AppUpdateSession {
    pub fn new(schedule: UpdateSchedule, loaded_from_disk: bool) -> Self {
        let force_pending_redownload = schedule.pending_version.is_some();
        Self {
            schedule,
            phase: UpdatePhase::Idle,
            force_pending_redownload,
            package_ready: false,
            sealed: loaded_from_disk,
        }
    }

    pub fn scheduled_check_due(&self, now_unix_ms: u64) -> bool {
        self.schedule
            .is_due(now_unix_ms, self.force_pending_redownload)
    }

    pub fn begin_check(&mut self) {
        self.force_pending_redownload = false;
        self.package_ready = false;
        self.phase = UpdatePhase::Checking;
    }

    pub fn note_up_to_date(&mut self, now_unix_ms: u64) {
        self.schedule.pending_version = None;
        self.schedule.record_success(now_unix_ms);
        self.package_ready = false;
        self.phase = UpdatePhase::UpToDate;
    }

    pub fn note_available(&mut self, now_unix_ms: u64, version: String) {
        self.schedule.pending_version = Some(version.clone());
        self.schedule.record_success(now_unix_ms);
        self.package_ready = false;
        self.phase = UpdatePhase::Available { version };
    }

    pub fn note_check_failed(&mut self, now_unix_ms: u64) {
        self.schedule.record_failure(now_unix_ms);
        self.phase = UpdatePhase::CheckFailed;
    }

    pub fn begin_download(&mut self) {
        let Some(version) = self.phase.version().map(str::to_owned) else {
            return;
        };
        self.phase = UpdatePhase::Downloading { version };
    }

    pub fn note_downloaded(&mut self) {
        let Some(version) = self.phase.version().map(str::to_owned) else {
            return;
        };
        self.package_ready = true;
        self.phase = UpdatePhase::Downloaded { version };
    }

    pub fn note_download_failed(&mut self) {
        let Some(version) = self.schedule.pending_version.clone() else {
            self.phase = UpdatePhase::CheckFailed;
            return;
        };
        self.package_ready = false;
        self.phase = UpdatePhase::DownloadFailed { version };
    }

    pub fn begin_install(&mut self) {
        let Some(version) = self.phase.version().map(str::to_owned) else {
            return;
        };
        self.phase = UpdatePhase::Installing { version };
    }

    pub fn note_install_failed(&mut self) {
        let Some(version) = self.schedule.pending_version.clone() else {
            return;
        };
        self.phase = UpdatePhase::InstallFailed { version };
    }

    pub fn set_auto_check(&mut self, enabled: bool) {
        self.schedule.auto_check = enabled;
        self.schedule.client_acked = true;
        self.sealed = true;
    }

    pub fn set_auto_download(&mut self, enabled: bool) -> bool {
        self.schedule.auto_download = enabled;
        enabled && matches!(self.phase, UpdatePhase::Available { .. }) && !self.package_ready
    }

    /// 更新画面のボタン。未ダウンロードでも、見つかっていればインストール操作に進める。
    pub fn install_ready(&self) -> bool {
        if self.package_ready {
            return !matches!(self.phase, UpdatePhase::Installing { .. });
        }
        matches!(self.phase, UpdatePhase::Available { .. })
    }

    /// トレイは、再起動だけで入れられるときだけ出す。
    pub fn tray_install_ready(&self) -> bool {
        self.package_ready && !matches!(self.phase, UpdatePhase::Installing { .. })
    }

    /// 画面の ack を待たずにスケジューラを進める。旧設定はまだ受け取れる。
    pub fn release_without_client(&mut self) -> bool {
        if self.sealed {
            return false;
        }
        self.sealed = true;
        true
    }

    /// 旧 localStorage を取り込む。確認が既に成功しているときは最終確認時刻を戻さない。
    pub fn accept_client(
        &mut self,
        auto_check: Option<bool>,
        auto_download: Option<bool>,
        last_checked_at_unix_ms: Option<u64>,
    ) -> ClientLegacyDecision {
        if self.schedule.client_acked {
            return ClientLegacyDecision {
                persist: false,
                clear_legacy: true,
            };
        }
        let last_checked_at_unix_ms = if self.schedule.last_checked_at_unix_ms.is_none() {
            last_checked_at_unix_ms
        } else {
            None
        };
        self.schedule
            .adopt_legacy(auto_check, auto_download, last_checked_at_unix_ms);
        self.schedule.client_acked = true;
        self.force_pending_redownload = self.schedule.pending_version.is_some();
        self.sealed = true;
        ClientLegacyDecision {
            persist: true,
            clear_legacy: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientLegacyDecision {
    pub persist: bool,
    pub clear_legacy: bool,
}

#[cfg(test)]
mod tests;
