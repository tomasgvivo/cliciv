use super::buildings::Buildings;
use super::errors::IterationError;
use super::state::Context;
use super::actions::Action;

use serde::{Serialize, Deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Serialize, Deserialize, Clone)]
pub struct Land {
    pub total_land: u64,
    pub tents: u64,
    pub wooden_huts: u64,
    pub barns: u64,
    pub wood_stockpiles: u64,
    pub stone_stockpiles: u64
}

impl Land {
    pub fn free_land(&self) -> u64 {
        self.total_land - self.land_use()
    }

    pub fn land_use(&self) -> u64 {
        self.tents +
        self.wooden_huts +
        self.barns +
        self.wood_stockpiles +
        self.stone_stockpiles
    }

    pub fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
        match action {
            Action::Build(building) => if self.free_land() > 0 {
                match building {
                    Buildings::Tent => Ok(Self { tents: self.tents + 1, ..self }),
                    Buildings::WoodenHut => Ok(Self { wooden_huts: self.wooden_huts + 1, ..self }),
                    Buildings::Barn => Ok(Self { barns: self.barns + 1, ..self }),
                    Buildings::WoodStockpile => Ok(Self { wood_stockpiles: self.wood_stockpiles + 1, ..self }),
                    Buildings::StoneStockpile => Ok(Self { stone_stockpiles: self.stone_stockpiles + 1, ..self }),
                }
            } else {
                Err(IterationError::NotEnaughtFreeLand)
            },
            _ => Ok(self)
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_u64(self.total_land);
        hasher.write_u64(self.tents);
        hasher.write_u64(self.wooden_huts);
        hasher.write_u64(self.barns);
        hasher.write_u64(self.wood_stockpiles);
        hasher.write_u64(self.stone_stockpiles);
        hasher.finish()
    }
}

impl Default for Land {
    fn default() -> Self {
        Self {
            total_land: 1000,
            tents: 0,
            wooden_huts: 0,
            barns: 0,
            wood_stockpiles: 0,
            stone_stockpiles: 0
        }
    }
}
