use super::utils::{AsBytes, RoundTo2};
use super::errors::IterationError;
use super::actions::Action;
use super::state::Context;

use serde::{Serialize, Deserialize};
use rand::distributions::{Distribution, Bernoulli};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Resource {
    Primary(PrimaryResource),
    Secondary(SecondaryResource),
    Tertiary(TertiaryResource),
    Special(SpecialResource),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum PrimaryResource {
    Food,
    Wood,
    Stone,
}

impl PrimaryResource {
    fn get_secondary_resource(&self) -> SecondaryResource {
        match self {
            Self::Food => SecondaryResource::Skins,
            Self::Wood => SecondaryResource::Herbs,
            Self::Stone => SecondaryResource::Ore,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SecondaryResource {
    Skins,
    Herbs,
    Ore,
}

impl SecondaryResource {
    fn get_distribution(&self) -> Bernoulli {
        match self {
            Self::Skins => Bernoulli::from_ratio(1, 10).unwrap(),
            Self::Herbs => Bernoulli::from_ratio(1, 10).unwrap(),
            Self::Ore => Bernoulli::from_ratio(1, 10).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TertiaryResource {
    Leather,
    Piety,
    Metal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpecialResource {
    Gold,
    Corpses
}

// TODO: Hacer que las propedades no sean públicas.
#[derive(Serialize, Deserialize, Clone)]
pub struct Resources {
    // Main
    pub food: f64,
    pub food_cons_rate: f64,
    pub food_prod_rate: f64,
    pub food_prod_rate_multiplier: f64,
    pub max_food: f64,
    pub wood: f64,
    pub wood_prod_rate: f64,
    pub wood_prod_rate_multiplier: f64,
    pub max_wood: f64,
    pub stone: f64,
    pub stone_prod_rate: f64,
    pub stone_prod_rate_multiplier: f64,
    pub max_stone: f64,

    // Secondary
    pub skins: f64,
    pub herbs: f64,
    pub ore: f64,
    pub leather: f64,
    pub piety: f64,
    pub metal: f64,

    // Special
    pub gold: f64,
    pub corpses: f64
}

impl Resources {
    fn increase(self, resource: Resource, amount: f64, ctx: &mut Context) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => {
                let secondary_resource_amount = primary_resource.get_secondary_resource()
                    .get_distribution()
                    .sample_iter(&mut ctx.rng)
                    .take(amount as usize)
                    .map(u64::from)
                    .sum::<u64>() as f64;

                match primary_resource {
                    PrimaryResource::Food => Ok(Self {
                        food: f64::min(self.food + amount, self.max_food).round_to_2(),
                        skins: (self.skins + secondary_resource_amount).round_to_2(),
                        ..self
                    }),
                    PrimaryResource::Wood => Ok(Self {
                        wood: f64::min(self.wood + amount, self.max_wood).round_to_2(),
                        herbs: (self.herbs + secondary_resource_amount).round_to_2(),
                        ..self
                    }),
                    PrimaryResource::Stone => Ok(Self {
                        stone: f64::min(self.stone + amount, self.max_wood).round_to_2(),
                        ore: (self.ore + secondary_resource_amount).round_to_2(),
                        ..self
                    }),
                }
            },
            Resource::Secondary(secondary_resource) => match secondary_resource {
                SecondaryResource::Skins => Ok(Self { skins: (self.skins + amount).round_to_2(), ..self }),
                SecondaryResource::Herbs => Ok(Self { herbs: (self.herbs + amount.round_to_2()), ..self }),
                SecondaryResource::Ore => Ok(Self { ore: (self.ore + amount).round_to_2(), ..self }),
            },
            Resource::Tertiary(tertiary_resource) => match tertiary_resource {
                TertiaryResource::Leather => Ok(Self { leather: (self.leather + amount).round_to_2(), ..self }),
                TertiaryResource::Piety => Ok(Self { piety: (self.piety + amount).round_to_2(), ..self }),
                TertiaryResource::Metal => Ok(Self { metal: (self.metal + amount).round_to_2(), ..self }),
            },
            Resource::Special(special_resource) => match special_resource {
                SpecialResource::Gold => Ok(Self { gold: (self.gold + amount).round_to_2(), ..self }),
                SpecialResource::Corpses => Ok(Self { corpses: (self.corpses + amount).round_to_2(), ..self }),
            },
        }
    }

    fn decrease(self, resource: Resource, amount: f64) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => match primary_resource {
                PrimaryResource::Food => if self.food as i64 - amount as i64 >= 0 {
                    Ok(Self { food: (self.food - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Primary(PrimaryResource::Food)))
                },
                PrimaryResource::Wood => if self.wood as i64 - amount as i64 >= 0 {
                    Ok(Self { wood: (self.wood - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Primary(PrimaryResource::Wood)))
                },
                PrimaryResource::Stone => if self.stone as i64 - amount as i64 >= 0 {
                    Ok(Self { stone: (self.stone - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Primary(PrimaryResource::Stone)))
                },
            },
            Resource::Secondary(secondary_resource) => match secondary_resource {
                SecondaryResource::Skins => if self.skins as i64 - amount as i64 >= 0 {
                    Ok(Self { skins: (self.skins - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Secondary(SecondaryResource::Skins)))
                },
                SecondaryResource::Herbs => if self.herbs as i64 - amount as i64 >= 0 {
                    Ok(Self { herbs: (self.herbs - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Secondary(SecondaryResource::Herbs)))
                },
                SecondaryResource::Ore => if self.ore as i64 - amount as i64 >= 0 {
                    Ok(Self { ore: (self.ore - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Secondary(SecondaryResource::Ore)))
                },
            },
            Resource::Tertiary(tertiary_resource) => match tertiary_resource {
                TertiaryResource::Leather => if self.leather as i64 - amount as i64 >= 0 {
                    Ok(Self { leather: (self.leather - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Tertiary(TertiaryResource::Leather)))
                },
                TertiaryResource::Piety => if self.piety as i64 - amount as i64 >= 0 {
                    Ok(Self { piety: (self.piety - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Tertiary(TertiaryResource::Piety)))
                },
                TertiaryResource::Metal => if self.metal as i64 - amount as i64 >= 0 {
                    Ok(Self { metal: (self.metal - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Tertiary(TertiaryResource::Metal)))
                },
            },
            Resource::Special(special_resource) => match special_resource {
                SpecialResource::Gold => if self.gold as i64 - amount as i64 >= 0 {
                    Ok(Self { gold: (self.gold - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Special(SpecialResource::Gold)))
                },
                SpecialResource::Corpses => if self.corpses as i64 - amount as i64 >= 0 {
                    Ok(Self { corpses: (self.corpses - amount).round_to_2(), ..self })
                } else {
                    Err(IterationError::NotEnaughtResource(Resource::Special(SpecialResource::Corpses)))
                },
            },
        }
    }

    fn increase_primary_resource_storage(self, primary_resource: PrimaryResource, amount: f64) -> Result<Self, IterationError> {
        match primary_resource {
            PrimaryResource::Food => Ok(Self { max_food: (self.max_food + amount).round_to_2(), ..self }),
            PrimaryResource::Wood => Ok(Self { max_wood: (self.max_wood + amount).round_to_2(), ..self }),
            PrimaryResource::Stone => Ok(Self { max_stone: (self.max_stone + amount).round_to_2(), ..self })
        }
    }

    fn increase_resource_production_rate(self, resource: Resource, amount: f64) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => match primary_resource {
                PrimaryResource::Food => Ok(Self { food_prod_rate: (self.food_prod_rate + amount).round_to_2(), ..self }),
                PrimaryResource::Wood => Ok(Self { wood_prod_rate: (self.wood_prod_rate + amount).round_to_2(), ..self }),
                PrimaryResource::Stone => Ok(Self { stone_prod_rate: (self.stone_prod_rate + amount).round_to_2(), ..self })
            }
            _ => Ok(self)
        }
    }

    fn decrease_resource_production_rate(self, resource: Resource, amount: f64) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => match primary_resource {
                PrimaryResource::Food => Ok(Self { food_prod_rate: (self.food_prod_rate - amount).round_to_2(), ..self }),
                PrimaryResource::Wood => Ok(Self { wood_prod_rate: (self.wood_prod_rate - amount).round_to_2(), ..self }),
                PrimaryResource::Stone => Ok(Self { stone_prod_rate: (self.stone_prod_rate - amount).round_to_2(), ..self })
            }
            _ => Ok(self)
        }
    }

    fn increase_food_consumption(self, amount: f64) -> Result<Self, IterationError> {
        Ok(Self { food_cons_rate: (self.food_cons_rate + amount).round_to_2(), ..self })
    }

    pub fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
        match action {
            Action::RecruitCitizen => {
                self.decrease(Resource::Primary(PrimaryResource::Food), 20.0)?
                    .increase_food_consumption(1.0)
            }
            Action::Collect(primary_resource) => self.increase(Resource::Primary(primary_resource.clone()), 1.0, ctx),
            Action::Build(building) => {
                let mut resources = self;

                for cost in building.costs() {
                    resources = resources.decrease(cost.0, cost.1)?;
                }

                if let Some(primary_resource_storage_increase) = building.primary_resource_storage_increase() {
                    resources = resources.increase_primary_resource_storage(
                        primary_resource_storage_increase.0,
                        primary_resource_storage_increase.1
                    )?;
                }

                Ok(resources)
            },
            Action::AssignJob(job) => self.increase_resource_production_rate(job.get_resource_production(), job.get_production_rate()),
            Action::DischargeJob(job) => self.decrease_resource_production_rate(job.get_resource_production(), job.get_production_rate()),
            _ => Ok(self)
        }
    }

    pub fn work(self, ctx: &mut Context) -> Result<Self, IterationError> {
        let food_inc = self.food_prod_rate * self.food_prod_rate_multiplier - self.food_cons_rate;
        let wood_inc = self.wood_prod_rate * self.wood_prod_rate_multiplier;
        let stone_inc = self.stone_prod_rate * self.stone_prod_rate_multiplier;

        self.increase(Resource::Primary(PrimaryResource::Food), food_inc, ctx)?
            .increase(Resource::Primary(PrimaryResource::Wood), wood_inc, ctx)?
            .increase(Resource::Primary(PrimaryResource::Stone), stone_inc, ctx)
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write(&self.food.as_bytes()[..]);
        hasher.write(&self.food_cons_rate.as_bytes()[..]);
        hasher.write(&self.food_prod_rate.as_bytes()[..]);
        hasher.write(&self.food_prod_rate_multiplier.as_bytes()[..]);
        hasher.write(&self.max_food.as_bytes()[..]);
        hasher.write(&self.wood.as_bytes()[..]);
        hasher.write(&self.wood_prod_rate.as_bytes()[..]);
        hasher.write(&self.wood_prod_rate_multiplier.as_bytes()[..]);
        hasher.write(&self.max_wood.as_bytes()[..]);
        hasher.write(&self.stone.as_bytes()[..]);
        hasher.write(&self.stone_prod_rate.as_bytes()[..]);
        hasher.write(&self.stone_prod_rate_multiplier.as_bytes()[..]);
        hasher.write(&self.max_stone.as_bytes()[..]);
        hasher.write(&self.skins.as_bytes()[..]);
        hasher.write(&self.herbs.as_bytes()[..]);
        hasher.write(&self.ore.as_bytes()[..]);
        hasher.write(&self.leather.as_bytes()[..]);
        hasher.write(&self.piety.as_bytes()[..]);
        hasher.write(&self.metal.as_bytes()[..]);
        hasher.write(&self.gold.as_bytes()[..]);
        hasher.write(&self.corpses.as_bytes()[..]);
        hasher.finish()
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            // Main
            food: 0.0,
            food_cons_rate: 0.0,
            food_prod_rate: 0.0,
            food_prod_rate_multiplier: 1.0,
            max_food: 200.0,
            wood: 0.0,
            wood_prod_rate: 0.0,
            wood_prod_rate_multiplier: 1.0,
            max_wood: 200.0,
            stone: 0.0,
            stone_prod_rate: 0.0,
            stone_prod_rate_multiplier: 1.0,
            max_stone: 200.0,

            // Secondary
            skins: 0.0,
            herbs: 0.0,
            ore: 0.0,
            leather: 0.0,
            piety: 0.0,
            metal: 0.0,

            // Special
            gold: 0.0,
            corpses: 0.0
        }
    }
}
