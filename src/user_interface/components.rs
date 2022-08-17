use crate::gol::models::Cell;
use crate::user_interface::grid::Grid;
use core::time::Duration;
use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent};
use crossterm::execute;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::time::Instant;
use std::usize;
use tui::layout::Alignment;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::canvas::{Canvas, Context, Map, MapResolution};
use tui::widgets::{Block, BorderType, Borders, Paragraph, Tabs};

pub enum GoLEvent {
    Tick,
    Input(KeyEvent),
    Mouse(MouseEvent),
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    Preparation,
    Run,
    Quit,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Preparation => 0,
            MenuItem::Run => 1,
            MenuItem::Home => 2,
            MenuItem::Quit => 3,
        }
    }
}

pub fn input_controller(millis: u64, tx: Sender<GoLEvent>) {
    // This function will run in paralell to the TUI main loop, capturing the input events or the
    // timeout ticks to reload the TUI.
    // Must run in a different thread to avoid blocking the main render loop.
    let tick_rate = Duration::from_millis(millis);

    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture).expect("Failed to enable Mouse capture.");

    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout).expect("poll works") {
            match event::read().expect("Can't read mouse") {
                Event::Mouse(e) => tx.send(GoLEvent::Mouse(e)).expect("Can't send mouse"),
                Event::Key(key) => tx
                    .send(GoLEvent::Input(key))
                    .expect("Can't send key events"),
                _ => (),
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Ok(_) = tx.send(GoLEvent::Tick) {
                last_tick = Instant::now();
            }
        }
    }
}

pub fn event_controller(rx: &Receiver<GoLEvent>) -> Result<Option<MenuItem>, RecvError> {
    match rx.recv()? {
        GoLEvent::Input(event) => match event.code {
            KeyCode::Char('q') => Ok(Some(MenuItem::Quit)),
            KeyCode::Char('r') => Ok(Some(MenuItem::Run)),
            KeyCode::Char('p') => Ok(Some(MenuItem::Preparation)),
            _ => Ok(None),
        },
        GoLEvent::Mouse(event) => {
            println!("Kurwa! {:?}", event);
            Ok(None)
        }
        GoLEvent::Tick => Ok(None),
    }
}

pub fn render_menu(menu_titles: Vec<&str>) -> Vec<Spans> {
    menu_titles
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
        .collect()
}

pub fn render_tabs(menu: Vec<Spans>, option: MenuItem) -> Tabs {
    Tabs::new(menu)
        // Default option to be selected when app starts
        .select(option.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"))
}

pub fn render_preparation<'a>() -> Canvas<'a, impl Fn(&mut Context<'_>)> {
    let grid = Grid::default;
    Canvas::default()
        .block(Block::default().title("Canvas").borders(Borders::ALL))
        .x_bounds([-180.0, 180.0])
        .y_bounds([-89.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Map {
                resolution: MapResolution::High,
                color: Color::White,
            });
        })
}

pub fn render_home<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![
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
        Spans::from(vec![Span::raw(
            "Press 'p' to access preparation, 'r' to run the pattern and 'q' to exit.",
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    )
}

pub fn render_boilderplate<'a>() -> Paragraph<'a> {
    Paragraph::new("Game of Life 2021 - all rights reserved")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Copyright")
                .border_type(BorderType::Plain),
        )
}
