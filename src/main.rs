use std::fs;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
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
        if self.items.len() != 0 {                       // Check if the list is not empty.
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
    }

    fn previous(&mut self) {
        if self.items.len() != 0 {
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
struct App {
    pend_items: StatefulList<String>,
    comp_items: StatefulList<String>,
    input: String,
    input_mode: bool,
}

fn main() -> Result<(), io::Error>{

    // Pending items from file.
    let pend_items: Vec<String> = read_file()?;

    // Setup terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run it.
    let tick_rate = Duration::from_millis(250);
    let mut app = App{
        pend_items: StatefulList::with_items(pend_items),
        comp_items: StatefulList::with_items(vec![]),    // Completed items are not stored since they are already completed. 
        input: String::new(),
        input_mode: false,
    };

    app.pend_items.state.select(Some(0));                // Start with the first item selected. 
    let res = run_app(&mut terminal, app, tick_rate);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    
    Ok(())
}

// ---- Main loop ----

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();
    
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()?{
                if app.input_mode {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        },
                        KeyCode::Backspace => {
                            app.input.pop();
                        },
                        KeyCode::Enter => {
                            app.input_mode = false;
                            app.pend_items.items.push(app.input.clone());
                            app.input = "".to_string();  // Clear the input.
                        },
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Down => app.pend_items.next(),
                        KeyCode::Up => app.pend_items.previous(),
                        KeyCode::Char('n') => app.input_mode = true,
                        KeyCode::Char('c') => completed_task(&mut app),
                        KeyCode::Char('r') => app.pend_items.remove_selected(),
                        KeyCode::Char('q') => {
                            write_file(&app.pend_items.items)?;
                            return Ok(());
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

// ---- UI ----

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    // Create two chunks, one to diplay the usege and another to display the ui.
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
    .split(f.size());

    let lower_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .margin(1)
    .constraints(
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
        .as_ref(),
    )
    .split(chunks[1]);

    // ---- Usage ----
    let title = "Usage:
        Press n to add item.
        Press r to remove item.
        Press c to mark item as complete.
        Press q to exit.
    ";

    let paragraph = Paragraph::new(title);

    f.render_widget(paragraph, chunks[0]);

    // ---- Pending items list. ----

    // Iterate through all elements in the `items` app and parse it into a ListItem vector.
    let pend_items: Vec<ListItem> = app
        .pend_items
        .items
        .iter()
        .map(|i| {
            let lines = i.as_str();
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one.
    let pend_items = List::new(pend_items)
        .block(Block::default().borders(Borders::ALL).title("Pending items"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the pending list
    f.render_stateful_widget(pend_items, lower_chunks[0], &mut app.pend_items.state);

    // ---- Completed items list. ----

    let comp_items: Vec<ListItem> = app
        .comp_items
        .items
        .iter()
        .map(|i| {
            let lines = i.as_str();
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one.
    let comp_items = List::new(comp_items)
        .block(Block::default().borders(Borders::ALL).title("Completed items"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list.
    f.render_stateful_widget(comp_items, lower_chunks[1], &mut app.comp_items.state);

    // ---- Popup ----

    // Input mode.
    if app.input_mode {

        let size = f.size();

        // Show popup.
        let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("New item:"));
        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area);                    // This clears out the background.
        f.render_widget(input, area);

        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering.
        f.set_cursor(
            // Put cursor past the end of the input text.
            area.x + app.input.len() as u16 + 1,
            // Move one line down, from the border to the input line.
            area.y + 1,
        );
    }
}

// ---- Utitlity functions. ----

fn read_file() -> io::Result<Vec<String>> {
    let mut items: Vec<String> = vec![];

    let data = match fs::read_to_string(FILENAME) {
        Ok(v) => v, 
        Err(e) => return Err(e),
    };

    if !data.is_empty() {
        items = serde_json::from_str(&data).unwrap();
    }

    Ok(items)
}

fn write_file(items: &Vec<String>) -> io::Result<()> {
    let data = serde_json::to_string(items).unwrap();

    match fs::write(FILENAME, data) {
        Ok(_) => Ok(()), 
        Err(e) => Err(e)
    }
}

// Stores the completed item to the completed items list and deletes the item from the pending list.
fn completed_task(app: &mut App) {
    let i = app.pend_items.state.selected();

    if app.pend_items.items.len() != 0 {
        app.comp_items.items.push(app.pend_items.items[i.unwrap()].clone());
        app.pend_items.items.remove(i.unwrap());
        app.pend_items.state.select(Some(0));
    }
}

// Helper function to create a centered rect using up certain percentage of the available rect `r`. 
// Mainly used by the popup that prompts user input.
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
