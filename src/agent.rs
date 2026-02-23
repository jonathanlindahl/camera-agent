use crate::types::{Action, Observation, SystemState};

pub struct CameraAgent {
    state: SystemState,
    error_count: u32,
}

impl CameraAgent {
    pub fn new() -> Self {
        Self {
            state: SystemState::Idle,
            error_count: 0,
        }
    }

    pub fn current_state(&self) -> SystemState {
        self.state
    }

    pub fn step(&mut self, obs: Observation) -> Action {
        let (next_state, action) = transition(self.state, obs);
        self.state = next_state;
        action
    }
}

fn transition(state: SystemState, obs: Observation) -> (SystemState, Action) {
    if obs.cpu_load > 85 || !obs.detector_healthy {
        return (SystemState::Degraded, Action::EnterDegradedMode);
    }

    match state {
        SystemState::Idle => {
            if obs.motion_level > 30 {
                (SystemState::Monitoring, Action::None)
            } else {
                (SystemState::Idle, Action::None)
            }
        }
        SystemState::Monitoring => {
            if obs.motion_level > 50 && obs.object_detected {
                (SystemState::Recording, Action::StartRecording)
            } else if obs.motion_level <= 30 {
                (SystemState::Idle, Action::None)
            } else {
                (SystemState::Monitoring, Action::None)
            }
        }
        SystemState::Recording => {
            if obs.confidence > 80 {
                (SystemState::Alerting, Action::SendAlert)
            } else if obs.motion_level <= 30 {
                (SystemState::Monitoring, Action::StopRecording)
            } else {
                (SystemState::Recording, Action::None)
            }
        }
        SystemState::Alerting => {
            // Always return to Recording after one cycle
            (SystemState::Recording, Action::None)
        }
        SystemState::Degraded => {
            // Recovery handled later
            (SystemState::Degraded, Action::None)
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

        let (next_state, action) = transition(SystemState::Idle, obs);

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
}
