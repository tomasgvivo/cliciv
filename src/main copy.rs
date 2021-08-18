use serde::{Serialize, Deserialize};
use serde_json::{to_string, to_string_pretty, from_reader, Error};
use rand::{thread_rng, Rng};
use std::io::{Stdin, Read};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::collections::BTreeMap;

enum Job {
    Helper
}

#[derive(Serialize, Deserialize, Clone)]
enum Action {
    // Add 1 worker to state workers_idle, cost 10 units.
    AdquireWorker,
    AssignHelperJob
}

#[derive(Serialize, Deserialize)]
enum Notice {
    New
}

/**
 * Los items son elementos que uno consigue jugando.
 * Se guardan en el inventario del state.
 * El usuario puede consumirlos cuando quiera.
 */
#[derive(Serialize, Deserialize)]
enum Item {
    
}

/**
 * Los buffs son efectos que afectan al resultado de una o varias iteraciones del juego.
 */
#[derive(Serialize, Deserialize)]
enum Buff {
    // Duplica la cantidad de unidades producidas en la próxima iteración.
    TimesTwo
}

#[derive(Serialize, Deserialize)]
struct Workers {
    idle: u64,
    helpers: u64,
}

impl Workers {

    can_assign_job(&self, job: Job) {
        match (job, self) => {
            
        }
    }

    assign_job(self, ) {
        if self.can
    }
}

impl Default for Workers {
    fn default() -> Self {
        Self {
            idle: 0,
            helpers: 0
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Resources {
    units: u128,
    workers: Workers,
    inventary: Vec<Item>
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            units: 0,
            workers: Default::default(),
            inventary: Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    seed: i128,
    prev_hash: u64,
    iterations: u128,
    notices: Vec<Notice>,
    log: BTreeMap<u128, Action>,
    resources: Resources
}

impl State {
    fn new(seed: i128) -> Self {
        Self {
            seed,
            prev_hash: 0,
            iterations: 0,
            notices: vec!{ Notice::New },
            log: Default::default(),
            resources: Default::default()
        }
    }

    /**
     * Check if the current state is valid by rebuilding it step by step.
     */
    fn check(&self) {
        let mut state = Self::new(self.seed);

        while state.iterations < self.iterations {
            let i = state.iterations;
            state = state.next(self.log.get(&(i + 1)));
        }

        if self.hash() != state.hash() {
            panic!("Check failed, hash does not match after {} iterations.", self.iterations);
        }
    }

    /**
     * Get next state from current state and an optional action.
     */
    fn next(self, maybe_action: Option<&Action>) -> Self {
        let prev_hash = self.hash();
        let iterations = self.iterations + 1;

        let mut log = self.log;
        let mut units = self.units;
        let mut workers_idle = self.workers_idle;
        let mut workers_helpers = self.workers_helpers;

        if let Some(action) = maybe_action {
            let commit = match action {
                Action::AdquireWorker => {
                    if units >= 10 {
                        units -= 10;
                        workers_idle += 1;
                        true
                    } else {
                        false
                    }
                },

                Action::AssignHelperJob => {
                    if workers_idle > 0 {
                        workers_idle -= 1;
                        workers_helpers += 1;
                        true
                    } else {
                        false
                    }
                }
            };

            if commit {
                log.insert(iterations, action.clone());
            }
        } else {
            units += 1;
        }

        units += 1 * workers_helpers as u128;

        let new_state = Self {
            prev_hash,
            seed: self.seed,
            units,
            iterations,
            notices: vec!{},
            log,
            workers_idle,
            workers_helpers
        };

        eprintln!("{}", to_string_pretty(&new_state).unwrap());

        new_state
    }

    fn forward(self, iterations: u128) -> Self {
        let mut state = self;
        for _ in 0..iterations {
            state = state.next(None);
        }
        state
    }

    /**
     * Get current state hash.
     */
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_i128(self.seed);
        hasher.write_u128(self.units);
        hasher.write_u128(self.iterations);
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

fn main() {
    let state = match atty::isnt(atty::Stream::Stdin) {
        true => match from_reader::<Stdin, State>(std::io::stdin()) {
            Ok(previous_state) => {
                previous_state.check();
                previous_state.next(None)
            },
            Err(error) => {
                eprintln!("Error while reading input! {}", error);
                State::rand()
            }
        },

        false => State::rand()
    };

    println!("{}", to_string(&state).unwrap());
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
    fn action_should_not_increment_units() {
        let state = State::new(43932030939219715774207308070970463251);
        assert_eq!(state.units, 1);
        assert_eq!(state.next(Some(&Action::AdquireWorker)).units, 1);
    }

    #[test]
    fn adquire_worker_action_should_incremente_workers_idle_by_one() {
        let mut state = State::new(43932030939219715774207308070970463251).forward(9);
        assert_eq!(state.units, 10);
        assert_eq!(state.workers_idle, 0);
        state = state.next(Some(&Action::AdquireWorker));
        assert_eq!(state.units, 0);
        assert_eq!(state.workers_idle, 1);
        state.check();
    }

    #[test]
    fn helpers_should_help() {
        let mut state = State::new(43932030939219715774207308070970463251)
            .forward(9)
            .next(Some(&Action::AdquireWorker))
            .next(Some(&Action::AssignHelperJob));
        assert_eq!(state.units, 0);
        assert_eq!(state.workers_idle, 0);
        assert_eq!(state.workers_helpers, 1);
        state = state.next(None);
        assert_eq!(state.units, 2);
        state.check();
    }
}

// export STATE=$(echo $STATE | ./target/debug/CliCommander) && echo $STATE