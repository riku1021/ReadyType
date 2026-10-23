use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::application::app_update::UpdateSchedule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedUpdateSchedule {
    pub schedule: UpdateSchedule,
    pub loaded_from_disk: bool,
}

#[derive(Debug)]
pub struct AppUpdateScheduleStore {
    path: PathBuf,
}

impl AppUpdateScheduleStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> LoadedUpdateSchedule {
        let Some(bytes) = fs::read(&self.path).ok() else {
            return LoadedUpdateSchedule {
                schedule: UpdateSchedule::defaults(),
                loaded_from_disk: false,
            };
        };
        match serde_json::from_slice::<UpdateScheduleDto>(&bytes) {
            Ok(dto) => LoadedUpdateSchedule {
                schedule: dto.into_schedule(),
                loaded_from_disk: true,
            },
            Err(error) => {
                eprintln!("app-update: ignore unreadable schedule: {error}");
                LoadedUpdateSchedule {
                    schedule: UpdateSchedule::defaults(),
                    loaded_from_disk: false,
                }
            }
        }
    }

    pub fn save(&self, schedule: &UpdateSchedule) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(&UpdateScheduleDto::from_schedule(schedule))
            .map_err(|error| error.to_string())?;
        let temporary = temporary_path(&self.path);
        {
            let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
            file.write_all(&bytes).map_err(|error| error.to_string())?;
            file.sync_all().map_err(|error| error.to_string())?;
        }
        fs::rename(&temporary, &self.path).map_err(|error| error.to_string())
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    PathBuf::from(temporary)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateScheduleDto {
    #[serde(default = "default_true")]
    auto_check: bool,
    #[serde(default = "default_true")]
    auto_download: bool,
    #[serde(default)]
    last_checked_at_unix_ms: Option<u64>,
    #[serde(default)]
    next_check_at_unix_ms: Option<u64>,
    #[serde(default)]
    pending_version: Option<String>,
    #[serde(default)]
    client_acked: bool,
}

fn default_true() -> bool {
    true
}

impl UpdateScheduleDto {
    fn from_schedule(schedule: &UpdateSchedule) -> Self {
        Self {
            auto_check: schedule.auto_check,
            auto_download: schedule.auto_download,
            last_checked_at_unix_ms: schedule.last_checked_at_unix_ms,
            next_check_at_unix_ms: schedule.next_check_at_unix_ms,
            pending_version: schedule.pending_version.clone(),
            client_acked: schedule.client_acked,
        }
    }

    fn into_schedule(self) -> UpdateSchedule {
        UpdateSchedule {
            auto_check: self.auto_check,
            auto_download: self.auto_download,
            last_checked_at_unix_ms: self.last_checked_at_unix_ms,
            next_check_at_unix_ms: self.next_check_at_unix_ms,
            pending_version: self.pending_version.filter(|version| !version.is_empty()),
            client_acked: self.client_acked,
        }
    }
}
