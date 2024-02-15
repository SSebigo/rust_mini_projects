use std::{
    io::{stdout, Result},
    iter, thread, time,
};

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
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
    let mut original_field: Vec<Vec<char>> = iter::repeat_with(|| {
        iter::repeat_with(|| if rng.gen_bool(1.0 / 3.0) { ',' } else { '_' })
            .take(FIELD_COLUMNS)
            .collect()
    })
    .take(FIELD_ROWS)
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
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(|(x, c)| match get_neighbors(row, x) {
                        (_, _, Some('o')) => Some('('),
                        (Some('o'), _, _) => Some(')'),
                        (_, '(', _) | (_, 'o', _) | (_, ')', _) => Some(original_field[y][x]),
                        _ => Some(*c),
                    })
                    .collect()
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
            if let event::Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char('q'),
                ..
            }) = event::read()?
            {
                break;
            }
        }

        thread::sleep(delay);
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn get_available_cells(field: &Vec<Vec<char>>) -> Vec<(usize, usize)> {
    field
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                // `move` captures parent and move them into current closure
                .filter_map(move |(x, &c)| (c == '_' || c == ',').then_some((x, y)))
        })
        .collect()
}

fn get_neighbors(field: &Vec<char>, index: usize) -> (Option<char>, char, Option<char>) {
    (
        (index != 0).then(|| field[index - 1]),
        field[index],
        (index != field.len() - 1).then(|| field[index + 1]),
    )
}
