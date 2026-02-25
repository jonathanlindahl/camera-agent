use crate::agent::CameraAgent;
use crate::types::Observation;
use std::time::{Duration, Instant};

pub struct Scheduler {
    agent: CameraAgent,
    frame_budget: Duration,
    deadline_misses: u64,
    worst_case: Duration,
    total_frames: u64,
}

impl Scheduler {
    pub fn new(agent: CameraAgent, fps: u32) -> Self {
        let frame_budget = Duration::from_millis(1000 / fps as u64);

        Self {
            agent,
            frame_budget,
            deadline_misses: 0,
            worst_case: Duration::ZERO,
            total_frames: 0,
        }
    }

    pub fn tick(&mut self, observation: Observation) {
        let start = Instant::now();

        self.agent.step(observation);

        let elapsed = start.elapsed();

        if elapsed > self.frame_budget {
            self.deadline_misses += 1;
        }

        if elapsed > self.worst_case {
            self.worst_case = elapsed;
        }

        self.total_frames += 1;
    }

    pub fn run<F>(&mut self, mut observation_source: F)
    where
        F: FnMut() -> Observation,
    {
        loop {
            let cycle_start = Instant::now();

            let obs = observation_source();
            self.tick(obs);

            let elapsed = cycle_start.elapsed();

            if elapsed < self.frame_budget {
                std::thread::sleep(self.frame_budget - elapsed);
            }
        }
    }
}
