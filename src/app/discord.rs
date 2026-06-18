use std::time::{Duration, SystemTime, UNIX_EPOCH};

use discord_presence::Client;
use discord_presence::models::rich_presence::{Activity, ActivityType, DisplayType};

use crate::app::state::{Phase, TimerStatus};

use super::state::Settings;
use super::timer::TimerState;

const WORK_RUNNING_IMAGE: &str = "pomoshumai_work_running";
const BREAK_RUNNING_IMAGE: &str = "pomoshumai_break_running";
const WORK_IDLE_IMAGE: &str = "pomoshumai_work_idle";
const BREAK_IDLE_IMAGE: &str = "pomoshumai_break_idle";
const PAUSED_IMAGE: &str = "pomoshumai_paused";

pub struct DiscordPresence {
    client: Option<Client>,
}

impl DiscordPresence {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn update(
        &mut self,
        settings: &Settings,
        timer: &TimerState,
        discord_status: &str,
        completed_pomodoros: u32,
    ) {
        if !settings.discord_presence {
            self.shutdown();
            return;
        }

        let Some(client_id) = read_client_id() else {
            return;
        };

        prepare_discord_ipc();

        let client = self.client.get_or_insert_with(|| {
            let mut client = Client::new(client_id);
            client.start();
            client
        });

        client.queue_activity(|activity| {
            build_activity(activity, timer, discord_status, completed_pomodoros)
        });
    }

    fn shutdown(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.clear_activity();
            let _ = client.shutdown();
        }
    }
}

fn discord_image_key(timer: &TimerState) -> &'static str {
    match timer.status {
        TimerStatus::Paused => PAUSED_IMAGE,
        TimerStatus::Running => match timer.phase {
            Phase::Work => WORK_RUNNING_IMAGE,
            Phase::ShortBreak | Phase::LongBreak => BREAK_RUNNING_IMAGE,
        },
        TimerStatus::Idle => match timer.phase {
            Phase::Work => WORK_IDLE_IMAGE,
            Phase::ShortBreak | Phase::LongBreak => BREAK_IDLE_IMAGE,
        },
    }
}

fn build_activity(
    activity: Activity,
    timer: &TimerState,
    discord_status: &str,
    completed_pomodoros: u32,
) -> Activity {
    let mut activity = activity
        .activity_type(ActivityType::Competing)
        .status_display(DisplayType::Name)
        .name(format!("今日の成果：{}ポモ", completed_pomodoros))
        .details(discord_status)
        .state(format!("現在{}ポモドーロ作業済み", completed_pomodoros))
        .assets(|assets| {
            assets
                .large_image(discord_image_key(timer))
                .large_text(discord_status)
        });

    if timer.status == TimerStatus::Running {
        let (start_ts, end_ts) = build_timestamps(timer);
        activity = activity
            .activity_type(ActivityType::Watching)
            .name(format!(
                "今日の成果：{}ポモ（{}）",
                completed_pomodoros, discord_status
            ))
            .timestamps(|timestamps| timestamps.start(start_ts).end(end_ts));
    }

    activity
}

fn read_client_id() -> Option<u64> {
    Some(1500051633709514812)
}

#[cfg(target_os = "macos")]
fn prepare_discord_ipc() {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    let Some(ipc_root) = discord_presence_ipc_root() else {
        return;
    };

    if find_discord_ipc_paths(&ipc_root).next().is_some() {
        return;
    }

    let mut candidate_roots = BTreeSet::new();
    if let Ok(path) = std::env::var("XDG_RUNTIME_DIR") {
        candidate_roots.insert(PathBuf::from(path));
    }
    if let Ok(path) = std::env::var("TMPDIR") {
        candidate_roots.insert(PathBuf::from(path));
    }
    candidate_roots.insert(std::env::temp_dir());
    candidate_roots.insert(PathBuf::from("/tmp"));
    candidate_roots.insert(PathBuf::from("/private/tmp"));
    candidate_roots.insert(PathBuf::from("/var/tmp"));

    for actual_root in candidate_roots {
        if actual_root == ipc_root {
            continue;
        }

        let mut linked = false;
        for actual_path in find_discord_ipc_paths(&actual_root) {
            let Some(file_name) = actual_path.file_name() else {
                continue;
            };
            let link_path = ipc_root.join(file_name);
            if link_path.exists() {
                continue;
            }

            if std::os::unix::fs::symlink(&actual_path, &link_path).is_ok() {
                linked = true;
            }
        }

        if linked {
            return;
        }
    }
}

#[cfg(target_os = "macos")]
fn discord_presence_ipc_root() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("XDG_RUNTIME_DIR") {
        return Some(std::path::PathBuf::from(path));
    }

    if let Ok(path) = std::env::var("TMPDIR") {
        return Some(std::path::PathBuf::from(path));
    }

    Some(std::env::temp_dir())
}

#[cfg(target_os = "macos")]
fn find_discord_ipc_paths(root: &std::path::Path) -> impl Iterator<Item = std::path::PathBuf> + '_ {
    (0..10)
        .map(|index| root.join(format!("discord-ipc-{index}")))
        .filter(|path| path.exists())
}

#[cfg(not(target_os = "macos"))]
fn prepare_discord_ipc() {}

fn build_timestamps(timer: &TimerState) -> (u64, u64) {
    let now = SystemTime::now();
    let total_secs = timer.session_duration.as_secs();
    let remaining_secs = timer.remaining.as_secs().min(total_secs);
    let elapsed_secs = total_secs.saturating_sub(remaining_secs);

    let start = now
        .checked_sub(Duration::from_secs(elapsed_secs))
        .unwrap_or(now);
    let end = now
        .checked_add(Duration::from_secs(remaining_secs))
        .unwrap_or(now);

    let start_ts = start
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let end_ts = end.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    (start_ts, end_ts)
}
