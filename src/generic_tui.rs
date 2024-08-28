use crate::data_row::DataRow;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::{sync::mpsc::Receiver, thread::JoinHandle};
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Table, Tabs},
    Terminal,
};

pub const DB_PATH: &str = "./data/db.json";

pub(crate) type WhichRow = usize;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MenuItem {
    Home,
    Data,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Data => 1,
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub(crate) fn io_handler() -> (Receiver<Event<event::KeyEvent>>, JoinHandle<()>) {
    enable_raw_mode().expect("can run in raw mode");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    let jh: JoinHandle<()> = thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                #[allow(clippy::collapsible_if)]
                if tx.send(Event::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            }
        }
    });
    (rx, jh)
}

pub(crate) fn get_terminal(
) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub(crate) fn get_data_list_state() -> ListState {
    let mut data_list_state = ListState::default();
    data_list_state.select(Some(0));
    data_list_state
}

pub(crate) fn tui_cleanup<B: Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}

pub(crate) fn render<T: DataRow, B: Backend>(
    terminal: &mut Terminal<B>,
    menu_titles: &[&str],
    active_menu_item: MenuItem,
    data_list_state: &mut ListState,
    loaded_data: &[T],
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|rect| {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size);

        let copyright = Paragraph::new(T::cli_name() + " 2020 - all rights reserved")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
            );

        let menu = menu_titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tabs = Tabs::new(menu)
            .select(active_menu_item.into())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"));

        rect.render_widget(tabs, chunks[0]);
        match active_menu_item {
            MenuItem::Home => rect.render_widget(render_home::<T>(), chunks[1]),
            MenuItem::Data => {
                let data_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                    .split(chunks[1]);
                let (left, right) = render_data(data_list_state, loaded_data);
                rect.render_stateful_widget(left, data_chunks[0], data_list_state);
                rect.render_widget(right, data_chunks[1]);
            }
        }
        rect.render_widget(copyright, chunks[2]);
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn word_input<T: DataRow, B: Backend>(
    rx: &Receiver<Event<event::KeyEvent>>,
    new_name: &mut String,
    loaded_data: &mut [T],
    selected: WhichRow,
    terminal: &mut Terminal<B>,
    menu_titles: &[&str],
    active_menu_item: MenuItem,
    data_list_state: &mut ListState,
    change_loaded: fn(&mut [T], WhichRow, &str),
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match rx.recv()? {
            Event::Input(evt) => match evt.code {
                KeyCode::Backspace => {
                    let _ = new_name.pop();
                }
                KeyCode::Char(c) => {
                    if c.is_alphanumeric() {
                        new_name.push(c);
                    } else {
                        break;
                    }
                }
                _ => {
                    break;
                }
            },
            Event::Tick => {
                change_loaded(loaded_data, selected, new_name);
                render(
                    terminal,
                    menu_titles,
                    active_menu_item,
                    data_list_state,
                    loaded_data,
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn render_home<'a, T: DataRow>() -> Paragraph<'a> {
    let mut welcome_part = vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            T::cli_name(),
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
    ];
    welcome_part.extend(T::help_part());
    let home = Paragraph::new(welcome_part)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );
    home
}

pub(crate) fn render_data<'a, T: DataRow>(
    data_list_state: &ListState,
    data_list: &'a [T],
) -> (List<'a>, Table<'a>) {
    let data = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(T::title())
        .border_type(BorderType::Plain);

    let items: Vec<_> = data_list
        .iter()
        .map(|cur_data| {
            ListItem::new(Spans::from(vec![Span::styled(
                cur_data.name().clone(),
                Style::default(),
            )]))
        })
        .collect();
    #[allow(clippy::expect_fun_call)]
    let selected_datum = data_list
        .get(
            data_list_state
                .selected()
                .expect(("there is always a selected".to_owned() + T::row_name()).as_str()),
        )
        .expect("exists");

    let list = List::new(items).block(data).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let data_detail = selected_datum.to_table();

    (list, data_detail)
}
