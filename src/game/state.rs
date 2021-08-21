use super::resources::{Resource, Resources, PrimaryResource};
use super::errors::{CheckError, Error, IterationError};
use super::actions::Action;
use super::land::Land;
use super::citizens::Citizens;

use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use rand::{thread_rng, Rng, prelude::SeedableRng};
use rand_chacha::{ChaChaRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

type LogEntry = (Action, u64);

// TODO: Hacer que las propedades sean privaadas.
pub struct Context {
    pub rng: ChaChaRng
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    seed: i128,
    prev_hash: u64,
    iterations: usize,
    pub resources: Resources,
    citizens: Citizens,
    land: Land,
    log: Vec<LogEntry>
}

impl State {
    pub fn new(seed: i128) -> Self {
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
    pub fn check(&self) -> Result<(), CheckError> {
        let state = Self::get_initial_state(&self).apply_log(self.log.clone())
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
    pub fn apply_action(self, action: Action) -> Result<Self, IterationError> {
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

    pub fn apply_log(self, log: Vec<LogEntry>) -> Result<Self, IterationError> {
        let hash = self.hash();
        let mut state = self;

        for log_entry in log {
            for _ in 0..log_entry.1 {
                state = state.apply_action(log_entry.0.clone())?;
            }
        }

        Ok(state)
    }

    pub fn apply_and_check_log(self, log: Vec<LogEntry>) -> Result<Self, CheckError> {
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
    pub fn hash(&self) -> u64 {
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
    pub fn rand() -> Self {
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

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
