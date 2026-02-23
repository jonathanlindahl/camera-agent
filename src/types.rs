#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    Idle,
    Monitoring,
    Recording,
    Alerting,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    StartRecording,
    StopRecording,
    SendAlert,
    EnterDegradedMode,
    ExitDegradedMode,
}

#[derive(Debug, Clone, Copy)]
pub struct Observation {
    pub motion_level: u8,
    pub object_detected: bool,
    pub confidence: u8,
    pub cpu_load: u8,
    pub detector_healthy: bool,
}
