use serde::{Serialize, Deserialize};
use serde_json::{to_string, to_vec, to_string_pretty, from_reader, from_str};
use rand::{thread_rng, Rng};
use std::io::{Stdin, Read};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::collections::BTreeMap;

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

#[derive(Serialize, Deserialize, Clone)]
enum Buildings {
    Tent,
    WoodenHut,
    Barn,
    WoodStockpile,
    StoneStockpile
}

impl Buildings {
    fn costs(&self) -> Vec<(Resource, u64)> {
        match self {
            Self::Tent => vec!{
                (Resource::Primary(PrimaryResource::Wood), 2),
                (Resource::Secondary(SecondaryResource::Leather), 2)
            },

            Self::WoodenHut => vec!{
                (Resource::Primary(PrimaryResource::Wood), 20),
                (Resource::Secondary(SecondaryResource::Leather), 1)
            },

            Self::Barn => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100)
            },

            Self::WoodStockpile => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100)
            },

            Self::StoneStockpile => vec!{
                (Resource::Primary(PrimaryResource::Wood), 100)
            }
        }
    }

    fn population_increase(self) -> u64 {
        match self {
            Self::Tent => 1,
            Self::WoodenHut => 3,
            _ => 0
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Job {
    Farmer,
    Woodcutter,
    Miner,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum PrimaryResource {
    Food,
    Wood,
    Stone,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum SecondaryResource {
    Skins,
    Herbs,
    Ore,
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
    Special(SpecialResource),
}

#[derive(Serialize, Deserialize, Clone)]
enum Action {
    // Resource gathering:
    Collect(PrimaryResource),

    // Citizent management:
    RecruitCitizen,
    AssignJob(Job),
    DischargeJob(Job),

    // Land management:
    Build(Buildings),
}

#[derive(Serialize, Deserialize)]
struct Citizens {
    idle: u64,
    farmers: u64,
    woodcutters: u64,
    miners: u64,
}

impl Citizens {
    fn count(&self) -> u64 {
        self.idle + self.farmers + self.woodcutters + self.miners
    }

    fn apply_action(self, action: &Action, max_population: u64) -> Result<Self, IterationError> {
        match action {
            Action::RecruitCitizen => if self.count() < max_population {
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
            _ => Ok(self)
        }
    }
}

impl Default for Citizens {
    fn default() -> Self {
        Self {
            idle: 0,
            farmers: 0,
            woodcutters: 0,
            miners: 0
        }
    }
}

#[derive(Serialize, Deserialize)]
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

    fn max_population(&self) -> u64 {
        self.tents * 1 +
        self.wooden_huts * 3
    }

    fn max_food(&self) -> u64 {
        200 + self.barns * 100
    }

    fn max_wood(&self) -> u64 {
        200 + self.wood_stockpiles * 100
    }

    fn max_stone(&self) -> u64 {
        200 + self.stone_stockpiles * 100
    }

    fn resource_limits(&self) -> Resources {
        Resources {
            food: self.max_food(),
            wood: self.max_wood(),
            stone: self.max_stone(),
            ..Default::default()
        }
    }

    fn apply_action(self, action: &Action) -> Result<Self, IterationError> {
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

#[derive(Serialize, Deserialize, Default)]
struct Resources {
    // Main
    food: u64,
    wood: u64,
    stone: u64,

    // Secondary
    skins: u64,
    herbs: u64,
    ore: u64,
    leather: u64,
    piety: u64,
    metal: u64,

    // Special
    gold: u64,
    corpses: u64
}

impl Resources {

    fn increase(self, resource: Resource, amount: u64, limits: &Resources) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => match primary_resource {
                PrimaryResource::Food => Ok(Self { food: self.food + amount, ..self }),
                PrimaryResource::Wood => Ok(Self { wood: self.wood + amount, ..self }),
                PrimaryResource::Stone => Ok(Self { stone: self.stone + amount, ..self }),
            },
            Resource::Secondary(secondary_resource) => match secondary_resource {
                SecondaryResource::Skins => Ok(Self { skins: self.skins + amount, ..self }),
                SecondaryResource::Herbs => Ok(Self { herbs: self.herbs + amount, ..self }),
                SecondaryResource::Ore => Ok(Self { ore: self.ore + amount, ..self }),
                SecondaryResource::Leather => Ok(Self { leather: self.leather + amount, ..self }),
                SecondaryResource::Piety => Ok(Self { piety: self.piety + amount, ..self }),
                SecondaryResource::Metal => Ok(Self { metal: self.metal + amount, ..self }),
            },
            Resource::Special(special_resource) => match special_resource {
                SpecialResource::Gold => Ok(Self { gold: self.gold + amount, ..self }),
                SpecialResource::Corpses => Ok(Self { corpses: self.corpses + amount, ..self }),
            },
        }
    }

    fn decrease(self, resource: Resource, amount: u64, limits: &Resources) -> Result<Self, IterationError> {
        match resource {
            Resource::Primary(primary_resource) => match primary_resource {
                PrimaryResource::Food => Ok(Self { food: self.food - amount, ..self }),
                PrimaryResource::Wood => Ok(Self { wood: self.wood - amount, ..self }),
                PrimaryResource::Stone => Ok(Self { stone: self.stone - amount, ..self }),
            },
            Resource::Secondary(secondary_resource) => match secondary_resource {
                SecondaryResource::Skins => Ok(Self { skins: self.skins - amount, ..self }),
                SecondaryResource::Herbs => Ok(Self { herbs: self.herbs - amount, ..self }),
                SecondaryResource::Ore => Ok(Self { ore: self.ore - amount, ..self }),
                SecondaryResource::Leather => Ok(Self { leather: self.leather - amount, ..self }),
                SecondaryResource::Piety => Ok(Self { piety: self.piety - amount, ..self }),
                SecondaryResource::Metal => Ok(Self { metal: self.metal - amount, ..self }),
            },
            Resource::Special(special_resource) => match special_resource {
                SpecialResource::Gold => Ok(Self { gold: self.gold - amount, ..self }),
                SpecialResource::Corpses => Ok(Self { corpses: self.corpses - amount, ..self }),
            },
        }
    }

    fn apply_action(self, action: &Action, limits: &Resources) -> Result<Self, IterationError> {
        match action {
            Action::RecruitCitizen => self.decrease(Resource::Primary(PrimaryResource::Food), 1, limits),
            Action::Collect(primary_resource) => match primary_resource {
                PrimaryResource::Food => self.increase(Resource::Primary(PrimaryResource::Food), 1, limits),
                PrimaryResource::Wood => self.increase(Resource::Primary(PrimaryResource::Wood), 1, limits),
                PrimaryResource::Stone => self.increase(Resource::Primary(PrimaryResource::Stone), 1, limits),
            },
            Action::Build(building) => {
                let mut resources = self;

                for cost in building.costs() {
                    resources = resources.decrease(cost.0, cost.1, &limits)?;
                }

                Ok(resources)
            },
            _ => Ok(self)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    seed: i128,
    prev_hash: u64,
    iterations: usize,
    resources: Resources,
    citizens: Citizens,
    land: Land,
    log: Vec<Action>
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

    /**
     * Check if the current state is valid by rebuilding it step by step.
     */
    fn check(&self) -> Result<(), CheckError> {
        let mut state = Self::new(self.seed);

        while state.iterations < self.iterations {
            let i = state.iterations;
            state = state.next(self.log[i].clone())
                .or_else(|error| Err(CheckError::InvalidStateRecreation(error)))?;
        }

        if self.hash() == state.hash() {
            Ok(())
        } else {
            Err(CheckError::HashMismatch)
        }
    }

    /**
     * Get next state from current state and an optional action.
     */
    fn next(self, action: Action) -> Result<Self, IterationError> {
        let prev_hash = self.hash();
        let seed = self.seed;
        let iterations = self.iterations + 1;
        let resources = self.resources.apply_action(&action, &self.land.resource_limits())?;
        let citizens = self.citizens.apply_action(&action, self.land.max_population())?;
        let land = self.land.apply_action(&action)?;

        let new_state = Self {
            prev_hash,
            seed,
            iterations,
            resources,
            citizens,
            land,
            log: [self.log, vec!{action}].concat()
        };

        Ok(new_state)
    }

    /**
     * Get current state hash.
     */
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_i128(self.seed);
        hasher.write_u64(self.prev_hash);
        hasher.write_usize(self.iterations);
        hasher.write(&to_vec(&self.resources).unwrap()[..]);
        hasher.write(&to_vec(&self.citizens).unwrap()[..]);
        hasher.write(&to_vec(&self.land).unwrap()[..]);
        hasher.write(&to_vec(&self.log).unwrap()[..]);
        hasher.finish()
    }

    /**
     * Creates a new state with a random seed.
     */
    fn rand() -> Self {
        let mut rng = thread_rng();
        Self::new(rng.gen())
    }
}

fn main() -> Result<(), Error> {
    let state = match atty::isnt(atty::Stream::Stdin) {
        true => match from_reader::<Stdin, State>(std::io::stdin()) {
            Ok(previous_state) => {
                previous_state.check()
                    .or_else(|error| Err(Error::Check(error)))?;

                let args: Vec<String> = std::env::args().collect();
                let action: Action = from_str(&args[1][..]).or_else(|error| Err(Error::ActionParseError(error)))?;

                previous_state.next(action)
                    .or_else(|error| Err(Error::Iteration(error)))?
            },
            Err(error) => Err(Error::InputParseError(error))?
        },

        false => State::rand()
    };

    println!("{}", to_string_pretty(&state).unwrap());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let state = State::new(43932030939219715774207308070970463251);
        assert_eq!(state.prev_hash, 0);
        assert_eq!(state.hash(), 16377919232096456363);
    }

    #[test]
    fn test() {
        println!("{}", to_string(&Action::Collect(PrimaryResource::Food)).unwrap());
    }
}

// export STATE=$(echo $STATE | ./target/debug/CliCommander) && echo $STATE