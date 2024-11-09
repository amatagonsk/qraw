use std::{
    fs::File,
    io::{stdout, Write},
    time::{Duration, Instant},
};

use color_eyre::{eyre::Ok, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyEventKind, MouseButton},
    ExecutableCommand,
};
use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, MouseEventKind},
    layout::{Position, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Points},
        Block, Widget,
    },
    DefaultTerminal, Frame,
};
use regex::Regex;

fn main() -> Result<()> {
    color_eyre::install()?;
    stdout().execute(EnableMouseCapture)?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    app_result
}

struct App {
    exit: bool,
    frame_width: u16,
    frame_height: u16,
    marker: Marker,
    points: Vec<Position>,
    is_drawing: bool,
}

impl App {
    const fn new() -> Self {
        Self {
            exit: false,
            frame_width: 0,
            frame_height: 0,
            marker: Marker::Block,
            points: vec![],
            is_drawing: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(16);
        let last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_press(key),
                    Event::Mouse(event) => self.handle_mouse_event(event),
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            // todo? check popup (not need?
            KeyCode::Char('s') => Self::save_text(self),
            KeyCode::Char('c') => self.paint_clear_all(),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, event: event::MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.is_drawing = true,
            MouseEventKind::Up(MouseButton::Left) => self.is_drawing = false,
            // paint
            MouseEventKind::Drag(MouseButton::Left) => {
                self.points.push(Position::new(event.column, event.row));
            }
            // erase
            MouseEventKind::Drag(MouseButton::Right) => {
                self.points
                    .retain(|&p| p != Position::new(event.column, event.row));
            }
            _ => {}
        }
    }

    fn save_text(&mut self) {
        let path = "./draw(qraw).txt";
        let mut file = File::create(path).unwrap();
        let mut str_line: String = String::new();

        // sort
        self.points.sort_by(|a, b| a.x.cmp(&b.x));
        self.points.sort_by(|a, b| a.y.cmp(&b.y));

        // rm dup
        self.points.dedup_by(|a, b| ((a.x == b.x) && (a.y == b.y)));

        // // // check
        // let mut _line: Vec<[u16; 2]> = vec![];
        // self.points.iter().for_each(|point| _line.push([point.x, point.y]));
        // // //

        // offset one title
        for h in 1..=self.frame_height {
            for w in 0..=self.frame_width {
                str_line.push(' ');

                for point in &self.points {
                    if point.x == w && point.y == h {
                        str_line.pop();
                        str_line.push('█');
                    }
                }

                if w == self.frame_width {
                    str_line.push('\n');
                }
            }
        }

        let re = Regex::new(r"\s+\n").unwrap();
        let str_line = re.replace_all(&str_line, "\n");

        file.write_all(format!("{}", str_line).as_bytes()).unwrap();
    }

    fn paint_clear_all(&mut self) {
        self.points = vec![]
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.frame_width = frame.area().width - 1;
        self.frame_height = frame.area().height - 1;
        frame.render_widget(self.draw_canvas(frame.area()), frame.area());
    }

    fn draw_canvas(&self, area: Rect) -> impl Widget + '_ {
        Canvas::default()
            .block(
                Block::new()
                    .title(" ↓↓ Draw here ↓↓ ")
                    .title(" exit: <q> or <Esc> "),
            )
            .marker(self.marker)
            .x_bounds([0.0, f64::from(area.width)])
            .y_bounds([0.0, f64::from(area.height)])
            .paint(move |ctx| {
                let points = self
                    .points
                    .iter()
                    .map(|p| {
                        (
                            f64::from(p.x) - f64::from(area.left()),
                            f64::from(area.bottom()) - f64::from(p.y),
                        )
                    })
                    .collect_vec();
                ctx.draw(&Points {
                    coords: &points,
                    color: Color::White,
                });
            })
    }
}
