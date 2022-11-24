use std::fs;
use text_colorizer::Colorize;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

const FILENAME: &str = "TODO.json";

// List struct declaration and methods.
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

// This struct holds the current state of the app.
struct App<'a> {
    items: StatefulList<&'a str>,
    options: Vec<&'a str>,
}

// Utility functions

fn read_file() -> io::Result<Vec<&str>> {
    let data = match fs::read_to_string(FILENAME) {
        Ok(v) => v, 
        Err(e) => return Err(e),
    };

    if !data.is_empty() {
        let list: Vec<&str> = serde_json::from_str(&data).unwrap();

        return Ok(list.clone());
    }

    Ok(vec![])
}

fn write_file(list: &Vec<String>) -> io::Result<()> {

    let data = serde_json::to_string(list).unwrap();

    match fs::write(FILENAME, data) {
        Ok(_) => Ok(()), 
        Err(e) => Err(e)
    }

}

fn print_list(list: &Vec<String>) {
    println!("{} list: \n", "TODO".green());

    let mut i: u32 = 1;

    for task in list {
        print!("{}. {}", i, task);

        i += 1;
    }
}

fn add_task(list: &mut Vec<String>) {

    println!("New task:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    list.push(input);

}

fn remove_task(list: &mut Vec<String>) {
    
}

fn main() -> Result<(), io::Error>{

    // Fetch list from file
    let list = read_file().expect("reading file");

    // Setup terminal
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App{
        items: StatefulList::with_items(list),
        options: vec![
            "Add list element",
            "Remove list element",
            "Exit program",
        ],
    };

    let res = run_app(&mut terminal, app, tick_rate);

    // Restore terminal
    disable_raw_mode()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(*i)];
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

}
/*
fn main() {
 
    println!("\n\n {} - This is a simple todo program", "WELCOME".green().bold());

    let mut list = read_file().expect("reading file");

    if list.is_empty() {

        println!("\nThere are no TODO tasks.\n");

    } else {

        println!("\n Current tasks: \n");
        print_list(&list);
    }

    let mut choice: u8 = 0;

    while choice != 4 {

        choice = prompt_input();

        match choice{
            1 => add_task(&mut list),
            2 => remove_task(&mut list),
            3 => print_list(&list),
            4 => match write_file(&list) {
                Ok(()) => println!("File saved successfully"),
                Err(e) => eprintln!("{} saving file: {}", "ERROR".red(), e),
            }
            _ => println!("No option selected, invalid input"),
        };
    }
    
}
*/