use super::resources::PrimaryResource;
use super::buildings::Buildings;
use super::jobs::Job;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Action {
    // Idle
    Idle,

    // Resource gathering:
    Collect(PrimaryResource),

    // Citizent management:
    RecruitCitizen,
    AssignJob(Job),
    DischargeJob(Job),

    // Land management:
    Build(Buildings),
}