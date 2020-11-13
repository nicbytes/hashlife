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
use getrandom::getrandom;
use itertools::Itertools;

use hashlife::{Hashlife, Edge, BoundingBox};
use hashlife::Automata;

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
    let mut buffer = vec![];

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
                let edge_rules = Edge::Torus;
                buffer = vec![ 0u8; width * height];
                let mut rbuffer = vec![ 0u8; width * height];
                getrandom(&mut rbuffer);
                let hashlife = Hashlife::from_array(rbuffer, width, height, edge_rules);
                gol = Some(hashlife);
            }
            let gol = gol.as_mut().unwrap();

            let title = format!(" Game of Life ({}x{}), Generation: {} ",
                viewport_width,
                viewport_height,
                gol.get_generation()
            );
            let container = container.title(title);

            f.render_widget(Clear, window_size);
            f.render_widget(container, window_size);

            let right = (viewport_width / 2) as isize;
            let left = viewport_width as isize - right;
            let top = (viewport_height / 2) as isize;
            let bottom = viewport_height as isize - top;
            let bound = BoundingBox::from(top, bottom, left, right);
            if gol.get_generation() == 0 {
                gol.draw_to_viewport_buffer(&mut buffer, bound);
            } else {
                gol.draw_diff_to_viewport_array(&mut buffer, bound);
            }
            

            buffer.iter()
                .map(|c| Automata::from(*c as usize))
                .chunks(viewport_width as usize * 2)
                .into_iter()
                .map(|iter| {
                    let a = iter.chunks(viewport_width as usize);
                    let b = a.into_iter();
                    b.map(|cells| cells.collect::<Vec<Vec<Automata>>>().clone())
                })
                .into_iter()
                .map(|mut it| {
                    let top = it.next().unwrap();
                    let bottom = it.next().unwrap();
                    top.into_iter().zip(bottom.into_iter())
                        .map(|(t,b )| {
                            match (t, b) {
                                (Automata::Alive, Automata::Alive) => BLOCK_FULL,
                                (Automata::Alive, Automata::Dead)  => BLOCK_HALF_UPPER,
                                (Automata::Dead, Automata::Alive) => BLOCK_HALF_LOWER,
                                (Automata::Dead, Automata::Dead) => " ",
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
                if let Some(gol) = &mut gol {
                    gol.next_generation();
                }
            }
        }
    }
    Ok(())
}