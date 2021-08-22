mod game;
use game::state::State;
use game::errors::{CheckError, IterationError};
use game::actions::Action;
use game::resources::PrimaryResource;
use game::buildings::Buildings;
use game::jobs::Job;

use clap::{App, Arg, SubCommand};
use directories::{ProjectDirs};
use std::path::{Path};
use std::fs::{create_dir_all, File, OpenOptions};

enum CliAction {
    Next { action: Action, times: usize, trust: bool },
    Check,
    Create
}

struct SaveFile(Box<Path>);

impl SaveFile {
    fn get_file(&self, read: bool, write: bool) -> File {
        OpenOptions::new()
            .write(write)
            .read(read)
            .append(false)
            .truncate(write)
            .open(self.0.clone())
            .expect("Could not open save file.")
    }

    fn read(&self) -> State {
        serde_json::from_reader(self.get_file(true, false)).expect("Could not read save file.")
    }

    fn write(&self, state: State) {
        serde_json::to_writer_pretty(self.get_file(false, true), &state).expect("Could write to save file.");
    }
}

fn main() {
    let matches = App::new("My Super Program")
        .subcommand(SubCommand::with_name("create")
            .about("Creates a new game."))
        .subcommand(SubCommand::with_name("check")
            .about("Check game save integrity."))
        .subcommand(SubCommand::with_name("next")
            .about("Advance in the game.")
            .arg(Arg::with_name("repeat")
                .short("r")
                .takes_value(true)
                .default_value("1")
                .help("Repeats the action n times."))
            .arg(Arg::with_name("trust")
                .short("t")
                .help("Do not check save integrity."))
            .subcommand(SubCommand::with_name("idle")
                .about("Iterates over the game for one turn without action."))
            .subcommand(SubCommand::with_name("collect")
                .about("Collect primary resources.")
                .subcommand(SubCommand::with_name("food")
                    .about("Collect food."))
                .subcommand(SubCommand::with_name("wood")
                    .about("Collect wood."))
                .subcommand(SubCommand::with_name("stone")
                    .about("Collect stone.")))
            .subcommand(SubCommand::with_name("build")
                .about("Transform resources into buildings.")
                .subcommand(SubCommand::with_name("tent")
                    .about("Build tent."))
                .subcommand(SubCommand::with_name("woodenhut")
                    .about("Build wooden hut."))
                .subcommand(SubCommand::with_name("barn")
                    .about("Build a barn."))
                .subcommand(SubCommand::with_name("woodstockpile")
                    .about("Build wood stockpile."))
                .subcommand(SubCommand::with_name("stonestockpile")
                    .about("Build stone stockpile.")))
            .subcommand(SubCommand::with_name("jobs")
                .about("Manages jobs.")
                .subcommand(SubCommand::with_name("assign")
                    .about("Assigns job to idle citizen.")
                    .subcommand(SubCommand::with_name("farmer")
                        .about("Assign farmer job to idle citizen."))
                    .subcommand(SubCommand::with_name("woodcutter")
                        .about("Assign woodcutter job to idle citizen."))
                    .subcommand(SubCommand::with_name("miner")
                        .about("Assign miner job to idle citizen.")))
                .subcommand(SubCommand::with_name("discharge")
                    .about("Discharges citizen from job.")
                    .subcommand(SubCommand::with_name("farmer")
                        .about("Discharges citizen from farmer job."))
                    .subcommand(SubCommand::with_name("woodcutter")
                        .about("Discharges citizen from woodcutter job."))
                    .subcommand(SubCommand::with_name("miner")
                        .about("Discharges citizen from miner job."))))
            .subcommand(SubCommand::with_name("recruit")
                .about("Recruits citizen.")))
        .get_matches();

    let cli_action = match matches.subcommand() {
        ("create", _) => CliAction::Create,
        ("check", _) => CliAction::Check,
        ("next", Some(sub)) => {
            let times: usize = sub.value_of("repeat").unwrap().parse().expect("Invalid value for 'repeat' option.");
            let trust: bool = sub.is_present("trust");

            let action = match sub.subcommand() {
                ("idle", _) => Action::Idle,
                ("collect", Some(sub)) => match sub.subcommand() {
                    ("food", _) => Action::Collect(PrimaryResource::Food),
                    ("wood", _) => Action::Collect(PrimaryResource::Wood),
                    ("stone", _) => Action::Collect(PrimaryResource::Stone),
                    (_, _) => unreachable!()
                },
                ("build", Some(sub)) => match sub.subcommand() {
                    ("tent", _) => Action::Build(Buildings::Tent),
                    ("woodenhut", _) => Action::Build(Buildings::WoodenHut),
                    ("barn", _) => Action::Build(Buildings::Barn),
                    ("woodstockpile", _) => Action::Build(Buildings::WoodStockpile),
                    ("stonestockpile", _) => Action::Build(Buildings::StoneStockpile),
                    (_, _) => unreachable!()
                },
                ("jobs", Some(sub)) => match sub.subcommand() {
                    ("assign", Some(sub)) => match sub.subcommand() {
                        ("farmer", _) => Action::AssignJob(Job::Farmer),
                        ("woodcutter", _) => Action::AssignJob(Job::Woodcutter),
                        ("miner", _) => Action::AssignJob(Job::Miner),
                        (_, _) => unreachable!()
                    },
                    ("discharge", Some(sub)) => match sub.subcommand() {
                        ("farmer", _) => Action::DischargeJob(Job::Farmer),
                        ("woodcutter", _) => Action::DischargeJob(Job::Woodcutter),
                        ("miner", _) => Action::DischargeJob(Job::Miner),
                        (_, _) => unreachable!()
                    },
                    (_, _) => unreachable!()
                },
                ("recruit", _) => Action::RecruitCitizen,
                (_, _) => unreachable!()
            };

            CliAction::Next { action, times, trust }
        },
        (_, _) => unreachable!()
    };

    let project_dir = ProjectDirs::from("com", "tomasgonzalezvivo", "cliciv").expect("Could not construct project dir path.");
    let project_path = project_dir.project_path();
    let save_path = project_path.join(Path::new("cliciv-save.json")).into_boxed_path();
    let save = SaveFile(save_path);

    if matches!(cli_action, CliAction::Create) {
        create_dir_all(project_path).expect("Could not create project path.");
        let new_state = State::rand();
        save.write(new_state);
    } else {
        let prev_state: State = save.read();

        let maybe_new_state: Option<State> = match cli_action {
            CliAction::Check => {
                match prev_state.check() {
                    Ok(()) => println!("Save file is ok."),
                    Err(error) => println!("Save file is corrupted: {}.", error.get_message())
                }

                None
            },
            CliAction::Next { action, times, trust } => {
                if !trust {
                    prev_state.check().expect("Save file is corrupted.");
                }

                match prev_state.repeat(times, action) {
                    Ok(new_state) => {
                        println!("{}", new_state);
                        Some(new_state)
                    },
                    Err(error) => {
                        println!("Failed to apply action: {}", error.get_message());
                        None
                    }
                }
            },

            _ => None
        };

        if let Some(new_state) = maybe_new_state {
            save.write(new_state);
        }
    }
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
