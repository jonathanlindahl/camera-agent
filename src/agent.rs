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

    fn obs(motion: u8, detected: bool, conf: u8, load: u8, healthy: bool) -> Observation {
        Observation {
            motion_level: motion,
            object_detected: detected,
            confidence: conf,
            cpu_load: load,
            detector_healthy: healthy,
        }
    }

    #[test]
    fn basic_state_transitions() {
        let cases = vec![
            (
                SystemState::Idle,
                obs(40, false, 0, 10, true),
                SystemState::Monitoring,
                Action::None,
            ),
            (
                SystemState::Monitoring,
                obs(60, true, 0, 10, true),
                SystemState::Recording,
                Action::StartRecording,
            ),
            (
                SystemState::Recording,
                obs(60, true, 90, 10, true),
                SystemState::Alerting,
                Action::SendAlert,
            ),
        ];

        for (state, observation, expected_state, expected_action) in cases {
            let (next_state, action, _, _) = transition(state, observation, None, 0);

            assert_eq!(next_state, expected_state);
            assert_eq!(action, expected_action);
        }
    }

    #[test]
    fn full_recording_sequence() {
        let mut agent = CameraAgent::new();

        // Idle -> Monitoring
        agent.step(obs(40, false, 0, 10, true));
        assert_eq!(agent.current_state(), SystemState::Monitoring);

        // Monitoring -> Recording
        agent.step(obs(60, true, 0, 10, true));
        assert_eq!(agent.current_state(), SystemState::Recording);

        // Recording -> Alerting
        agent.step(obs(60, true, 90, 10, true));
        assert_eq!(agent.current_state(), SystemState::Alerting);
    }
}
