use itertools::Itertools;
use regex::Regex;

use crate::automata::Automata;

pub struct RleData {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Automata   >,
}

#[derive(Eq, PartialEq, Debug)]
enum RleCharacter {
    Number,
    NewLine,
    Dead,
    Alive,
    End,
}

#[derive(Debug, Copy, Clone)]
enum RleToken {
    Number(u32),
    NewLine,
    Dead,
    Alive,
    End,
}

#[derive(Debug, Copy, Clone)]
enum RleElement {
    NewLine(u32),
    Dead(u32),
    Alive(u32),
    End,
}


pub fn load(rle_string: &str) -> RleData {
    let re = Regex::new(r"x\s*=\s*(\d+)\s*,\s*y\s*=\s*(\d+)")
        .expect("Regex failed to compile");

    // The iterator starts here
    let mut iterator_to_xy_line = rle_string
        .lines()
        .map(|line| line.trim_left())
        // remove comment lines
        .filter(|line| ! line.starts_with("#"))
        // remove empty lines
        .filter(|line| line.trim().len() != 0);
    
    let xy_line = iterator_to_xy_line.next().expect("cannot read rle source");

    // collect the width and height
    let cap = re.captures(xy_line).expect("Rle string input is not valid");
    let x = String::from(cap.get(1).unwrap().as_str());
    let y = String::from(cap.get(2).unwrap().as_str());
    let width_from_file: u32 = x.parse().expect(&format!("error parsing x (`{}`)", x));
    let height_from_file: u32 = y.parse().expect(&format!("error parsing y (`{}`)", y));

    let mut cells = iterator_to_xy_line
        // Convert to chars
        .flat_map(|line| line.chars())
        .into_iter()
        // Identify adjacent tokens and group them together
        .group_by(|c| match *c {
            'b' => RleCharacter::Dead,
            'o' => RleCharacter::Alive,
            '$' => RleCharacter::NewLine,
            '!' => RleCharacter::End,
             _  => RleCharacter::Number,
        })
        .into_iter()
        // Convert characters into tokens (particularly Number)
        .map(|(key, group)| {
            match key {
                RleCharacter::Dead => RleToken::Dead,
                RleCharacter::Alive => RleToken::Alive,
                RleCharacter::NewLine => RleToken::NewLine,
                RleCharacter::End => RleToken::End,
                RleCharacter::Number => RleToken::Number(group.collect::<String>().parse::<u32>().unwrap()),
            }
        })
        // A super simple parser
        .batching(|it| {
            match it.next() {
                None => None,
                Some(RleToken::Dead) => Some(RleElement::Dead(1)),
                Some(RleToken::Alive) => Some(RleElement::Alive(1)),
                Some(RleToken::NewLine) => Some(RleElement::NewLine(1)),
                Some(RleToken::End) => Some(RleElement::End),
                Some(RleToken::Number(x)) => {
                    match it.next() {
                        None => None,
                        Some(RleToken::Dead) => Some(RleElement::Dead(x)),
                        Some(RleToken::Alive) => Some(RleElement::Alive(x)),
                        Some(RleToken::NewLine) => Some(RleElement::NewLine(x)),
                        Some(RleToken::Number(_)) => panic!("have a number after a number."),
                        Some(RleToken::End) => panic!("cannot have multiple ends"),
                    }
                }
            }
        })
        // Split up by lines
        .collect::<Vec<RleElement>>()
        .into_iter()
        .group_by(|key| match key {
            RleElement::NewLine(_) | RleElement::End => false,
            _ => true
        })
        .into_iter()
        // Oranise into (line, newline_or_end)
        .map(|(_, group)| {
            group.collect::<Vec<RleElement>>()
        })
        .chunks(2)
        .into_iter()
        .map(|mut it| {
            let line = it.next().expect("No next iterator");
            let new_line_or_end = it
                .next()
                .expect("No next iterator")
                .get(0)
                .expect("Missing newline element")
                .clone();
            (line, new_line_or_end)
        })
        // Convert from RleElements to Vec<Automata>
        .map(|(line, newline_or_end): (Vec<RleElement> ,RleElement)| {
            let complete_line = construct_line(line, width_from_file);
            let newlines = if let RleElement::NewLine(x) = newline_or_end {
                vec![ Automata::Dead; ((x-1) * width_from_file) as usize]
            } else { vec![] };
            let mut section = Vec::new();
            section.extend(complete_line);
            section.extend(newlines);
            section
        })
        .flatten()
        // Expand
        .collect::<Vec<Automata>>()
    ;
    cells.extend(vec![ Automata::Dead; (width_from_file * height_from_file) as usize - cells.len()]);

    RleData {
        width: width_from_file,
        height: height_from_file,
        cells,
    }
}

