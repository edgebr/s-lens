use std::io::Read;
/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages

/// serial uses
    //use std::sync::{Arc, Mutex};
    use std::thread;
    //use serialport::SerialPortType;
    //use std::io::{Read, Write};
    use std::sync::mpsc::channel;
    //use std::io;
    use std::time::{Duration};
    extern crate serialport;
    use std::sync::mpsc;
    use chrono::prelude::*;
/// Time uses
/// 
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    // let mut port = String::new();
    // println!("Type the port: ");
    // io::stdin().read_line(&mut port).expect("Falha ao ler entrada");
    // let mut frq = String::new();
    // println!("Type the frequency: ");
    // io::stdin().read_line(&mut frq).expect("Falha ao ler entrada");
    // let brate:u32 = frq.trim().parse().unwrap();
    // println!("porta: {}  frq: {}",port,frq);
    let mut serial1 = serialport::new("COM5", 115200).timeout(Duration::from_millis(100)).open().unwrap();
    let mut serial2 = serial1.try_clone().expect("Failed to clone");
    let (tx_write_thread, rx_write_thread) = channel();
    let (tx_read_thread, rx_read_thread) = channel();
    let (tx_time, rx_time) = channel();
    thread::spawn(move || {
        println!("thread 1 running");
        loop {
            let mut data_to_send:String = rx_write_thread.recv().unwrap();
            //println!("<{}>",data_to_send);
            data_to_send.push('\r');
            data_to_send.push('\n');
            serial1.write_all(data_to_send.as_bytes()).expect("Failed to write to serial port");
        }
    });

    thread::spawn(move || {
        
        println!("thread 2 running");
        loop {
            let mut buffer: [u8; 1] = [0; 1];
            let mut out:String = "".to_string();
            let mut x = 0;
            for _i in 0..1024{
                match serial2.read(&mut buffer) {
                    Ok(bytes) => {
                        let curr = buffer[0];
                        //let breakline = \n;
                        if bytes == 1{
                            if x == 2{
                                out.push(curr as char);
                                //tx_read_thread.send(out).expect("Send failed!");
                                break;
                            } 
                            if curr == 91{
                                out.push(curr as char);
                                x = x + 1;
                            }else if curr == 10 {
                                continue;
                            }else{
                                out.push(curr as char);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            tx_read_thread.send(out).expect("Send failed!");
            let dt: DateTime<Local> = Local::now();
            let currtime = format!("({}:{}:{})",dt.hour(),dt.minute(),dt.second());
            //let s:String = currtime[11..19];
            //println!("enviei");
            tx_time.send(currtime).expect("Send time failed!");
            //tx_read_thread.send(buffer).expect("Send failed!");
            //thread::sleep(Duration::from_millis(100));
        }
    });

    // loop{
    //     let mut message = String::new();
    //     io::stdin().read_line(&mut message).expect("Falha ao ler entrada");
    //     let a = message;
    //     tx_write_thread.send(a).expect("Send failed!");
    //     //thread::sleep(Duration::from_millis(100));
    //     let data_to_show = rx_read_thread.recv().unwrap();
    //     println!("{}", data_to_show);
    // }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tx_write_thread2 = tx_write_thread.clone();
    //let rx_read_thread2 = rx_read_thread.clone();
    let app = App::default();
    let res = run_app(&mut terminal, app,tx_write_thread2,rx_read_thread,rx_time);

    // restore terminalS
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, tx_write_thread: mpsc::Sender<String>, rx_read_thread: mpsc::Receiver<String>,rx_time: mpsc::Receiver<String>) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        tx_write_thread.send(app.input.drain(..).collect()).expect("Send failed!");
                        let data_to_show = rx_read_thread.recv().unwrap();
                        let currtime = rx_time.recv().unwrap();
                        let together = format!("{} - {}",currtime, data_to_show);
                        app.messages.push(together);
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(_i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}", m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[1]);

    let (msg, style) = match app.input_mode {
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

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[2].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[2].y + 1,
            )
        }
    }
}
