use core::time::Duration;
use std::time::Instant;
use std::{thread, usize};
use std::sync::mpsc;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event;
use crossterm::event::{KeyCode, Event as CEvent};
use serde::{Deserialize, Serialize};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::widgets::{Paragraph, Block, Borders, BorderType, Tabs};
use tui::widgets::canvas::{Canvas, Map, MapResolution, Line, Rectangle};
use tui::style::{Color, Style, Modifier};
use tui::layout::Alignment;
use tui::text::{Span, Spans};

use tui::layout::{Layout, Constraint, Direction};
use std::io;

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
}

#[derive(Serialize, Deserialize, Clone)]
struct Coordinate {
    x: u64,
    y: u64
}

impl Coordinate {
    fn new(x: u64, y: u64) -> Coordinate {
        Coordinate { x: x, y: y }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Cell {
    id: usize,
    position: Coordinate,
    neighbors: usize,
}

// TUI related stuff

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Preparation,
    Run,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Preparation => 0,
            MenuItem::Run => 1,
            MenuItem::Home => 2,
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // This block will run in paralell to our main loop, managing the inputs or the timeout ticks
    // to reload the TUI. Must run in a different thread to avoid blocking the main render loop.
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                // The example I was following didn't documented really well that CEVent is
                // actually crossterm Event with an alias, FML.
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // Now we set up the boilerplate to make us able to render on the screen.
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Finally we can add the elements to be render on the screen.
    let menu_titles = vec!["Preparation", "Run", "Quit"];

    // Default option to be selected when app starts
    let mut active_menu_item = MenuItem::Home;

    Ok(loop {
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

            let boilerplate = Paragraph::new("Game of Life 2021 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            rect.render_widget(boilerplate, chunks[2]);

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

            // Draws a map when Preparation is selected.
            let draw = Canvas::default()
                .block(Block::default().title("Canvas").borders(Borders::ALL))
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .paint(|ctx| {
                    ctx.draw(&Map {
                        resolution: MapResolution::High,
                        color: Color::White
                    });
                    ctx.layer();
                    ctx.draw(&Line {
                        x1: 0.0,
                        y1: 10.0,
                        x2: 10.0,
                        y2: 10.0,
                        color: Color::White,
                    });
                    ctx.draw(&Rectangle {
                        x: 10.0,
                        y: 20.0,
                        width: 10.0,
                        height: 10.0,
                        color: Color::Red
                    });
                });

            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Preparation=> {rect.render_widget(draw, chunks[1])},
                MenuItem::Run => {}
            }
        });

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('r') => active_menu_item = MenuItem::Run,
                KeyCode::Char('p') => active_menu_item = MenuItem::Preparation,
                _ => {}
            },
            Event::Tick => {}
        }
    })
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Conway's Game of Life on a Rust TUI.",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'p' to access preparation, 'r' to run the pattern and 'q' to exit.")]),
    ])
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
