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
            path: PathBuf::from("./test_dir"),
            outputed_count: 0,
            scroll_y: 0,
        }
    }

    fn run(&mut self) {
        self.read_dir();
        self.update_entry_list();

        let stdin = io::stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q')  => break,
                Key::Char('k')  => self.dec_cursor_y(),
                Key::Char('j')  => self.inc_cursor_y(),
                Key::Char('g')  => self.go_up(),
                Key::Char('d')  => self.go_down(),
                Key::Char('\n') => {
                    self.go_inside_dir();
                    self.read_dir();
                },
                Key::Backspace  => {
                    self.go_back();
                    self.read_dir();
                },
                _ => ()
            }

            self.update_entry_list();

            self.stdout.flush().unwrap();
        }
    }

    fn read_dir(&mut self) {
        print!("{}{}\r", termion::clear::All, termion::cursor::Goto(1, 1));

        self.list_length = 0;

        self.scroll_y = 0;

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

    fn update_entry_list(&mut self) {
        let (_, size_y) = termion::terminal_size().unwrap();

        print!("{}{}\r",
               termion::clear::All,
               termion::cursor::Goto(1, 0));

        println!("{}{}{}\r",
                 color::Bg(color::Black),
                 fs::canonicalize(PathBuf::from(&self.path)).unwrap().to_string_lossy(),
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
            if self.scroll_y < self.list_length as u16 % termion::terminal_size().unwrap().1 + 2  {
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
        self.update_entry_list();
        self.cursor.y = 1;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }

    fn go_inside_dir(&mut self) {
        let curr_file = self.entries.get(self.cursor.y as usize - 1).unwrap();
        if curr_file.is_dir {
            self.path = curr_file.path.clone();
        }
    }

    fn go_back(&mut self) {
        let path = self.path.to_str().unwrap();
        let mut slash_index = 0;

        for i in 0..self.path.to_str().unwrap().len() {
            if path.as_bytes()[i] as char == '/' {
                slash_index = i;
            }
        }

        self.path = PathBuf::from(path[..slash_index].to_owned())
    }

    fn get_cursor_filename(&self) {

    }

    fn go_down(&mut self) {
        self.cursor.y = self.outputed_count as u16;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }
}
