#[allow(dead_code)]
mod util;
use crate::util::event::{Config, Event, Events};

use std::{error::Error, io, time::Duration};
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use termion::event::Key;


use tui::widgets::{canvas::Canvas, Block, Borders, Clear};
use tui::layout::{Layout, Constraint, Direction};

use tui::style::{Color, Style};

use itertools::Itertools;

use hashlife::Cell;

// use crate::util::event::{Config, Event, Events};
// use std::{error::Error, io, time::Duration};
// use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
// use tui::{
//     backend::TermionBackend,
//     layout::{Constraint, Direction, Layout, Rect},
//     style::Color,
//     widgets::{
//         canvas::{Canvas, Map, MapResolution, Rectangle},
//         Block, Borders,
//     },
//     Terminal,
// };

use hashlife::HashLife;

const BLOCK_HALF_UPPER: &'static str = "▀";
const BLOCK_HALF_LOWER: &'static str = "▄";
const BLOCK_FULL: &'static str = "█";


fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Default::default()
    };
    let events = Events::with_config(config);

    let mut hashlife = None;

    loop {
        terminal.draw(|f| {
            let window_size = f.size();
            let container_size = window_size;
            // let title = format!(" Generation: {} ", hashlife.get_generation());
            let container = Block::default()
                // .title(title)
                .borders(Borders::ALL)
            ;
            // Get the size of the grid
            let grid_size = container.inner(container_size);
            let viewport_width = grid_size.width;
            let viewport_height = grid_size.height;

            if let None = hashlife {
                hashlife = Some(HashLife::new(viewport_width as usize, (viewport_height * 2) as usize));
            }
            let mut hashlife = hashlife.as_mut().unwrap();

            let title = format!(" Hashlife ({}x{}), Generation: {}",
                hashlife.width(),
                hashlife.height(),
                hashlife.get_generation()
            );
            let container = container.title(title);

            f.render_widget(Clear, window_size);
            f.render_widget(container, window_size);

            hashlife.cells()
                .chunks(viewport_width as usize * 2)
                .into_iter()
                .map(|iter| {
                    iter
                        .chunks(viewport_width as usize)
                        .map(|cells| cells.to_vec().clone())
                })
                .into_iter()
                .map(|mut it| {
                    let top = it.next().unwrap();
                    let bottom = it.next().unwrap();
                    top.into_iter().zip(bottom.into_iter())
                        .map(|(t,b )| {
                            match (t, b) {
                                (Cell::Alive, Cell::Alive) => BLOCK_FULL,
                                (Cell::Alive, Cell::Dead)  => BLOCK_HALF_UPPER,
                                (Cell::Dead, Cell::Alive) => BLOCK_HALF_LOWER,
                                (Cell::Dead, Cell::Dead) => " ",
                            }
                        })
                    .collect::<String>()
                })
                .enumerate()
                .for_each(|(i, s)| {
                    let line = Block::default().borders(Borders::NONE).title(s);
                    let mut area = grid_size.clone();
                    area.y += i as u16;
                    area.height = 2;
                    // let x = grid_size.x;
                    // let y = grid_size.y + i as u16;
                    // let area = tui::layout::Rect::new(x, y, grid_size.width, 1);
                    // println!("{:?}", area);
                    f.render_widget(line, area);
                })
            ;
        }).expect("failed to draw terminal");
        
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                _ => {}
            },
            Event::Tick => {
                if let Some(hashlife) = &mut hashlife {
                    hashlife.tick();
                }
            }
        }
    }
    Ok(())
}