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
        // Global degraded rule
        if obs.cpu_load > 85 || !obs.detector_healthy {
            self.state = SystemState::Degraded;
            return Action::EnterDegradedMode;
        }

        match self.state {
            SystemState::Idle => {
                if obs.motion_level > 30 {
                    self.state = SystemState::Monitoring;
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}
