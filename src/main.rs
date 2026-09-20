#![windows_subsystem = "windows"]

use iced::font::Family;
use iced::keyboard::{self, Key, key::Named};
use iced::mouse::Cursor;
use iced::theme::{Custom, Palette};
use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke, Text};
use iced::widget::{column, container, mouse_area, row, slider, text};
use iced::{
    Alignment, Color, Element, Event, Font, Length, Point, Rectangle, Renderer, Size, Subscription,
    Task, Theme, window,
};
use std::sync::Arc;
use std::time::Instant;

const LEXEND: Font = Font {
    family: Family::Name("Lexend"),
    ..Font::DEFAULT
};

#[derive(Default)]
struct State {
    tap: f32,
    dot_thresh: f32,
    seq_thresh: f32,
    states: [bool; 26],
    unknown_state: bool,
    is_pressed: bool,

    // Timing & Morse Sequence Tracking
    press_start: Option<Instant>,
    last_release: Option<Instant>,
    current_sequence: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputPressed,
    InputReleased,
    DotSlider(f32),
    SeqSlider(f32),
    EventOccurred(Event),
}

#[derive(Debug)]
struct Diagram {
    states: [bool; 26],
    unknown_state: bool,
}

pub fn main() -> iced::Result {
    iced::application(new, update, view)
        .theme(theme)
        .font(include_bytes!("lexend.ttf"))
        .default_font(LEXEND)
        .subscription(subscription)
        .window(window::Settings {
            maximized: true,
            ..Default::default()
        })
        .run()
}

fn new() -> State {
    State {
        tap: 0.0,
        dot_thresh: 0.1,
        seq_thresh: 1.0,
        states: [false; 26],
        unknown_state: false,
        is_pressed: false,
        press_start: None,
        last_release: None,
        current_sequence: String::new(),
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    iced::event::listen().map(Message::EventOccurred)
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::InputPressed => {
            if !state.is_pressed {
                state.is_pressed = true;
                let now = Instant::now();

                // Check time gap since last release: if >= sequence threshold, reset sequence
                if let Some(last_rel) = state.last_release {
                    if now.duration_since(last_rel).as_secs_f32() >= state.seq_thresh {
                        state.current_sequence.clear();
                    }
                }

                state.press_start = Some(now);
                update_highlights(state);
            }
        }
        Message::InputReleased => {
            if state.is_pressed {
                state.is_pressed = false;
                let now = Instant::now();

                // Determine if press duration was dot or dash
                if let Some(start) = state.press_start.take() {
                    let duration = now.duration_since(start).as_secs_f32();

                    if duration < state.dot_thresh {
                        state.current_sequence.push('.');
                    } else {
                        state.current_sequence.push('-');
                    }
                }

                state.last_release = Some(now);
                state.tap += 1.0;
                update_highlights(state);
            }
        }
        Message::DotSlider(thresh) => state.dot_thresh = thresh,
        Message::SeqSlider(thresh) => state.seq_thresh = thresh,

        // Global Spacebar handling
        Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Space),
            ..
        })) => {
            return update(state, Message::InputPressed);
        }
        Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased {
            key: Key::Named(Named::Space),
            ..
        })) => {
            return update(state, Message::InputReleased);
        }

        // Global mouse release fallback
        Message::EventOccurred(Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        ))) => {
            if state.is_pressed {
                return update(state, Message::InputReleased);
            }
        }

        Message::EventOccurred(_) => {}
    }
    Task::none()
}

fn update_highlights(state: &mut State) {
    state.states = [false; 26];
    state.unknown_state = false;

    if state.current_sequence.is_empty() {
        return;
    }

    let current = decode_morse(&state.current_sequence);
    if current == "?" {
        state.unknown_state = true;
    } else {
        if let Some(ch) = current.chars().next() {
            let idx = (ch as u8 - b'A') as usize;
            if idx < 26 {
                state.states[idx] = true;
            }
        }
    }
}

