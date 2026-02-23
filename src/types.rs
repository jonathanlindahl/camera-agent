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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradedReason {
    HighCpu,
    DetectorFailure,
}

#[derive(Debug, Clone, Copy)]
pub struct Observation {
    pub motion_level: u8, // 0-255
    pub object_detected: bool,
    pub confidence: u8, // 0-100
    pub cpu_load: u8,   // 0-100
    pub detector_healthy: bool,
}
