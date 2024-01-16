use std::{fs, io::{self, Stdout, Write}, path::PathBuf};
use termion::{raw::{IntoRawMode, RawTerminal}, color::{self}};
use termion::input::TermRead;
use termion::event::Key;

fn main() {
    let mut explorer = Explorer::new();
    explorer.run()
}

struct Point {
    x: u16,
    y: u16
}

#[derive(Debug)]
struct File {
    name: String,
    is_dir: bool,
    path: PathBuf
}

struct Explorer {
    stdout: RawTerminal<Stdout>,
    folder_fg: color::Fg<color::Blue>,
    scroll_y: u16,
    cursor: Point,
    list_length: usize,
    entries: Vec<File>,
    outputed_count: u16,
    path: PathBuf,
}

impl Explorer {
    fn new() -> Explorer {
        Explorer {
            stdout: io::stdout().into_raw_mode().unwrap(),
            folder_fg: color::Fg(color::Blue),
            cursor: Point { x: 1, y: 1 },
            list_length: 0,
            entries: Vec::new(),
            path: fs::canonicalize(PathBuf::from("./")).unwrap(),
            outputed_count: 0,
            scroll_y: 0,
        }
    }

    fn run(&mut self) {
        self.read_dir();
        self.print_entries_list();

        let stdin = io::stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q')  => break,
                Key::Char('k')  => self.dec_cursor_y(),
                Key::Char('j')  => self.inc_cursor_y(),
                Key::Char('g')  => self.go_up(),
                Key::Char('d')  => self.go_down(),
                Key::Char('\n') => self.go_inside_dir(),
                Key::Backspace  => self.go_back(),
                _ => ()
            }

            self.print_entries_list();

            self.stdout.flush().unwrap();
        }
    }

    fn read_dir(&mut self) {
        print!("{}{}\r", termion::clear::All, termion::cursor::Goto(1, 1));

        self.list_length = 0;

        self.scroll_y = 0;
        self.cursor.y = 1;

        self.entries = Vec::new();

        for entry in fs::read_dir(&self.path).unwrap() {
            let e = entry.unwrap();
            let file_name = e.file_name().to_string_lossy().into_owned();
            self.entries.push(File {
                name: file_name,
                is_dir: e.path().is_dir(),
                path: e.path()
            });
            self.list_length += 1;
        }

        self.entries.sort_by_key(|a| !a.is_dir);
    }

    fn print_entries_list(&mut self) {
        let (_, size_y) = termion::terminal_size().unwrap();

        print!("{}{}\r",
               termion::clear::All,
               termion::cursor::Goto(1, 0));

        println!("{}{}{}\r",
                 color::Bg(color::Black),
                 fs::canonicalize(&self.path).unwrap().to_string_lossy(),
                 color::Bg(color::Reset),
        );

        self.outputed_count = 0;

        for i in self.scroll_y as usize..self.entries.len() {
            if i as i32 >= size_y as i32 + self.scroll_y as i32 - 2 {
                break;
            }
            let curr_entry = self.entries.get(i).unwrap();
            if curr_entry.is_dir {
                println!("{}{}{}\r",
                            self.folder_fg,
                            curr_entry.name,
                            color::Fg(color::Reset));
            } else {
                println!("{}\r", curr_entry.name);
            }
            self.outputed_count += 1;
        }

        println!("{}\r", termion::cursor::Goto(1, self.cursor.y));
        self.stdout.flush().unwrap();
    }

    fn dec_cursor_y(&mut self) {
        if self.cursor.y == 1 {
            if self.scroll_y > 0 {
                self.scroll_y -= 1;
            }
        }
        if self.cursor.y > 1 {
            self.cursor.y -= 1;
        }
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }

    fn inc_cursor_y(&mut self) {
        if self.cursor.y < self.outputed_count {
            self.cursor.y += 1;
        } else {
            if  (self.cursor.y > termion::terminal_size().unwrap().1 - 4) &&
                (self.scroll_y < self.list_length as u16 % termion::terminal_size().unwrap().1 + 2)
            {
                self.scroll_y += 1;
            }
        }
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }

    fn go_up(&mut self) {
        self.cursor.y = 0;
        self.print_entries_list();
        self.cursor.y = 1;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }

    fn go_inside_dir(&mut self) {
        let curr_file = self.entries.get(self.cursor.y as usize - 1).unwrap();
        if curr_file.is_dir {
            self.path = fs::canonicalize(curr_file.path.clone()).unwrap();
            self.read_dir();
        }
    }

    fn go_back(&mut self) {
        if let Some(path) = self.path.parent() {
            self.path = path.to_path_buf();
            self.read_dir();
        }
    }

    fn go_down(&mut self) {
        self.cursor.y = self.outputed_count as u16;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }
}
