use super::resources::Resource;
use super::jobs::Job;

#[derive(Debug)]
pub enum Error {
    InputParseError(serde_json::Error),
    ActionParseError(serde_json::Error),
    Check(CheckError),
    Iteration(IterationError)
}

#[derive(Debug)]
pub enum IterationError {
    NotEnaughtResource(Resource),
    NotEnaughtFreeLand,
    NotEnaughtIdleWorkers,
    NoWorkersInJob(Job),
    PopulationLimitReached,
}

impl IterationError {
    pub fn get_message(&self) -> String {
        match self {
            IterationError::NotEnaughtFreeLand => "not enauth free land".to_owned(),
            IterationError::NotEnaughtIdleWorkers => "not enauth idle workers".to_owned(),
            IterationError::NotEnaughtResource(_) => "not enauth resources".to_owned(),
            IterationError::NoWorkersInJob(_) => "no workers in job".to_owned(),
            IterationError::PopulationLimitReached => "population limit reached".to_owned()
        }
    }
}

#[derive(Debug)]
pub enum CheckError {
    HashMismatch,
    InvalidStateRecreation(usize, IterationError)
}

impl CheckError {
    pub fn get_message(&self) -> String {
        match self {
            CheckError::HashMismatch => "Hash mismatch".to_owned(),
            CheckError::InvalidStateRecreation(iteration, error) => {
                format!("Invalid state recreation ({} at iteration {})", error.get_message(), iteration)
            }
        }
    }
}
