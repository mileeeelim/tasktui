use core::panic;
use std::{default, fs::{self, read}, io::Error, vec};

use color_eyre::Result;
use itertools::WhileSome;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize,},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap
    },
    DefaultTerminal, Frame,
};
use serde::{Serialize, Deserialize};

const FG: Color = Color::Rgb(157, 146, 170);
const BG: Color = Color::Rgb(44, 29, 58);


fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::default().run(terminal);
    ratatui::restore();
    result
}

struct App {
    should_exit: bool,
    todo_list: TodoList,
    state: ListState,
    add_new_state: bool,
    input_box: InputBox,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoList {
    items: Vec<TodoItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoItem {
    todo: String,
    status: bool,
    info: String,
}

struct InputBox {
    todo: String,
    desc: String,
    active: ActiveTodo,
}

enum ActiveTodo {
    Todo,
    Desc,
}

impl InputBox {
    fn new() -> Self {
        Self {
            todo: String::default(),
            desc: String::default(),
            active: ActiveTodo::Todo,
        }
    }

    fn render_popup(& self, area: Rect, buf: &mut Buffer ) {
        // Paragraph::new(self.todo.as_str())
        //     .style(Style::default())
        //     .block(Block::bordered().title("Input"));
        Paragraph::new(self.todo.as_str())
            .style(match self.active {
                ActiveTodo::Todo => Style::default(),
                ActiveTodo::Desc => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input")).render(area, buf);
    }

    fn handle_input(&mut self, event: Event, mut input_box: InputBox) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => input_box.todo.push(c), // Add character to input
                KeyCode::Backspace => {
                    self.todo.pop(); // Remove last character
                }
                KeyCode::Enter => {
                    
                }, // Submit input
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
enum Status {
    Todo,
    Completed,
}

impl Default for App {
    fn default() -> Self {

        // Read the TOML file
        println!("{}", (fs::read_to_string("rsc/main.toml")).unwrap());

        //type TodoList = Vec<(bool, String, String)>; 

        let toml_str = match fs::read_to_string("rsc/main.toml") {
            Ok(content) => {
                content
            },
            Err(e) => {
                eprintln!("Failed to read TOML file: {}", e);
                "couldnt read toml".to_string()
            }
        };

        // Parse the TOML string into the Todos struct
        let todos: TodoList = match toml::from_str(&toml_str) {
            Ok(parsed) => {
                parsed
            },
            Err(_e) => TodoList::from_iter([
                ("Rewrite everything with Rust!".to_string(), false,  "I can't hold my inner voice. He tells me to rewrite the complete universe with Rust {e}".to_string()),
            ]),
        };

        let todos_as_tuples: Vec<(String, bool, String)> = todos
        .items.iter()
        .map(|item| {
            let todo = item.todo.clone();
            let status = item.status;
            let info = item.info.clone();
            (todo, status, info)
        })
        .collect();


        Self {
            should_exit: false,
            todo_list: TodoList::from_iter(todos_as_tuples),
            state: ListState::default(),
            add_new_state: false,
            input_box: InputBox::new(),
        }
    }
}

impl FromIterator<(String, bool, String)> for TodoList {
    fn from_iter<I: IntoIterator<Item = (String, bool, String)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(todo, status, info)| TodoItem::new(status, todo, info))
            .collect();
        Self { items}
    }
}

impl TodoList {
    fn from_items(items: Vec<TodoItem>) -> Self {
        Self { items,}
    }
}

impl TodoItem {
    fn new(status: bool, todo: String, info: String) -> Self {
        Self {
            status,
            todo: todo.to_string(),
            info: info.to_string(),
        }
    }
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.add_new_state {
            match key.code {
                KeyCode::Esc => {
                    self.add_new_state = false; // Close popup on ESC
                }
                KeyCode::Enter => {
                    self.add_new_state = false;
                    println!("User input: {}", self.input_box.todo);
                }
                KeyCode::Char(c) => self.input_box.todo.push(c),
                KeyCode::Backspace => {
                    self.input_box.todo.pop(); // Remove last character
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('a') => {
                self.add_new_state();
            },
            KeyCode::Char('l') | KeyCode::Right => {
                self.toggle_status();
            }
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }

    fn add_new_state(&mut self) {
        self.add_new_state = true;
    }

    fn handle_new_input(&mut self, key: KeyEvent) {
        match key.code {
            // KeyCode::Esc => {
            //     self.add_new_state = false; // Close popup on ESC
            // }
            // KeyCode::Enter => {
            //     self.add_new_state = !self.add_new_state;
            //     panic!("User input: {}", self.input_box.todo);
            // }
            // KeyCode::Char(c) => self.input_box.todo.push(c), // Add character to input
            // KeyCode::Backspace => {
            //     self.input_box.todo.pop(); // Remove last character
            // }
            _ => {
                //self.input_box.handle_input(Event::Key(key), self.input_box)
                // match key.code {
                    
                //     KeyCode::Enter => {
                //         self.add_new_state = !self.add_new_state;
                //         panic!("User input: {}", self.input_box.todo);
                //     }, // Submit input
                //     _ => {panic!("NO USER INPUT!")}
                // }
                
            }
        }
    }


    fn toggle_status(&mut self) {
        if let Some(i) = self.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                true => false,
                false => true,
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        if self.add_new_state {
            self.input_box.render_popup(popup_area, buf);
            self.input_box.active = ActiveTodo::Todo;
        }

    }
}

impl App {

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ratatui List Example")
            .bold()
            .fg(FG)
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
        .fg(FG)
        .centered()
        .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("TODO List"))
            .borders(Borders::ALL)
            .border_style(Style::new().fg(FG))
            .bg(BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .todo_list
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                ListItem::from(todo_item).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(FG)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.state.selected() {
            match self.todo_list.items[i].status {
                true => format!("✓ DONE: {}", self.todo_list.items[i].info),
                false => format!("☐ TODO: {}", self.todo_list.items[i].info),
            }
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("TODO Info").centered())
            .borders(Borders::ALL)
            .border_style(Style::new().fg(FG))
            .bg(BG);

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(FG)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        BG
    } else {
        BG
    }
}

impl From<&TodoItem> for ListItem<'_> {
    fn from(value: &TodoItem) -> Self {
        let line = match value.status {
            false => Line::styled(format!(" ☐ {}", value.todo), FG),
            true => {
                Line::styled(format!(" ✓ {}", value.todo), FG)
            }
        };
        ListItem::new(line)
    }
}