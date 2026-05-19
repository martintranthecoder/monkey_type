use crate::config::{self, Config, TestMode};
use crate::game::{self, CharResult, GameState};
use crate::stats::TestResult;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io;
use std::time::{Duration, Instant};

pub enum Outcome {
    Completed(TestResult),
    Quit,
}

const MONKEY_ART: &[&str] = &[
    r#"                        .="=."#,
    r#"                      _/.-.-.\_     _"#,
    r#"                     ( ( o o ) )    ))"#,
    r#"                      |/  "  \|    //"#,
    r#"      .-------.        \'---'/    //"#,
    r#"     _|~~ ~~  |_       /`"""`\\  (("#,
    r#"   =(_|_______|_)=    / /_,_\ \\  \\"#,
    r#"     |:::::::::|      \_\\_'__/ \  ))"#,
    r#"     |:::::::[]|       /`  /`~\  |//"#,
    r#"     |o=======.|      /   /    \  /"#,
    r#"     `"""""""""`  ,--`,--'\/\    /"#,
    r#"                   '-- "--'  '--'"#,
];

fn monkey_lines() -> Vec<Line<'static>> {
    let style = Style::default().fg(Color::Rgb(220, 180, 130));
    MONKEY_ART
        .iter()
        .map(|line| Line::from(Span::styled(*line, style)))
        .collect()
}

fn tier(wpm: f64) -> (&'static str, Color) {
    match wpm {
        w if w >= 100.0 => ("BANANAS!! top-tier ape", Color::Yellow),
        w if w >= 70.0 => ("excellent -- silverback", Color::Green),
        w if w >= 50.0 => ("nice swing!", Color::Cyan),
        w if w >= 30.0 => ("keep climbing", Color::White),
        _ => ("more bananas needed", Color::DarkGray),
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut GameState,
    config: &Config,
) -> io::Result<Outcome> {
    loop {
        terminal.draw(|f| render(f, state))?;

        if game::is_finished(state) {
            if state.finished_at.is_none() {
                state.finished_at = Some(Instant::now());
            }
            let result = TestResult::from_game(state, config);
            return results_loop(terminal, &result).map(|_| Outcome::Completed(result));
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => return Ok(Outcome::Quit),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Outcome::Quit);
                    }
                    KeyCode::Tab => restart(state, config),
                    _ => game::handle_key(state, key),
                }
            }
        }
    }
}

fn restart(state: &mut GameState, config: &Config) {
    let count = match config.test_mode {
        TestMode::Timed(secs) => ((secs as usize) * 6).clamp(50, 2000),
        TestMode::WordCount(n) => n.max(1),
    };
    let duration = state.test_duration;
    let words = game::load_words_from_str(config::word_list_json(&config.word_list), count);
    *state = GameState::new(words, duration);
}

fn results_loop<B: Backend>(terminal: &mut Terminal<B>, result: &TestResult) -> io::Result<()> {
    loop {
        terminal.draw(|f| render_results(f, result))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code, modifiers, kind, ..
            }) = event::read()?
            {
                if kind == KeyEventKind::Release {
                    continue;
                }
                match code {
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Tab => return Ok(()),
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn render(frame: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Top bar: stylized stats
    let elapsed = state.elapsed().as_secs_f64();
    let progress: Vec<Span> = match state.test_duration {
        Some(d) => {
            let remaining = (d.as_secs_f64() - elapsed).max(0.0);
            vec![
                Span::raw("  time "),
                Span::styled(
                    format!("{remaining:>3.0}s"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]
        }
        None => vec![
            Span::raw("  words "),
            Span::styled(
                format!("{}/{}", state.current_word, state.words.len()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ],
    };
    let mut top_spans = progress;
    top_spans.extend([
        Span::raw("     wpm "),
        Span::styled(
            format!("{:>5.1}", game::calculate_wpm(state)),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("     acc "),
        Span::styled(
            format!("{:>5.1}%", game::calculate_accuracy(state)),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let top = Paragraph::new(Line::from(top_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                "  monkey_type  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(top, chunks[0]);

    // Word display
    let mut spans: Vec<Span> = Vec::new();
    for (wi, word) in state.words.iter().enumerate() {
        let chars: Vec<char> = word.chars().collect();
        let cursor_pos: Option<usize> = if wi == state.current_word {
            Some(state.input.chars().count())
        } else {
            None
        };

        for (ci, c) in chars.iter().enumerate() {
            let result = state.char_results[wi][ci];
            let mut style = match result {
                CharResult::Correct => Style::default().fg(Color::Green),
                CharResult::Incorrect => Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::UNDERLINED),
                CharResult::Untyped => Style::default().fg(Color::DarkGray),
            };
            if cursor_pos == Some(ci) {
                style = style
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD);
            }
            spans.push(Span::styled(c.to_string(), style));
        }

        let mut space_style = Style::default();
        if cursor_pos == Some(chars.len()) {
            space_style = space_style
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD);
        }
        spans.push(Span::styled(" ", space_style));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, chunks[1]);

    // Bottom help bar with banana flair
    let bottom = Paragraph::new(Line::from(vec![
        Span::styled("  (=)  ", Style::default().fg(Color::Yellow)),
        Span::styled("tab", Style::default().fg(Color::Yellow)),
        Span::styled(" restart   ", Style::default().fg(Color::DarkGray)),
        Span::styled("esc", Style::default().fg(Color::Yellow)),
        Span::styled(" quit", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(bottom, chunks[2]);
}

pub fn render_results(frame: &mut Frame, result: &TestResult) {
    let area = frame.area();

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            "  results  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(42), Constraint::Min(20)])
        .split(inner);

    // Left: pixel-block monkey
    let monkey = Paragraph::new(monkey_lines());
    frame.render_widget(monkey, cols[0]);

    // Right: tier headline + stats
    let (tier_msg, tier_color) = tier(result.wpm);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {tier_msg}"),
        Style::default()
            .fg(tier_color)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  wpm        "),
        Span::styled(
            format!("{:>6.1}", result.wpm),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  accuracy   "),
        Span::styled(
            format!("{:>6.1}%", result.accuracy),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  correct    "),
        Span::styled(
            format!("{:>6}", result.correct_chars),
            Style::default().fg(Color::Green),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  incorrect  "),
        Span::styled(
            format!("{:>6}", result.incorrect_chars),
            Style::default().fg(Color::Red),
        ),
    ]));
    lines.push(Line::from(format!(
        "  time       {:>6.1}s",
        result.duration_secs
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  enter/tab  new test     esc  quit",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, cols[1]);
}
