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
    NotEnaughtWorkersInJob(Job),
    PopulationLimitReached,
}

#[derive(Debug)]
pub enum CheckError {
    HashMismatch,
    InvalidStateRecreation(IterationError)
}
