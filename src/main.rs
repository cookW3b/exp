use std::{fs, io::{self, Stdout, Write}, path::PathBuf, collections::HashMap, borrow::Borrow};
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
                Key::Char('q')  => {
                    println!("{}{}\r", termion::cursor::Goto(1, 1), termion::clear::All);
                    break
                },
                Key::Char('k')  => self.dec_cursor_y(),
                Key::Char('j')  => self.inc_cursor_y(),
                Key::Char('g')  => self.go_up(),
                Key::Char('d')  => self.go_down(),
                Key::Char('\n') => self.go_inside_dir(),
                Key::Char('r')  => self.rename_file(),
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
                path: e.path()
            });
            self.list_length += 1;
        }

        self.entries.sort_by_key(|a| !a.path.is_dir());
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
            if curr_entry.path.is_dir() {
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
            if (self.cursor.y > termion::terminal_size().unwrap().1 - 4) &&
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
        let curr_file = self.get_curr_file();
        if curr_file.path.is_dir() {
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

    fn rename_file(&mut self) {
        let mut input_window = InputWindow::new();

        input_window.set_content(self.get_curr_filename());
        input_window.render();

        for key in std::io::stdin().keys() {
            match key.unwrap() {
                Key::Backspace => input_window.remove_char(),
                Key::Ctrl('c') => return (),
                Key::Left => input_window.move_cursor_left(),
                Key::Right => input_window.move_cursor_right(),
                Key::Char('\n') => {
                    if input_window.content.len() > 0 {
                        break;
                    } else {
                        continue;
                    }
                },
                Key::Char(c) => input_window.add_char(c),
                _ => ()
            }

            self.print_entries_list();
            input_window.render();
        }

        let curr_file = self.get_curr_file();
        let before_rename = curr_file.path.as_path().to_owned();
        curr_file.name = input_window.content.clone();
        curr_file.path.set_file_name(input_window.content);
        fs::rename(before_rename, curr_file.path.as_path()).unwrap();
    }

    fn go_down(&mut self) {
        self.cursor.y = self.outputed_count as u16;
        println!(
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        );
    }

    fn get_curr_file(&mut self) -> &mut File {
        self.entries.get_mut(self.cursor.y as usize - 1).unwrap()
    }

    fn get_curr_filename(&mut self) -> String {
        self.entries.get(self.cursor.y as usize - 1).unwrap().name.to_string()
    }
}

struct InputWindow {
    size: (u16, u16),
    pos: (u16, u16),
    title: String,
    scroll_x: u16,
    cursor_x: u16,
    content: String,
}

impl InputWindow {
    fn new() -> InputWindow {
        let size = termion::terminal_size().unwrap();

        InputWindow {
            size: (size.0 / 2, size.1 / 2),
            pos: (size.0 / 4, size.1 / 2),
            title: String::from("Input Window"),
            scroll_x: 0,
            cursor_x: 0,
            content: String::new(),
        }
    }

    fn add_char(&mut self, new_char: char) {
        if self.cursor_x == self.content.chars().count() as u16 + self.pos.0 + 1 {
            self.content.push(new_char);
        } else {
            let mut result = String::new();
            for (i, c) in self.content.chars().enumerate() {
                if i == (self.cursor_x - self.pos.0) as usize - 1 {
                    result.push(new_char);
                }
                result.push(c);
            }
            self.content = result;
        }

        self.move_cursor_right();
    }

    fn remove_char(&mut self) {
        if self.content.len() == 0 { return };
        let mut result = String::new();

        for (i, c) in self.content.chars().enumerate() {
            if i + 1 != (self.cursor_x - self.pos.0) as usize - 1 {
                result.push(c)
            }
        }

        self.content = result;
        self.move_cursor_left();
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > self.pos.0 + 1 {
            self.cursor_x -= 1;
        }

        println!("{}", termion::cursor::Goto(self.cursor_x, self.pos.1 + 1))
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_x < self.content.chars().count() as u16 + self.pos.0 + 1 {
            self.cursor_x += 1;
        }

        println!("{}", termion::cursor::Goto(self.cursor_x, self.pos.1 + 1))
    }

    fn render(&self) {
        let mut line = String::new();

        for _ in 0..self.size.0 {
            line.push('─')
        }

        println!("{}", termion::cursor::Goto(self.pos.0, self.pos.1));
        println!("{}{}", termion::cursor::Up(1), self.title);
        println!("{}", termion::cursor::Goto(self.pos.0, self.pos.1));
        println!("╭{}╮", line);
        println!("{}", termion::cursor::Goto(self.pos.0, self.pos.1 + 2));
        println!("╰{}╯", line);
        println!("{}", termion::cursor::Goto(self.pos.0, self.pos.1 + 1));
        println!("│");
        println!("{}", termion::cursor::Goto(self.size.0 + self.pos.0 + 1, self.pos.1 + 1));
        println!("│");
        println!("{}", termion::cursor::Goto(self.pos.0 + 1, self.pos.1 + 1));

        println!("{}", self.content);
        println!("{}", termion::cursor::Goto(
                self.cursor_x,
                self.pos.1 + 1));
    }

    fn set_size(&mut self, width: u16, height: u16) {
        self.size = (width, height);
    }

    fn set_position(&mut self, x: u16, y: u16) {
        self.pos = (x, y);
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn set_content(&mut self, content: String) {
        self.content = content.clone();
        self.cursor_x = self.pos.0 + content.chars().count() as u16;
    }
}