fn decode_morse(seq: &str) -> String {
    let letter = match seq {
        ".-" => "A",
        "-..." => "B",
        "-.-." => "C",
        "-.." => "D",
        "." => "E",
        "..-." => "F",
        "--." => "G",
        "...." => "H",
        ".." => "I",
        ".---" => "J",
        "-.-" => "K",
        ".-.." => "L",
        "--" => "M",
        "-." => "N",
        "---" => "O",
        ".--." => "P",
        "--.-" => "Q",
        ".-." => "R",
        "..." => "S",
        "-" => "T",
        "..-" => "U",
        "...-" => "V",
        ".--" => "W",
        "-..-" => "X",
        "-.--" => "Y",
        "--.." => "Z",
        _ => "?",
    };
    letter.to_string()
}

fn theme(_state: &State) -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "Dark Mint".to_string(),
        Palette {
            background: Color::from_rgb8(0, 0, 0),
            text: Color::from_rgb8(255, 255, 255),
            primary: Color::from_rgb8(0, 255, 175),
            success: Color::from_rgb8(50, 200, 50),
            danger: Color::from_rgb8(200, 50, 50),
            warning: Color::from_rgb8(255, 180, 50),
        },
    )))
}

fn view(state: &State) -> Element<'_, Message> {
    let neon = Color::from_rgb8(0, 255, 175);

    column![
        // diagram
        Canvas::new(Diagram {
            states: state.states,
            unknown_state: state.unknown_state,
        })
        .width(Length::Fill)
        .height(Length::Fill),
        // Live Morse Output Status (Centered via container)
        container(text(&state.current_sequence).size(48))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(10),
        // threshold sliders
        row![
            column![
                text(format!("Dot Threshold: {:.1}s", state.dot_thresh)).size(30),
                slider(0.1..=1.0, state.dot_thresh, Message::DotSlider)
                    .step(0.1_f32)
                    .style(move |theme, status| {
                        let mut style = slider::default(theme, status);
                        style.rail.backgrounds = (neon.into(), Color::WHITE.into());
                        style
                    })
            ]
            .align_x(Alignment::Center)
            .spacing(10)
            .width(Length::FillPortion(1)),
            column![
                text(format!("Sequence Threshold: {:.1}s", state.seq_thresh)).size(30),
                slider(0.1..=4.0, state.seq_thresh, Message::SeqSlider)
                    .step(0.1_f32)
                    .style(move |theme, status| {
                        let mut style = slider::default(theme, status);
                        style.rail.backgrounds = (neon.into(), Color::WHITE.into());
                        style
                    })
            ]
            .align_x(Alignment::Center)
            .spacing(10)
            .width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .padding(20),
        // tap button with custom styling (#00ffaf)
        row![
            mouse_area(
                container(text("TAP / SPACE").size(48).color(Color::BLACK))
                    .width(Length::Fill)
                    .padding(20)
                    .align_x(Alignment::Center)
                    .style(move |_theme| {
                        let mut style = container::Style::default();
                        style.background = Some(neon.into());
                        style.border.radius = 4.0.into();
                        style
                    })
            )
            .on_press(Message::InputPressed)
            .on_release(Message::InputReleased)
        ],
    ]
    .align_x(Alignment::Center)
    .into()
}

