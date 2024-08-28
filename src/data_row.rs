use tui::{text::Spans, widgets::Table};

pub trait DataRow {
    fn name(&self) -> String;
    fn set_name<T: Into<String>>(&mut self, new_name: T);

    fn to_table(&self) -> Table;
    fn cli_name() -> String;
    fn help_part<'a>() -> Vec<Spans<'a>>;
    fn menu_titles() -> Vec<&'static str>;
    fn title() -> &'static str;
    fn row_name() -> &'static str;
    fn create_placeholder() -> Self;
}

pub trait Aged {
    fn age(&self) -> u8;
    fn set_age<T: Into<u8>>(&mut self, new_age: T);
    fn decr_age<T: Into<u8>>(&mut self, decr_amt: T) {
        let real_decr_amt = decr_amt.into();
        if self.age() <= real_decr_amt {
            self.set_age(0);
        } else {
            let new_age = self.age() - real_decr_amt;
            self.set_age(new_age);
        }
    }
    fn incr_age<T: Into<u8>>(&mut self, incr_amt: T) {
        let new_age = self.age() + incr_amt.into();
        self.set_age(new_age);
    }
}
