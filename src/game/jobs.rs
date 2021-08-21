use super::resources::{Resource, PrimaryResource};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Job {
    Farmer,
    Woodcutter,
    Miner,
}

impl Job {
    pub fn get_production_rate(&self) -> f64 {
        match self {
            Job::Farmer => 1.2,
            Job::Woodcutter => 0.5,
            Job::Miner => 0.2
        }
    }

    pub fn get_resource_production(&self) -> Resource {
        match self {
            Self::Farmer => Resource::Primary(PrimaryResource::Food),
            Self::Woodcutter => Resource::Primary(PrimaryResource::Wood),
            Self::Miner => Resource::Primary(PrimaryResource::Stone)
        }
    }
}