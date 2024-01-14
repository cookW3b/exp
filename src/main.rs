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

struct Explorer {
    stdout: RawTerminal<Stdout>,
    folder_fg: color::Fg<color::Blue>,
    scroll_y: u16,
    cursor: Point,
    list_length: usize,
    dirs: Option<Vec<String>>,
    files: Option<Vec<String>>,
    outputed_count: u16,
    path: String
}

impl Explorer {
    fn new() -> Explorer {
        Explorer {
            stdout: io::stdout().into_raw_mode().unwrap(),
            folder_fg: color::Fg(color::Blue),
            cursor: Point { x: 1, y: 0 },
            list_length: 0,
            dirs: None,
            files: None,
            path: "/home/artem/".to_string(),
            outputed_count: 0,
            scroll_y: 0
        }
    }

    fn run(&mut self) {
        self.read_dir();
        self.update_entry_list();

        let stdin = io::stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char('k') => self.dec_cursor_y(),
                Key::Char('j') => self.inc_cursor_y(),
                Key::Char('g') => self.go_up(),
                Key::Char('d') => self.go_down(),
                _ => ()
            }

            self.update_entry_list();

            self.stdout.flush().unwrap()
        }
    }

    fn read_dir(&mut self) {
        print!("{}{}\r", termion::clear::All, termion::cursor::Goto(1, 1));

        self.list_length = 0;

        let mut files = Vec::new();
        let mut dirs = Vec::new();
        for entry in fs::read_dir(&self.path).unwrap() {
            let e = entry.unwrap();
            let file_name = e.file_name().to_string_lossy().into_owned();
            if e.path().is_dir() {
                dirs.push(file_name);
            } else {
                files.push(file_name);
            }
            self.list_length += 1;
        }

        dirs.sort();
        files.sort();

        self.dirs = Some(dirs);
        self.files = Some(files);
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

        let dirs_len = self.dirs.as_ref().unwrap().len();
        let files_len = self.files.as_ref().unwrap().len();
        let entries_len = dirs_len + files_len;

        self.outputed_count = 0;

        for i in self.scroll_y as usize..entries_len {
            if i as i32 >= size_y as i32 + self.scroll_y as i32 - 2 {
                break;
            }
            if i < entries_len - files_len {
                println!("{}{}{}\r",
                         self.folder_fg,
                         self.dirs.as_ref().unwrap().get(i).unwrap(),
                         color::Fg(color::Reset));
            } else {
                // println!("{} of {} | {}\r", i, files_len, entries_len);
                println!("{}\r", self.files.as_ref().unwrap().get(i % files_len).unwrap());
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
            self.scroll_y += 1;
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

    fn go_down(&mut self) {
        self.cursor.y = self.outputed_count as u16;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }
}
