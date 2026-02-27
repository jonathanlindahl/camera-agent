use camera_agent::agent::CameraAgent;
use camera_agent::scheduler::Scheduler;
use camera_agent::types::Observation;

fn main() {
    let agent = CameraAgent::new();
    let mut scheduler = Scheduler::new(agent, 30);

    scheduler.run(|| Observation {
        motion_level: 0,
        object_detected: false,
        confidence: 0,
        cpu_load: 10,
        detector_healthy: true,
    });
}
