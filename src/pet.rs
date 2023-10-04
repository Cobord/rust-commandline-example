use chrono::prelude::*;
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use tui::{
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table},
};

use crate::data_row::DataRow;

#[derive(Serialize, Deserialize, Clone)]
pub struct Pet {
    pub id: usize,
    pub name: String,
    pub category: String,
    pub age: usize,
    pub created_at: DateTime<Utc>,
}

impl DataRow for Pet {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn menu_titles() -> Vec<&'static str> {
        vec!["Home", "Pets", "Add", "Edit Name", "Delete", "Quit"]
    }

    fn title() -> &'static str {
        "Pets"
    }

    fn row_name() -> &'static str {
        "pet"
    }

    fn cli_name() -> String {
        "pet CLI".to_string()
    }

    fn help_part<'a>() -> Vec<Spans<'a>> {
        vec![
            Spans::from(vec![Span::raw(
                "Press 'p' to access pets, 'a' to add random new pets,",
            )]),
            Spans::from(vec![Span::raw(
                "'e' to edit the name of currently selected pet",
            )]),
            Spans::from(vec![Span::raw(
                "and 'd' to delete the currently selected pet.",
            )]),
        ]
    }

    fn to_table(&self) -> Table {
        Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(self.id.to_string())),
            Cell::from(Span::raw(self.name.to_string())),
            Cell::from(Span::raw(self.category.to_string())),
            Cell::from(Span::raw(self.age.to_string())),
            Cell::from(Span::raw(self.created_at.to_string())),
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
                "Category",
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
        let catsdogs = match rng.gen_range(0, 1) {
            0 => "cats",
            _ => "dogs",
        };
        Self {
            id: rng.gen_range(0, 9999999),
            name: rng.sample_iter(Alphanumeric).take(10).collect(),
            category: catsdogs.to_owned(),
            age: rng.gen_range(1, 15),
            created_at: Utc::now().round_subsecs(0),
        }
    }
}
