use std::{
    io::{stdout, Result},
    thread, time,
};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::{seq::SliceRandom, Rng};
use ratatui::{
    backend::CrosstermBackend, style::Stylize, text::Line, widgets::Paragraph, Terminal,
};

const FIELD_COLUMNS: usize = 32 * 2;
const FIELD_ROWS: usize = 16 * 1;

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut rng = rand::thread_rng();

    // Initialize the original field
    // This will never change
    let mut original_field: Vec<Vec<char>> = vec![vec!['_'; FIELD_COLUMNS]; FIELD_ROWS]
        .iter()
        .map(|row| {
            row.iter()
                .map(|e| {
                    let change: bool = rng.gen_bool(1.0 / 3.0);

                    if change {
                        ','
                    } else {
                        *e
                    }
                })
                .collect()
        })
        .collect();

    // Write RAIN
    original_field[FIELD_ROWS / 2][FIELD_COLUMNS / 2 - 2] = 'R';
    original_field[FIELD_ROWS / 2][FIELD_COLUMNS / 2 - 1] = 'A';
    original_field[FIELD_ROWS / 2][FIELD_COLUMNS / 2] = 'I';
    original_field[FIELD_ROWS / 2][FIELD_COLUMNS / 2 + 1] = 'N';

    let delay = time::Duration::from_millis(150);

    let mut work_field = original_field.clone();

    loop {
        // Change current cell based on neighbors
        // _o, -> (_)
        // _(, | _), -> __,
        let mut display_field = work_field
            .iter()
            .enumerate()
            .map(|row| {
                let mut new_row: Vec<char> = Vec::new();

                for (i, e) in row.1.iter().enumerate() {
                    let neighbors = get_neighbors(row.1, i);

                    match neighbors {
                        (_, Some(_), Some('o')) => new_row.push('('),
                        (Some('o'), Some(_), _) => new_row.push(')'),
                        (_, Some('('), _) | (_, Some('o'), _) | (_, Some(')'), _) => {
                            new_row.push(original_field[row.0][i])
                        }
                        _ => new_row.push(*e),
                    }
                }

                new_row
            })
            .collect();

        // Add drops
        let to_modify: Vec<(usize, usize)> = get_available_cells(&display_field)
            .choose_multiple(&mut rng, 10)
            .cloned()
            .collect();

        for e in to_modify.iter() {
            display_field[e.1][e.0] = 'o'
        }

        let text: Vec<Line> = display_field
            .iter()
            .map(|row| Line::from(String::from_iter(row)))
            .collect();

        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(Paragraph::new(text).white().on_black(), area);
        })?;

        // Update work field with display field
        work_field = display_field;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        thread::sleep(delay);
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn get_available_cells(field: &Vec<Vec<char>>) -> Vec<(usize, usize)> {
    let mut available_cells: Vec<(usize, usize)> = Vec::new();

    for (y, row) in field.iter().enumerate() {
        for (x, e) in row.iter().enumerate() {
            match e {
                '_' | ',' => available_cells.push((x, y)),
                _ => (),
            }
        }
    }

    available_cells
}

fn get_neighbors(field: &Vec<char>, index: usize) -> (Option<char>, Option<char>, Option<char>) {
    match index {
        0 => (None, Some(field[index]), Some(field[index + 1])),
        x if x == field.len() - 1 => (Some(field[index - 1]), Some(field[index]), None),
        _ => (
            Some(field[index - 1]),
            Some(field[index]),
            Some(field[index + 1]),
        ),
    }
}
