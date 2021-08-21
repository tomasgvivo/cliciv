mod game;
use game::state::State;
use game::errors::{CheckError};

use std::io::Stdin;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, from_str, to_string_pretty};

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
    use super::game::state::State;
    use super::game::actions::Action;
    use super::game::resources::*;
    use super::game::buildings::*;
    use super::game::jobs::*;

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
}
