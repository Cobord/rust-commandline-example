use crossterm::event::KeyCode;
use rand::{distributions::Alphanumeric, prelude::*};
use std::fs;
use std::io;
use thiserror::Error;
use tui::widgets::ListState;

mod data_row;
use data_row::DataRow;

mod pet;
use pet::Pet;

mod generic_tui;
use generic_tui::*;

pub const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rx, _join_handle) = io_handler();

    let mut terminal = get_terminal()?;

    let menu_titles = Pet::menu_titles();
    let mut active_menu_item = MenuItem::Home;

    let mut pet_list_state = get_data_list_state();

    let mut loaded_pets = read_db().expect("can fetch pet list");

    loop {
        render(
            &mut terminal,
            &menu_titles,
            active_menu_item,
            &mut pet_list_state,
            &loaded_pets,
        )?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    tui_cleanup(&mut terminal)?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Data,
                KeyCode::Char('a') => {
                    loaded_pets = add_random_pet_to_db().expect("can add new random pet");
                }
                KeyCode::Char('d') => {
                    remove_pet_at_index(&mut pet_list_state).expect("can remove pet");
                    loaded_pets = read_db().expect("can fetch pet list")
                }
                KeyCode::Down => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected >= amount_pets - 1 {
                            pet_list_state.select(Some(0));
                        } else {
                            pet_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected > 0 {
                            pet_list_state.select(Some(selected - 1));
                        } else {
                            pet_list_state.select(Some(amount_pets - 1));
                        }
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(selected) = pet_list_state.selected() {
                        let mut new_name = String::new();
                        let change_loaded = |pet_list: &mut [Pet], idx: usize, new_str: &str| {
                            pet_list[idx].name = new_str.to_string()
                        };
                        word_input(
                            &rx,
                            &mut new_name,
                            &mut loaded_pets,
                            selected,
                            &mut terminal,
                            &menu_titles,
                            active_menu_item,
                            &mut pet_list_state,
                            change_loaded,
                        )?;
                        if new_name.is_empty() {
                            let rng = rand::thread_rng();
                            new_name = rng.sample_iter(Alphanumeric).take(10).collect();
                        }
                        edit_pet_at_index(&mut pet_list_state, Some(new_name), 0, &mut loaded_pets)
                            .expect("can edit pet");
                    }
                }
                KeyCode::Left => {
                    edit_pet_at_index(&mut pet_list_state, None, -1, &mut loaded_pets)
                        .expect("can edit pet");
                }
                KeyCode::Right => {
                    edit_pet_at_index(&mut pet_list_state, None, 1, &mut loaded_pets)
                        .expect("can edit pet");
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {        
        std::process::Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };

    Ok(())
}

fn read_db() -> Result<Vec<Pet>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_random_pet_to_db() -> Result<Vec<Pet>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;

    let random_pet = Pet::create_placeholder();

    parsed.push(random_pet);
    fs::write(DB_PATH, serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_pet_at_index(pet_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = pet_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, serde_json::to_vec(&parsed)?)?;
        let _amount_pets = read_db().expect("can fetch pet list").len();
        if selected > 0 {
            pet_list_state.select(Some(selected - 1));
        } else {
            pet_list_state.select(Some(0));
        }
    }
    Ok(())
}

fn edit_pet_at_index(
    pet_list_state: &mut ListState,
    name_change: Option<String>,
    age_shift: i8,
    pet_list: &mut [Pet],
) -> Result<(), Error> {
    if let Some(selected) = pet_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
        if let Some(new_name) = name_change {
            parsed[selected].name = new_name.clone();
            pet_list[selected].name = new_name;
        }
        match age_shift {
            z if z > 0 => {
                parsed[selected].age += z as usize;
                pet_list[selected].age = parsed[selected].age;
            }
            w if w < 0 => {
                parsed[selected].age = std::cmp::max(0, (parsed[selected].age as i8) + w) as usize;
                pet_list[selected].age = parsed[selected].age;
            }
            _ => {}
        }
        fs::write(DB_PATH, serde_json::to_vec(&parsed)?)?;
        let _amount_pets = read_db().expect("can fetch pet list").len();
    }
    Ok(())
}
