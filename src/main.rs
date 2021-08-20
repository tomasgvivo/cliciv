use serde::{Serialize, Deserialize};
use serde_json::{to_string, to_string_pretty, from_reader, from_str};
use rand::{thread_rng, Rng, prelude::SeedableRng};
use rand::distributions::{Distribution, Bernoulli};
use rand_chacha::{ChaChaRng};
use std::io::{Stdin, Read};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use byteorder::{LittleEndian, WriteBytesExt};
use backtrace::Backtrace;

trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl AsBytes for f64 {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bs = [0u8; std::mem::size_of::<Self>()];
        bs.as_mut()
            .write_f64::<LittleEndian>(*self)
            .expect("Unable to write");

        bs.to_vec()
    }
}

trait RoundTo2 {
    fn round_to_2(&self) -> Self;
}

impl RoundTo2 for f64 {
    fn round_to_2(&self) -> Self {
        (self * 100.0).round() / 100.0
    }
}

#[derive(Debug)]
enum Error {
    InputParseError(serde_json::Error),
    ActionParseError(serde_json::Error),
    Check(CheckError),
    Iteration(IterationError)
}

#[derive(Debug)]
enum IterationError {
    NotEnaughtResource(Resource),
    NotEnaughtFreeLand,
    NotEnaughtIdleWorkers,
    NotEnaughtWorkersInJob(Job),
    PopulationLimitReached,
}

