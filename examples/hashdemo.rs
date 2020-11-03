#[allow(dead_code)]
mod util;
use crate::util::event::{Config, Event, Events};

use std::{error::Error, io, time::Duration};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, Clear};
use termion::raw::IntoRawMode;
use termion::event::Key;
use argh::FromArgs;
use itertools::Itertools;

use hashlife::hashlife::{Cell, HashLife};
use hashlife::{EdgeRule, GameOfLife};

const BLOCK_HALF_UPPER: &'static str = "▀";
const BLOCK_HALF_LOWER: &'static str = "▄";
const BLOCK_FULL: &'static str = "█";

/// Hashlife demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
}


fn main() -> Result<(), Box<dyn Error>> {
    print!("{}", termion::clear::All);
    let cli: Cli = argh::from_env();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(cli.tick_rate),
        ..Default::default()
    };
    let events = Events::with_config(config);

    let mut gol = None;
    let mut focus = (0,0);

    loop {
        terminal.draw(|f| {
            let window_size = f.size();
            let container_size = window_size;
            let container = Block::default()
                .borders(Borders::ALL)
            ;
            // Get the size of the grid
            let grid_size = container.inner(container_size);
            let viewport_width = grid_size.width;
            let viewport_height = grid_size.height;

            if let None = gol {
                let width = viewport_width as usize;
                let height = (viewport_height * 2) as usize;
                let edge_rules = EdgeRule::Wrap(width, height);
                gol = Some(HashLife::new(edge_rules));
            }
            let gol = gol.as_mut().unwrap();

            let title = format!(" Game of Life ({}x{}) @ {:?}, Generation: {} ",
                gol.width(),
                gol.height(),
                focus,
                gol.get_generation()
            );
            let container = container.title(title);

            f.render_widget(Clear, window_size);
            f.render_widget(container, window_size);

            // let x_half = gol.width() as isize / 2;
            // let y_half = gol.height() as isize / 2;
            // let x_lower = 0 - x_half;
            // let y_lower = 0 - y_half;
            // let x_upper = 0 + x_half;
            // let y_upper = 0 + y_half;
            // let xs = x_lower..x_upper;
            // let ys = y_lower..y_upper;
            // let positions = ys
            //     .clone()
            //     .map(|y| xs
            //         .clone()
            //         .zip(std::iter::repeat(y))
            //         .collect::<Vec<_>>()
            //     )
            //     .flatten()
            //     .collect::<Vec<_>>();
            // let cells = positions
            //     .into_iter()
            //     .map(|(x, y)| gol.top().get_cell(x, y))
            //     .collect::<Vec<_>>();
            // cells.into_iter()
            // .chunks(viewport_width as usize * 2)
            // .into_iter()
            // .map(|it| {
            //     let cells: Vec<Cell> = it.collect();
            //     let top = &cells[..(viewport_width as usize)];
            //     let bottom = &cells[(viewport_width as usize)..];
            //     top.into_iter().zip(bottom.into_iter())
            //             .map(|(t,b )| {
            //                 match (t, b) {
            //                     (Cell::Alive, Cell::Alive) => BLOCK_FULL,
            //                     (Cell::Alive, Cell::Dead)  => BLOCK_HALF_UPPER,
            //                     (Cell::Dead, Cell::Alive) => BLOCK_HALF_LOWER,
            //                     (Cell::Dead, Cell::Dead) => " ",
            //                 }
            //             })
            //         .collect::<String>()
            // })
            //     .enumerate()
            //     .for_each(|(i, s)| {
            //         let line = Block::default().borders(Borders::NONE).title(s);
            //         let mut area = grid_size.clone();
            //         area.y += i as u16;
            //         area.height = 2;
            //         f.render_widget(line, area);
            //     })
            // ;
        }).expect("failed to draw terminal");
        
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                },
                Key::Up => focus.1 += 1,
                Key::Down => focus.1 -= 1,
                Key::Left => focus.0 -= 1,
                Key::Right => focus.0 += 1,
                _ => {}
            },
            Event::Tick => {
                if let Some(gol) = &mut gol {
                    gol.tick();
                }
            }
        }
    }
    Ok(())
}