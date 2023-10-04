use tui::{text::Spans, widgets::Table};

pub trait DataRow {
    fn name(&self) -> String;
    fn to_table(&self) -> Table;
    fn cli_name() -> String;
    fn help_part<'a>() -> Vec<Spans<'a>>;
    fn menu_titles() -> Vec<&'static str>;
    fn title() -> &'static str;
    fn row_name() -> &'static str;
    fn create_placeholder() -> Self;
}
