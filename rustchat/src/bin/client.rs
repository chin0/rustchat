use std::error::Error;
use std::io;
use std::io::Bytes;
use std::io::Read;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::process::exit;
use std::thread::spawn;
use std::sync::Arc;
use parking_lot::Mutex;
use unicode_width::UnicodeWidthStr;

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use rustchat::ui::events::{Event, Events};
use rustchat::data::Message;
use rustchat::protocol::Framing;
use rustchat::protocol::Command;
use rustchat::user::User;

fn msgread_thread(mut stream: Bytes<TcpStream>, app: Arc<Mutex<App>>) {
    loop {
        let data = Message::decode(&mut stream).unwrap();
        let mut app = app.lock();
        app.messages.push(data);
    }
}

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    username: String,
    messages: Vec<Message>,
    network: TcpStream
}

//1. UI 분리.
fn start_ui(app: Arc<Mutex<App>>) -> Result<(), Box<dyn Error>>{
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let mut events = Events::new();
    let mut input_buf = String::new();
    let mut input_mode = InputMode::Normal;

    // Create default app state
    loop {
        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let (msg, style) = match input_mode {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to start editing."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to record the message"),
                    ],
                    Style::default(),
                ),
            };
            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);
            f.render_widget(help_message, chunks[0]);

            let input = Paragraph::new(input_buf.as_ref())
                .style(match input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[1]);
            match input_mode {
                InputMode::Normal =>
                    // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                    {}

                InputMode::Editing => {
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        chunks[1].x + input_buf.width() as u16 + 1,
                        // Move one line down, from the border to the input line
                        chunks[1].y + 1,
                    )
                }
            }

            let messages: Vec<ListItem> = app.lock()
                .messages
                .iter()
                .enumerate()
                .map(|(_, m)| {
                    let content = vec![Spans::from(Span::raw(m.to_str()))];
                    ListItem::new(content)
                })
                .collect();
            let messages =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
            f.render_widget(messages, chunks[2]);
        })?;

        // Handle input
        if let Event::Input(input) = events.next()? {
            match input_mode {
                InputMode::Normal => match input {
                    Key::Char('e') => {
                        input_mode = InputMode::Editing;
                        events.disable_exit_key();
                    }
                    Key::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                InputMode::Editing => match input {
                    Key::Char('\n') => {
                        let buf = input_buf.drain(..).collect::<String>();
                        let msg = Message::new(&app.lock().username, &buf);
                        app.lock().network.write(&Command::Message(msg).encode_data())?;
                        app.lock().network.flush()?;
                    }
                    Key::Char(c) => {
                        input_buf.push(c);
                    }
                    Key::Backspace => {
                        input_buf.pop();
                    }
                    Key::Esc => {
                        input_mode = InputMode::Normal;
                        events.enable_exit_key();
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>>{
    let stdin = stdin();
    let arguments : Vec<String> = std::env::args().collect();

    if arguments.len() < 3 {
        eprintln!("usage: {} ip port", arguments[0]);
        exit(1);
    }

    let ip : Ipv4Addr = arguments[1].parse()
        .expect("not a ip address!");

    let port: u16 = arguments[2].parse()
        .expect("not a port number!");

    
    let mut conn = TcpStream::connect(SocketAddr::from((ip, port)))?;
    
    let mut username = String::new();
    let mut password = String::new();
    print!("username: ");
    stdout().flush()?;
    stdin.read_line(&mut username)?;
    print!("password: ");
    stdout().flush()?;
    stdin.read_line(&mut password)?;

    let request = Command::Login(User::new(&username, &password));
    conn.write(&request.encode_data())?;

    let app = Arc::new(Mutex::new(App {
        username,
        messages: Vec::new(),
        network: conn.try_clone().unwrap()
    }));

    let reader = conn.try_clone().unwrap().bytes();
    let app_ref = Arc::clone(&app);
    spawn(|| {
        //reader를 빼고 app의 network에서 가져오는쪽으로 할까..
        msgread_thread(reader, app_ref);
    });

    start_ui(Arc::clone(&app))?;
    Ok(())
}