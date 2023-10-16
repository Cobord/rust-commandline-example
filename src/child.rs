use chrono::prelude::*;
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table},
};

use crate::data_row::{DataRow, Aged};

#[derive(Serialize, Deserialize, Clone)]
pub struct Child {
    pub id: usize,
    pub name: String,
    pub age: u8,
    pub birthdate: DateTime<Utc>,
}

impl DataRow for Child {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name<T : Into<String>>(&mut self, new_name: T) {
        self.name = new_name.into();
    }

    fn menu_titles() -> Vec<&'static str> {
        vec!["Home", "Children", "Add", "Edit Name", "Delete", "Quit"]
    }

    fn home_char() -> char {
        'h'
    }
    fn quit_char() -> char {
        'q'
    }
    fn data_char() -> char {
        'c'
    }
    fn add_char() -> char {
        'a'
    }
    fn del_char() -> char {
        'd'
    }
    fn edit_char() -> char {
        'e'
    }

    fn title() -> &'static str {
        "Children"
    }

    fn row_name() -> &'static str {
        "Child"
    }

    fn cli_name() -> String {
        "Child CLI".to_string()
    }

    fn help_part<'a>() -> Vec<Spans<'a>> {
        vec![
            Spans::from(vec![Span::raw(
                "Press 'c' to access children, 'a' to add random new children,",
            )]),
            Spans::from(vec![Span::raw(
                "'e' to edit the name of currently selected child",
            )]),
            Spans::from(vec![Span::raw(
                "and 'd' to delete the currently selected child.",
            )]),
        ]
    }

    fn to_table(&self) -> Table {
        Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(self.id.to_string())),
            Cell::from(Span::raw(self.name.to_string())),
            Cell::from(Span::raw(self.age.to_string())),
            Cell::from(Span::raw(self.birthdate.to_string())),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "ID",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Name",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Age",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Created At",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
    }

    fn create_placeholder() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: rng.gen_range(0, 9999999),
            name: rng.sample_iter(Alphanumeric).take(10).collect(),
            age: rng.gen_range(1, 15),
            birthdate: Utc::now().round_subsecs(0),
        }
    }
}

impl Aged for Child {
    fn age(&self) -> u8 {
        self.age
    }

    fn set_age<T : Into<u8>>(&mut self,new_age: T) {
        self.age = new_age.into();
    }
}