impl<Message> Program<Message> for Diagram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let neon = Color::from_rgb8(0, 255, 175);

        let col_w = bounds.width / 8.0;
        let row_h = bounds.height / 8.0;

        let points = [
            Point::new(col_w * 4.0, row_h * 4.0), // a
            Point::new(col_w * 3.0, row_h * 7.0), // b
            Point::new(col_w * 2.0, row_h * 5.0), // c
            Point::new(col_w * 3.0, row_h * 6.0), // d
            Point::new(col_w * 4.0, row_h),       // e
            Point::new(col_w * 5.0, row_h * 3.0), // f
            Point::new(col_w * 2.0, row_h * 2.0), // g
            Point::new(col_w * 7.0, row_h),       // h
            Point::new(col_w * 5.0, row_h),       // i
            Point::new(col_w * 4.0, row_h * 7.0), // j
            Point::new(col_w * 2.0, row_h * 4.0), // k
            Point::new(col_w * 6.0, row_h * 4.0), // l
            Point::new(col_w * 2.0, row_h),       // m
            Point::new(col_w * 3.0, row_h * 4.0), // n
            Point::new(col_w, row_h),             // o
            Point::new(col_w * 5.0, row_h * 6.0), // p
            Point::new(col_w, row_h * 2.0),       // q
            Point::new(col_w * 5.0, row_h * 4.0), // r
            Point::new(col_w * 6.0, row_h),       // s
            Point::new(col_w * 3.0, row_h),       // t
            Point::new(col_w * 5.0, row_h * 2.0), // u
            Point::new(col_w * 6.0, row_h * 2.0), // v
            Point::new(col_w * 4.0, row_h * 6.0), // w
            Point::new(col_w * 2.0, row_h * 6.0), // x
            Point::new(col_w, row_h * 4.0),       // y
            Point::new(col_w * 2.0, row_h * 3.0), // z
        ];

        let dots = [
            false, // a
            true,  // b
            true,  // c
            true,  // d
            true,  // e
            true,  // f
            true,  // g
            true,  // h
            true,  // i
            false, // j
            false, // k
            true,  // l
            false, // m
            true,  // n
            false, // o
            true,  // p
            false, // q
            true,  // r
            true,  // s
            false, // t
            false, // u
            false, // v
            false, // w
            false, // x
            false, // y
            true,  // z
        ];

        // Lines
        let connects = [
            (14, 7),  // o - h
            (24, 13), // y - n
            (0, 11),  // a - l
            (16, 6),  // q - g
            (23, 3),  // x - d
            (22, 15), // w - p
            (12, 25), // m - z
            (19, 1),  // t - b
            (4, 9),   // e - j
            (10, 2),  // k - c
            (8, 5),   // i - f
            (18, 21), // s - v
        ];

        for connect in connects {
            let line = Path::line(points[connect.0], points[connect.1]);
            frame.stroke(
                &line,
                Stroke::default().with_color(Color::WHITE).with_width(8.0),
            );
        }

        // Start line
        let line = Path::line(Point::new(col_w * 3.5, 0.0), Point::new(col_w * 3.5, row_h));
        frame.stroke(
            &line,
            Stroke::default().with_color(Color::WHITE).with_width(8.0),
        );

        // Unknown '?' Diamond Node in Bottom-Right corner
        let unknown_pt = Point::new(col_w * 7.5, row_h * 7.0);
        let diamond_size = 40.0;
        let diamond_path = Path::new(|builder| {
            builder.move_to(Point::new(unknown_pt.x, unknown_pt.y - diamond_size));
            builder.line_to(Point::new(unknown_pt.x + diamond_size, unknown_pt.y));
            builder.line_to(Point::new(unknown_pt.x, unknown_pt.y + diamond_size));
            builder.line_to(Point::new(unknown_pt.x - diamond_size, unknown_pt.y));
            builder.close();
        });

        if self.unknown_state {
            frame.fill(&diamond_path, neon);
            frame.stroke(
                &diamond_path,
                Stroke::default().with_color(neon).with_width(8.0),
            );
        } else {
            frame.fill(&diamond_path, Color::BLACK);
            frame.stroke(
                &diamond_path,
                Stroke::default().with_color(neon).with_width(8.0),
            );
        }

        frame.fill_text(Text {
            content: "?".to_string(),
            position: unknown_pt,
            color: if self.unknown_state {
                Color::BLACK
            } else {
                neon
            },
            size: 50.0.into(),
            font: LEXEND,
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });

        // Dots and dashes
        for i in 0..26 {
            if dots[i] {
                // dots
                let dot = Path::circle(points[i], 40.0);

                if self.states[i] {
                    frame.fill(&dot, neon);
                    frame.stroke(&dot, Stroke::default().with_color(neon).with_width(8.0));
                } else {
                    frame.fill(&dot, Color::BLACK);
                    frame.stroke(&dot, Stroke::default().with_color(neon).with_width(8.0));
                }
            } else {
                // dashes
                let dash = Path::rectangle(
                    Point::new(points[i].x - 40.0, points[i].y - 40.0),
                    Size::new(80.0, 80.0),
                );

                if self.states[i] {
                    frame.fill(&dash, neon);
                    frame.stroke(&dash, Stroke::default().with_color(neon).with_width(8.0));
                } else {
                    frame.fill(&dash, Color::BLACK);
                    frame.stroke(&dash, Stroke::default().with_color(neon).with_width(8.0));
                }
            }

            // Labels
            let letter = (b'A' + i as u8) as char;

            frame.fill_text(Text {
                content: letter.to_string(),
                position: points[i],
                color: if self.states[i] { Color::BLACK } else { neon },
                size: 60.0.into(),
                font: LEXEND,
                align_x: text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}
