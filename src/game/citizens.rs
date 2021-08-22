use super::actions::Action;
use super::errors::IterationError;
use super::state::Context;
use super::jobs::Job;

use serde::{Serialize, Deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Citizens {
    pub idle: u64,
    pub farmers: u64,
    pub woodcutters: u64,
    pub miners: u64,
    pub max_population: u64
}

impl Citizens {
    pub fn count(&self) -> u64 {
        self.idle + self.farmers + self.woodcutters + self.miners
    }

    pub fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
        match action {
            Action::RecruitCitizen => if self.count() < self.max_population {
                Ok(Self { idle: self.idle + 1, ..self })
            } else {
                Err(IterationError::PopulationLimitReached)
            },
            Action::AssignJob(job) => if self.idle > 0 {
                match job {
                    Job::Farmer if self.idle > 0 => Ok(Self { idle: self.idle - 1, farmers: self.farmers + 1, ..self }),
                    Job::Woodcutter if self.idle > 0 => Ok(Self { idle: self.idle - 1, woodcutters: self.woodcutters + 1, ..self }),
                    Job::Miner if self.idle > 0 => Ok(Self { idle: self.idle - 1, miners: self.miners + 1, ..self }),
                    _ => Ok(self)
                }
            } else {
                Err(IterationError::NotEnaughtIdleWorkers)
            },
            Action::DischargeJob(job) => match job {
                Job::Farmer if self.farmers > 0 => Ok(Self { idle: self.idle + 1, farmers: self.farmers - 1, ..self }),
                Job::Woodcutter if self.woodcutters > 0 => Ok(Self { idle: self.idle + 1, woodcutters: self.woodcutters - 1, ..self }),
                Job::Miner if self.miners > 0 => Ok(Self { idle: self.idle + 1, miners: self.miners - 1, ..self }),
                job => Err(IterationError::NoWorkersInJob(job.clone()))
            },
            Action::Build(building) => Ok(Self { max_population: self.max_population + building.population_capacity_increase(), ..self }),
            _ => Ok(self)
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_u64(self.idle);
        hasher.write_u64(self.farmers);
        hasher.write_u64(self.woodcutters);
        hasher.write_u64(self.miners);
        hasher.write_u64(self.max_population);
        hasher.finish()
    }
}
