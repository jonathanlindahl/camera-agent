use crate::types::{Action, DegradedReason, Observation, SystemState};

pub struct CameraAgent {
    state: SystemState,
    error_count: u32,
    degraded_reason: Option<DegradedReason>,
    degraded_cycles: u32,
}

impl CameraAgent {
    pub fn new() -> Self {
        Self {
            state: SystemState::Idle,
            error_count: 0,
            degraded_reason: None,
            degraded_cycles: 0,
        }
    }

    pub fn current_state(&self) -> SystemState {
        self.state
    }

    pub fn step(&mut self, obs: Observation) -> Action {
        let (next_state, action, degraded_reason, reset_cycles) =
            transition(self.state, obs, self.degraded_reason, self.degraded_cycles);

        self.state = next_state;
        self.degraded_reason = degraded_reason;
        self.degraded_cycles = if reset_cycles {
            0
        } else {
            self.degraded_cycles + 1
        };
        action
    }
}

fn transition(
    state: SystemState,
    obs: Observation,
    degraded_reason: Option<DegradedReason>,
    degraded_cycles: u32,
) -> (SystemState, Action, Option<DegradedReason>, bool) {
    const CPU_DEGRADE: u8 = 85;
    const CPU_RECOVER: u8 = 60;
    const MOTION_THRESHOLD: u8 = 30;
    const RECORD_THRESHOLD: u8 = 50;
    const ALERT_CONFIDENCE: u8 = 80;
    const RECOVERY_CYCLES: u32 = 3; // number of stables cycles to recover

    // global degraded override
    if obs.cpu_load > CPU_DEGRADE {
        return (
            SystemState::Degraded,
            Action::EnterDegradedMode,
            Some(DegradedReason::HighCpu),
            true, // reset cycles
        );
    }

    if !obs.detector_healthy {
        return (
            SystemState::Degraded,
            Action::EnterDegradedMode,
            Some(DegradedReason::DetectorFailure),
            true, // reset cycles
        );
    }

    // degraded state recovery
    if state == SystemState::Degraded {
        if degraded_cycles + 1 >= RECOVERY_CYCLES
            && obs.cpu_load < CPU_RECOVER
            && obs.detector_healthy
        {
            return (
                SystemState::Monitoring,
                Action::ExitDegradedMode,
                None,
                true, // reset cycles
            );
        } else {
            return (SystemState::Degraded, Action::None, degraded_reason, false);
        }
    }

    // normal state transitions
    match state {
        SystemState::Idle => {
            if obs.motion_level > MOTION_THRESHOLD {
                (SystemState::Monitoring, Action::None, degraded_reason, true)
            } else {
                (SystemState::Idle, Action::None, degraded_reason, true)
            }
        }
        SystemState::Monitoring => {
            if obs.motion_level > 50 && obs.object_detected {
                (
                    SystemState::Recording,
                    Action::StartRecording,
                    degraded_reason,
                    true,
                )
            } else if obs.motion_level <= 30 {
                (SystemState::Idle, Action::None, degraded_reason, true)
            } else {
                (SystemState::Monitoring, Action::None, degraded_reason, true)
            }
        }
        SystemState::Recording => {
            if obs.confidence > 80 {
                (
                    SystemState::Alerting,
                    Action::SendAlert,
                    degraded_reason,
                    true,
                )
            } else if obs.motion_level <= 30 {
                (
                    SystemState::Monitoring,
                    Action::StopRecording,
                    degraded_reason,
                    true,
                )
            } else {
                (SystemState::Recording, Action::None, degraded_reason, true)
            }
        }
        SystemState::Alerting => {
            // Always return to Recording after one cycle
            (SystemState::Recording, Action::None, degraded_reason, true)
        }
        SystemState::Degraded => {
            // Recovery handled later
            (SystemState::Degraded, Action::None, degraded_reason, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, Observation, SystemState};

    #[test]
    fn idle_to_monitoring() {
        let obs = Observation {
            motion_level: 50,
            object_detected: false,
            confidence: 0,
            cpu_load: 10,
            detector_healthy: true,
        };

        let (next_state, action, degraded_reason, degraded_cycles) =
            transition(SystemState::Idle, obs, None, 0);

        assert_eq!(next_state, SystemState::Monitoring);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn monitoring_to_recording() {
        let obs = Observation {
            motion_level: 60,
            object_detected: true,
            confidence: 90,
            cpu_load: 10,
            detector_healthy: true,
        };

        let (next_state, action) = transition(SystemState::Monitoring, obs);

        assert_eq!(next_state, SystemState::Recording);
        assert_eq!(action, Action::StartRecording);
    }

    #[test]
    fn recording_to_alerting() {
        let obs = Observation {
            motion_level: 60,
            object_detected: true,
            confidence: 90,
            cpu_load: 10,
            detector_healthy: true,
        };

        let (next_state, action) = transition(SystemState::Recording, obs);

        assert_eq!(next_state, SystemState::Alerting);
        assert_eq!(action, Action::SendAlert);
    }

    #[test]
    fn degraded_on_high_cpu() {
        let obs = Observation {
            motion_level: 0,
            object_detected: false,
            confidence: 0,
            cpu_load: 90,
            detector_healthy: true,
        };

        let (next_state, action) = transition(SystemState::Monitoring, obs);

        assert_eq!(next_state, SystemState::Degraded);
        assert_eq!(action, Action::EnterDegradedMode);
    }

    #[test]
    fn degraded_on_detector_failure() {
        let obs = Observation {
            motion_level: 0,
            object_detected: false,
            confidence: 0,
            cpu_load: 10,
            detector_healthy: false,
        };

        let (next_state, action) = transition(SystemState::Recording, obs);

        assert_eq!(next_state, SystemState::Degraded);
        assert_eq!(action, Action::EnterDegradedMode);
    }
}
