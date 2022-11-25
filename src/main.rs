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
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
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

    fn remove_selected(&mut self) {

        let i = self.state.selected();

        if self.items.len() != 0 {
            self.items.remove(i.unwrap());
            self.state.select(Some(0));
        }
    }
}

// This struct holds the current state of the app.
struct App<'a> {
    items: StatefulList<&'a str>,
    options: Vec<&'a str>,
}

// Utility functions

fn read_file() -> io::Result<Vec<String>> {
    let data = match fs::read_to_string(FILENAME) {
        Ok(v) => v, 
        Err(e) => return Err(e),
    };

    if !data.is_empty() {
        let list: Vec<String> = serde_json::from_str(&data).unwrap();

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

fn promt_input() -> & 'static str {
    
    // TODO: stert with one element selected and finish the file write and reach functions as well as the add functions :P

    "ASD"
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn main() -> Result<(), io::Error>{

    // Fetch list from file - TODO: Be able to fetch list from file as a &str
    let list: Vec<&str> = vec!["adfdas", "adfdas", "adfdas", "adfdas", "adfdas"];

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
            "Press 3 to exit program",
            "Press 2 to remove selected list element",
            "Press 1 to add list element",
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
    
    let mut sel_list_item = true;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()?{
                match key.code {
                    KeyCode::Right => sel_list_item = false,
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Char('1') => app.items.items.push(promt_input()),
                    KeyCode::Char('2') => app.items.remove_selected(),
                    KeyCode::Char('3') => {
                        return Ok(());
                    },
                    _ => {}
                }
                
            }
        }
    }
}

// UI

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and parse it into a ListItem vector.
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

    // Let's do the same for the events.
    // The options list doesn't have any state and only displays the avaiable options.
    let events: Vec<ListItem> = app
        .options
        .iter()
        .rev()
        .map(|i| {
            let mut lines = vec![Spans::from(*i)];
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Actions"));

    f.render_widget(events_list, chunks[1]);

    // Popup
    let size = f.size();

    let block = Block::default().title("Popup").borders(Borders::ALL);
    let area = centered_rect(60, 20, size);
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);
}