mod gol;
mod user_interface;
use crate::user_interface::components::{
    event_controller, input_controller, render_boilderplate, render_home, render_menu,
    render_preparation, render_tabs, MenuItem,
};
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || input_controller(200, tx));
    // Now we set up the boilerplate to make us able to render on the screen.
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Finally we can add the elements to be render on the screen.
    let mut active_menu_item = MenuItem::Home;

    loop {
        terminal.draw(|rect| {
            let menu_titles = vec!["Preparation", "Run", "Quit"];
            let menu = render_menu(menu_titles);
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

            rect.render_widget(render_tabs(menu, active_menu_item), chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Preparation => rect.render_widget(render_preparation(), chunks[1]),
                MenuItem::Run => (),
                MenuItem::Quit => (),
            };
            rect.render_widget(render_boilderplate(), chunks[2]);
        })?;
        match event_controller(&rx).expect("Error processing inputs") {
            Some(MenuItem::Home) => {
                active_menu_item = MenuItem::Home;
            }
            Some(MenuItem::Preparation) => {
                active_menu_item = MenuItem::Preparation;
            }
            Some(MenuItem::Run) => {
                active_menu_item = MenuItem::Run;
            }
            Some(MenuItem::Quit) => break,
            _ => {}
        };
    }

    let mut stdout = std::io::stdout();
    execute!(stdout, DisableMouseCapture).expect("Mouse capture has not been disabled.");
    disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;
    Ok(())
}