fn construct_line(line: Vec<RleElement>, width: u32) -> Vec<Automata> {
    let width = width as usize;
    let mut line = line
        .into_iter()
        .map(|element| match element {
            RleElement::Alive(x) => vec![ Automata::Alive; x as usize],
            RleElement::Dead(x) => vec![ Automata::Dead; x as usize],
            _ => panic!("invalid line"),
        })
        .flatten()
        .collect::<Vec<Automata>>();
    if line.len() < width {
        let missing_cells = vec![ Automata::Dead; width as usize - line.len()];
        line.extend(missing_cells);
        line
    } else {
        line.into_iter().take(width).collect()
    }
}

pub fn load_spaceships(width: u32, height: u32) -> Vec<Automata> {
    let raw_contents = include_str!("../spaceships.rle");

    let xy_line = raw_contents
        .lines()
        .skip(1)
        .next()
        .expect("cannot read ship source.");
    let x: String = xy_line.chars().skip(4).take(3).collect();
    let width_from_file: u32 = x.parse().expect(&format!("x (`{}`) is not a number", x));
    let y: String = xy_line.chars().skip(13).take(3).collect();
    let height_from_file: u32 = y.parse().expect(&format!("y (`{}`) is not a number", y));
    // unsafe {
    //     log!("width_from_file: {}", width_from_file);
    //     log!("height_from_file: {}", height_from_file);
    //     log!("width: {}", width);
    //     log!("height: {}", height);
    // }

    let width_of_grid = if width > width_from_file { width } else { width_from_file };
    let height_of_grid = if height > height_from_file { height } else { height_from_file };
    println!("width: {}, height: {}", width, height);

    

    // let coordinates_string: String = raw_contents
    let mut stream = raw_contents
        .lines()
        .skip(2)
        .flat_map(|line| line.chars())
        .collect::<Vec<char>>()
        .into_iter()
        // Identify adjacent tokens and group them together
        .group_by(|c| match *c {
            'b' => RleCharacter::Dead,
            'o' => RleCharacter::Alive,
            '$' => RleCharacter::NewLine,
            '!' => RleCharacter::End,
             _  => RleCharacter::Number,
        })
        .into_iter()
        // Convert characters into tokens (particularly Number)
        .map(|(key, group)| {
            match key {
                RleCharacter::Dead => RleToken::Dead,
                RleCharacter::Alive => RleToken::Alive,
                RleCharacter::NewLine => RleToken::NewLine,
                RleCharacter::End => RleToken::End,
                RleCharacter::Number => RleToken::Number(group.collect::<String>().parse::<u32>().unwrap()),
            }
        })
        // A super simple parser
        .batching(|it| {
            match it.next() {
                None => None,
                Some(RleToken::Dead) => Some(RleElement::Dead(1)),
                Some(RleToken::Alive) => Some(RleElement::Alive(1)),
                Some(RleToken::NewLine) => Some(RleElement::NewLine(1)),
                Some(RleToken::End) => Some(RleElement::End),
                Some(RleToken::Number(x)) => {
                    match it.next() {
                        None => None,
                        Some(RleToken::Dead) => Some(RleElement::Dead(x)),
                        Some(RleToken::Alive) => Some(RleElement::Alive(x)),
                        Some(RleToken::NewLine) => Some(RleElement::NewLine(x)),
                        Some(RleToken::Number(_)) => panic!("have a number after a number."),
                        Some(RleToken::End) => panic!("cannot have multiple ends"),
                    }
                }
            }
        })
        // Split up by lines
        .collect::<Vec<RleElement>>()
        .into_iter()
        .group_by(|key| match key {
            RleElement::NewLine(_) | RleElement::End => false,
            _ => true
        })
        .into_iter()
        // Oranise into (line, newline_or_end)
        .map(|(_, group)| {
            group.collect::<Vec<RleElement>>()
        })
        .chunks(2)
        .into_iter()
        .map(|mut it| {
            let line = it.next().expect("No next iterator");
            let new_line_or_end = it
                .next()
                .expect("No next iterator")
                .get(0)
                .expect("Missing newline element")
                .clone();
            (line, new_line_or_end)
        })
        // Convert from RleElements to Vec<Cell>
        .map(|(line, newline_or_end): (Vec<RleElement> ,RleElement)| {
            let complete_line = construct_line(line, width);
            let newlines = if let RleElement::NewLine(x) = newline_or_end {
                vec![ Automata::Dead; ((x-1) * width) as usize]
            } else { vec![] };
            let mut section = Vec::new();
            section.extend(complete_line);
            section.extend(newlines);
            section
        })
        .flatten()
        // Truncate
        .take((width * height) as usize)
        // Expand
        .collect::<Vec<Automata>>()
    ;
    stream.extend(vec![ Automata::Dead; (width * height) as usize - stream.len()]);
    stream
}