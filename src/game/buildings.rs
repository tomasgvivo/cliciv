use super::resources::{Resource, PrimaryResource, SecondaryResource};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Buildings {
    Tent,
    WoodenHut,
    Barn,
    WoodStockpile,
    StoneStockpile
}

impl Buildings {
    pub fn costs(&self) -> Vec<(Resource, f64)> {
        match self {
            Self::Tent => vec!{
                (Resource::Primary(PrimaryResource::Wood), 2.0),
                (Resource::Secondary(SecondaryResource::Skins), 2.0)
            },

            Self::WoodenHut => vec!{
                (Resource::Primary(PrimaryResource::Wood), 20.0),
                (Resource::Secondary(SecondaryResource::Skins), 1.0)
            },

            Self::Barn => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100.0)
            },

            Self::WoodStockpile => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100.0)
            },

            Self::StoneStockpile => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100.0)
            }
        }
    }

    pub fn population_capacity_increase(&self) -> u64 {
        match self {
            Self::Tent => 1,
            Self::WoodenHut => 3,
            _ => 0
        }
    }

    pub fn primary_resource_storage_increase(&self) -> Option<(PrimaryResource, f64)> {
        match self {
            Self::Barn => Some((PrimaryResource::Food, 100.0)),
            Self::WoodStockpile => Some((PrimaryResource::Wood, 100.0)),
            Self::StoneStockpile => Some((PrimaryResource::Stone, 100.0)),
            _ => None
        }
    }
}
