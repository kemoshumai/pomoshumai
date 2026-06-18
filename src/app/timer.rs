use std::time::{Duration, Instant};

use super::state::{Phase, Settings, TimerStatus};

#[derive(Debug, Clone)]
pub struct TimerState {
    pub phase: Phase,
    pub status: TimerStatus,
    pub remaining: Duration,
    pub session_duration: Duration,
    end_at: Option<Instant>,
}

impl TimerState {
    pub fn new(settings: &Settings) -> Self {
        let session_duration = settings.duration_for_phase(Phase::Work);
        Self {
            phase: Phase::Work,
            status: TimerStatus::Idle,
            remaining: session_duration,
            session_duration,
            end_at: None,
        }
    }

    pub fn start(&mut self, now: Instant) {
        if self.status == TimerStatus::Running {
            return;
        }
        self.status = TimerStatus::Running;
        self.end_at = Some(now + self.remaining);
    }

    pub fn pause(&mut self, now: Instant) {
        if self.status != TimerStatus::Running {
            return;
        }
        if let Some(end_at) = self.end_at {
            self.remaining = end_at.saturating_duration_since(now);
        }
        self.end_at = None;
        self.status = TimerStatus::Paused;
    }

    pub fn cancel(&mut self, settings: &Settings) {
        self.phase = Phase::Work;
        self.status = TimerStatus::Idle;
        self.session_duration = settings.duration_for_phase(Phase::Work);
        self.remaining = self.session_duration;
        self.end_at = None;
    }

    pub fn set_phase(&mut self, phase: Phase, settings: &Settings) {
        self.phase = phase;
        self.status = TimerStatus::Idle;
        self.session_duration = settings.duration_for_phase(phase);
        self.remaining = self.session_duration;
        self.end_at = None;
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        if self.status != TimerStatus::Running {
            return false;
        }

        let Some(end_at) = self.end_at else {
            return false;
        };

        if now >= end_at {
            self.remaining = Duration::ZERO;
            self.status = TimerStatus::Idle;
            self.end_at = None;
            return true;
        }

        self.remaining = end_at.saturating_duration_since(now);
        false
    }
}