#[derive(Debug)]
enum CheckError {
    HashMismatch,
    InvalidStateRecreation(IterationError)
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
enum Buildings {
    Tent,
    WoodenHut,
    Barn,
    WoodStockpile,
    StoneStockpile
}

impl Buildings {
    fn costs(&self) -> Vec<(Resource, f64)> {
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

    fn population_capacity_increase(&self) -> u64 {
        match self {
            Self::Tent => 1,
            Self::WoodenHut => 3,
            _ => 0
        }
    }

    fn primary_resource_storage_increase(&self) -> Option<(PrimaryResource, f64)> {
        match self {
            Self::Barn => Some((PrimaryResource::Food, 100.0)),
            Self::WoodStockpile => Some((PrimaryResource::Wood, 100.0)),
            Self::StoneStockpile => Some((PrimaryResource::Stone, 100.0)),
            _ => None
        }
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
enum Job {
    Farmer,
    Woodcutter,
    Miner,
}

impl Job {
    fn get_production_rate(&self) -> f64 {
        match self {
            Job::Farmer => 1.2,
            Job::Woodcutter => 0.5,
            Job::Miner => 0.2
        }
    }

    fn get_resource_production(&self) -> Resource {
        match self {
            Self::Farmer => Resource::Primary(PrimaryResource::Food),
            Self::Woodcutter => Resource::Primary(PrimaryResource::Wood),
            Self::Miner => Resource::Primary(PrimaryResource::Stone)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
enum PrimaryResource {
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
enum SecondaryResource {
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
enum TertiaryResource {
    Leather,
    Piety,
    Metal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum SpecialResource {
    Gold,
    Corpses
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Resource {
    Primary(PrimaryResource),
    Secondary(SecondaryResource),
    Tertiary(TertiaryResource),
    Special(SpecialResource),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
enum Action {
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

#[derive(Serialize, Deserialize, Clone)]
struct Citizens {
    idle: u64,
    farmers: u64,
    woodcutters: u64,
    miners: u64,
    max_population: u64
}

impl Citizens {
    fn count(&self) -> u64 {
        self.idle + self.farmers + self.woodcutters + self.miners
    }

    fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
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
                job => Err(IterationError::NotEnaughtWorkersInJob(job.clone()))
            },
            Action::Build(building) => Ok(Self { max_population: self.max_population + building.population_capacity_increase(), ..self }),
            _ => Ok(self)
        }
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_u64(self.idle);
        hasher.write_u64(self.farmers);
        hasher.write_u64(self.woodcutters);
        hasher.write_u64(self.miners);
        hasher.write_u64(self.max_population);
        hasher.finish()
    }
}

impl Default for Citizens {
    fn default() -> Self {
        Self {
            idle: 0,
            farmers: 0,
            woodcutters: 0,
            miners: 0,
            max_population: 0
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Land {
    total_land: u64,
    tents: u64,
    wooden_huts: u64,
    barns: u64,
    wood_stockpiles: u64,
    stone_stockpiles: u64
}

impl Land {
    fn free_land(&self) -> u64 {
        self.total_land - self.land_use()
    }

    fn land_use(&self) -> u64 {
        self.tents +
        self.wooden_huts +
        self.barns +
        self.wood_stockpiles +
        self.stone_stockpiles
    }

    fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
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

    fn hash(&self) -> u64 {
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

#[derive(Serialize, Deserialize, Clone)]
struct Resources {
    // Main
    food: f64,
    food_cons_rate: f64,
    food_prod_rate: f64,
    food_prod_rate_multiplier: f64,
    max_food: f64,
    wood: f64,
    wood_prod_rate: f64,
    wood_prod_rate_multiplier: f64,
    max_wood: f64,
    stone: f64,
    stone_prod_rate: f64,
    stone_prod_rate_multiplier: f64,
    max_stone: f64,

    // Secondary
    skins: f64,
    herbs: f64,
    ore: f64,
    leather: f64,
    piety: f64,
    metal: f64,

    // Special
    gold: f64,
    corpses: f64
}

impl Resources {

    fn increase(self, resource: Resource, amount: f64, ctx: &mut Context) -> Result<Self, IterationError> {
        //eprintln!("----------------------------> {:?} + {}", resource, amount);

        let state = match resource {
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
        };

        state
    }

    fn decrease(self, resource: Resource, amount: f64) -> Result<Self, IterationError> {
        //eprintln!("----------------------------> {:?} - {}", resource, amount);
        let state = match resource {
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
        };
        state
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

    fn apply_action(self, action: &Action, ctx: &mut Context) -> Result<Self, IterationError> {
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

    fn work(self, ctx: &mut Context) -> Result<Self, IterationError> {
        let food_inc = self.food_prod_rate * self.food_prod_rate_multiplier - self.food_cons_rate;
        let wood_inc = self.wood_prod_rate * self.wood_prod_rate_multiplier;
        let stone_inc = self.stone_prod_rate * self.stone_prod_rate_multiplier;

        self.increase(Resource::Primary(PrimaryResource::Food), food_inc, ctx)?
            .increase(Resource::Primary(PrimaryResource::Wood), wood_inc, ctx)?
            .increase(Resource::Primary(PrimaryResource::Stone), stone_inc, ctx)
    }

    fn hash(&self) -> u64 {
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

type LogEntry = (Action, u64);

#[derive(Serialize, Deserialize, Clone)]
struct State {
    seed: i128,
    prev_hash: u64,
    iterations: usize,
    resources: Resources,
    citizens: Citizens,
    land: Land,
    log: Vec<LogEntry>
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_limit = 5;

        writeln!(f, "")?;
        writeln!(f, "Seed\t\t{:032x}", self.seed)?;
        writeln!(f, "Previous Hash\t{:016x}", self.prev_hash)?;
        writeln!(f, "Resources:")?;
        writeln!(f, "\tPrimary:")?;
        writeln!(f, "\t\tFood\t\t{:.2}\t{:.2}/i\t(max {})",
            self.resources.food,
            self.resources.food_prod_rate * self.resources.food_prod_rate_multiplier - self.resources.food_cons_rate,
            self.resources.max_food
        )?;
        writeln!(f, "\t\tWood\t\t{:.2}\t{:.2}/i\t(max {})",
            self.resources.wood,
            self.resources.wood_prod_rate * self.resources.wood_prod_rate_multiplier,
            self.resources.max_wood
        )?;
        writeln!(f, "\t\tStone\t\t{:.2}\t{:.2}/i\t(max {})",
            self.resources.stone,
            self.resources.stone_prod_rate * self.resources.stone_prod_rate_multiplier,
            self.resources.max_stone
        )?;
        writeln!(f, "\tSecondary:")?;
        writeln!(f, "\t\tSkins\t\t{}", self.resources.skins)?;
        writeln!(f, "\t\tHerbs\t\t{}", self.resources.herbs)?;
        writeln!(f, "\t\tOre\t\t{}", self.resources.ore)?;
        writeln!(f, "\tTertiary:")?;
        writeln!(f, "\t\tLeather\t\t{}", self.resources.leather)?;
        writeln!(f, "\t\tPiety\t\t{}", self.resources.piety)?;
        writeln!(f, "\t\tMetal\t\t{}", self.resources.metal)?;
        writeln!(f, "\tSpetial:")?;
        writeln!(f, "\t\tGold\t\t{}", self.resources.gold)?;
        writeln!(f, "\t\tCorpses\t\t{}", self.resources.corpses)?;
        writeln!(f, "Citizens:")?;
        writeln!(f, "\tStats:")?;
        writeln!(f, "\t\tMax\t\t{}", self.citizens.max_population)?;
        writeln!(f, "\t\tIdle\t\t{}", self.citizens.idle)?;
        writeln!(f, "\t\tTotal\t\t{}", self.citizens.count())?;
        writeln!(f, "\tWorkers:")?;
        writeln!(f, "\t\tFarmers\t\t{}", self.citizens.farmers)?;
        writeln!(f, "\t\tWoodcutters\t{}", self.citizens.woodcutters)?;
        writeln!(f, "\t\tMiners\t\t{}", self.citizens.miners)?;
        writeln!(f, "Log:")?;
        if self.log.len() > log_limit + 1 {
            writeln!(f, "\t... {} more entries ...", self.log.len() - log_limit)?;
        } else if self.log.len() == log_limit + 1 {
            writeln!(f, "\t... {} more entry ...", self.log.len() - log_limit)?;
        }
        for log_entry in self.log.iter().rev().take(log_limit).rev() {
            writeln!(f, "\tx{}\t{:?}", log_entry.1, log_entry.0)?;
        }
        Ok(())
    }
}

impl State {
    fn new(seed: i128) -> Self {
        Self {
            seed,
            prev_hash: 0,
            iterations: 0,
            log: Default::default(),
            resources: Default::default(),
            citizens: Default::default(),
            land: Default::default()
        }
    }

    fn get_initial_state(&self) -> Self {
        Self {
            seed: self.seed,
            prev_hash: 0,
            iterations: 0,
            log: Default::default(),
            resources: Default::default(),
            citizens: Default::default(),
            land: Default::default()
        }
    }

    /**
     * Check if the current state is valid by rebuilding it step by step.
     */
    fn check(&self) -> Result<(), CheckError> {
        let state = Self::new(self.seed).apply_log(self.log.clone())
            .or_else(|error| Err(CheckError::InvalidStateRecreation(error)))?;

        if self.hash() == state.hash() {
            Ok(())
        } else {
            Err(CheckError::HashMismatch)
        }
    }

    /**
     * Get apply_action state from current state and an optional action.
     */
    fn apply_action(self, action: Action) -> Result<Self, IterationError> {
        let mut ctx = self.get_context();
        let prev_hash = self.hash();
        let seed = self.seed;
        let iterations = self.iterations + 1;
        let resources = self.resources.work(&mut ctx)?.apply_action(&action, &mut ctx)?;
        let citizens = self.citizens.apply_action(&action, &mut ctx)?;
        let land = self.land.apply_action(&action, &mut ctx)?;

        let log = {
            let mut log: Vec<LogEntry> = vec!{};

            for log_entry in [self.log, vec!{(action, 1)}].concat() {
                if log.len() > 0 {
                    let last_index = log.len() - 1;
    
                    if log[last_index].0 == log_entry.0 {
                        log[last_index].1 += 1;
                        continue;
                    }
                }

                log.push(log_entry);
            }

            log
        };

        Ok(Self {
            prev_hash,
            seed,
            iterations,
            resources,
            citizens,
            land,
            log
        })
    }

    fn apply_log(self, log: Vec<LogEntry>) -> Result<Self, IterationError> {
        let hash = self.hash();
        let mut state = self;

        for log_entry in log {
            for _ in 0..log_entry.1 {
                state = state.apply_action(log_entry.0.clone())?;
            }
        }

        Ok(state)
    }

    fn apply_and_check_log(self, log: Vec<LogEntry>) -> Result<Self, CheckError> {
        let mut state = self;

        for log_entry in log {
            for _ in 0..log_entry.1 {
                state.check()?;

                state = state.apply_action(log_entry.0.clone())
                    .or_else(|error| Err(CheckError::InvalidStateRecreation(error)))?;
            }
        }

        Ok(state)
    }

    /**
     * Get current state hash.
     */
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_i128(self.seed);
        hasher.write_u64(self.prev_hash);
        hasher.write_usize(self.iterations);
        hasher.write_u64(self.resources.hash());
        hasher.write_u64(self.citizens.hash());
        hasher.write_u64(self.land.hash());
        hasher.finish()
    }

    /**
     * Creates a new state with a random seed.
     */
    fn rand() -> Self {
        let mut rng = thread_rng();
        Self::new(rng.gen())
    }

    fn get_rng(&self) -> ChaChaRng {
        let seed: [u8; 32] = [
            // State seed
            ((self.seed >> (8 * 0)) & 0b11111111) as u8,
            ((self.seed >> (8 * 1)) & 0b11111111) as u8,
            ((self.seed >> (8 * 2)) & 0b11111111) as u8,
            ((self.seed >> (8 * 3)) & 0b11111111) as u8,
            ((self.seed >> (8 * 4)) & 0b11111111) as u8,
            ((self.seed >> (8 * 5)) & 0b11111111) as u8,
            ((self.seed >> (8 * 6)) & 0b11111111) as u8,
            ((self.seed >> (8 * 7)) & 0b11111111) as u8,
            ((self.seed >> (8 * 8)) & 0b11111111) as u8,
            ((self.seed >> (8 * 9)) & 0b11111111) as u8,
            ((self.seed >> (8 * 10)) & 0b11111111) as u8,
            ((self.seed >> (8 * 11)) & 0b11111111) as u8,
            ((self.seed >> (8 * 12)) & 0b11111111) as u8,
            ((self.seed >> (8 * 13)) & 0b11111111) as u8,
            ((self.seed >> (8 * 14)) & 0b11111111) as u8,
            ((self.seed >> (8 * 15)) & 0b11111111) as u8,

            // State previous hash
            ((self.prev_hash >> (8 * 0)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 1)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 2)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 3)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 4)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 5)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 6)) & 0b11111111) as u8,
            ((self.prev_hash >> (8 * 7)) & 0b11111111) as u8,

            // Padding
            0, 0, 0, 0, 0, 0, 0, 0
        ];

        return ChaChaRng::from_seed(seed);
    }

    fn get_context(&self) -> Context {
        Context {
            rng: self.get_rng()
        }
    }
}

struct Context {
    rng: ChaChaRng
}

fn main() {
    let state = match atty::isnt(atty::Stream::Stdin) {
        true => match from_reader::<Stdin, State>(std::io::stdin()) {
            Ok(previous_state) => {
                let original_state = previous_state.clone();
                let check_result: Result<(), CheckError> = previous_state.check();
                if check_result.is_err() {
                    eprintln!("{:?}", check_result.unwrap_err());
                    original_state
                } else {
                    let args: Vec<String> = std::env::args().collect();
                    match from_str(&args[1][..]) {
                        Ok(action) => match previous_state.apply_action(action) {
                            Ok(state) => state,
                            Err(error) => {
                                eprintln!("{:?}", error);
                                original_state
                            }
                        },
                        Err(error) => {
                            eprintln!("{:?}", error);
                            original_state
                        }
                    }
                }
            },
            Err(error) => {
                eprintln!("{:?}", error);
                panic!();
            }
        },

        false => State::rand()
    };

    println!("{}", to_string_pretty(&state).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use rand::{Rng, RngCore};
    use rand_chacha::ChaChaRng;
    use rand::distributions::{ Distribution, Uniform, Bernoulli };

    #[test]
    fn it_works() {
        let state = State::new(43932030939219715774207308070970463251);
        assert_eq!(state.prev_hash, 0);
        assert_eq!(state.hash(), 16377919232096456363);
    }

    #[test]
    fn farmer() {
        let state = State::new(43932030939219715774207308070970463251)
            .apply_log(vec!{
                (Action::Collect(PrimaryResource::Food), 200),
                (Action::Collect(PrimaryResource::Wood), 200),
                (Action::Collect(PrimaryResource::Stone), 200),
                (Action::Build(Buildings::WoodenHut), 1),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::Idle, 100),
            }).unwrap();

        println!("{}", state);

        assert_eq!(true, state.check().is_ok());
    }

    #[test]
    fn food_production() {
        let state = State::new(-141872649309347578469772012024767025949)
            .apply_log(vec!{
                (Action::Collect(PrimaryResource::Food), 200),
                (Action::Collect(PrimaryResource::Wood), 200),
                (Action::Build(Buildings::WoodenHut), 3),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::RecruitCitizen, 1),
                (Action::AssignJob(Job::Farmer), 1),
                (Action::Idle, 101),
            }).unwrap();

        println!("{}", state);

        assert_eq!(state.resources.food, 200.0);
        assert_eq!(true, state.check().is_ok());
    }

    #[test]
    fn test() {
        println!("{}", to_string(&Action::Collect(PrimaryResource::Food)).unwrap());
    }

    #[test]
    fn test2() {
        let state = State::rand();
        let mut state_rng = state.get_rng();

        let dist = Bernoulli::from_ratio(1, 10).unwrap();
        let v = dist.sample(&mut state_rng);
        println!("{}", v);
    }
}

// export STATE=$(echo $STATE | ./target/debug/CliCommander) && echo $STATE