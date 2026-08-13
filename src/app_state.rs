use std::sync::{Arc, Mutex};

use arboard::Clipboard;
use rand::random_range;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::STD_PW_CHARS;

#[derive(Clone)]
pub struct CouncilListItem {
    pub name: String,
    pub password: String,
    pub password_gen_rules: String,
    pub info: Vec<(String, String)>,
    pub selected_info: usize,
}

impl CouncilListItem {
    pub fn new(name: String, password: String) -> Self {
        Self {
            name,
            password_gen_rules: String::from_iter(STD_PW_CHARS),
            password,
            info: vec![],
            selected_info: 1,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub exit: bool,
    pub list_index: usize,
    pub items: Vec<CouncilListItem>,
    pub new_item_input: Option<CreateNewEntry>,
    pub confirm_deletion_menu: Option<usize>,
    pub clipboard: Arc<Mutex<Clipboard>>,
}

#[derive(Clone, Debug)]
pub enum CouncilItemSelect {
    Name,
    Password,
    First(usize),
    Second(usize),
}

#[derive(Clone)]
pub struct CreateNewEntry {
    pub item: CouncilListItem,
    pub selected: CouncilItemSelect,
    pub generate_phase: bool,
    pub password_gen_len: usize,
    pub password_gen_scroll: usize,
    pub cursor_text_pos: CursorTextPositions,
}

#[derive(Clone, Debug)]
pub struct CursorTextPositions {
    pub name: u16,
    pub password: u16,
    pub password_gen: u16,
    pub info: Vec<(u16, u16)>,
}

impl Default for CreateNewEntry {
    fn default() -> Self {
        Self {
            item: CouncilListItem::new(String::new(), String::new()),
            selected: CouncilItemSelect::Name,
            generate_phase: true,
            password_gen_len: 16,
            password_gen_scroll: 0,
            cursor_text_pos: CursorTextPositions {
                name: 0,
                password: 0,
                password_gen: 0,
                info: vec![],
            },
        }
    }
}

impl AppState {
    pub fn compute_key(&mut self, key_event: KeyEvent) {
        if let Some(delete_entry_index) = self.confirm_deletion_menu {
            match key_event.code {
                KeyCode::Char('n') => self.confirm_deletion_menu = None,
                KeyCode::Char('y') => {
                    self.items.remove(delete_entry_index);
                    self.confirm_deletion_menu = None
                }
                _ => (),
            }
        } else if let Some(create_new_entry) = self.new_item_input.as_mut() {
            create_new_entry.compute_key(key_event);
            if key_event.code == KeyCode::Esc && create_new_entry.item.name.is_empty() {
                self.new_item_input = None;
            } else if key_event.code == KeyCode::Enter && create_new_entry.is_ready() {
                create_new_entry.clean_up();
                self.items.push(create_new_entry.item.clone());
                self.new_item_input = None;
            }
        } else {
            match key_event.code {
                KeyCode::Char('N') => self.new_item_input = Some(CreateNewEntry::default()),
                KeyCode::Up => self.list_index = self.list_index.saturating_sub(1),
                KeyCode::Down => {
                    self.list_index = (self.list_index + 1).min(self.items.len().saturating_sub(1))
                }
                KeyCode::Tab => {
                    if let Some(council_item) = self.items.get_mut(self.list_index) {
                        council_item.selected_info =
                            (council_item.selected_info + 1).min(council_item.info.len() + 1)
                    }
                }
                KeyCode::BackTab => {
                    if let Some(council_item) = self.items.get_mut(self.list_index) {
                        council_item.selected_info = council_item.selected_info.saturating_sub(1)
                    }
                }
                KeyCode::Delete => self.confirm_deletion_menu = Some(self.list_index),
                KeyCode::Enter => {
                    if let Some(council_item) = self.items.get(self.list_index) {
                        let text = match council_item.selected_info {
                            0 => council_item.name.clone(),
                            1 => council_item.password.clone(),
                            i => council_item.info[i - 2].1.clone(),
                        };
                        self.clipboard
                            .lock()
                            .unwrap()
                            .set_text(text)
                            .expect("Could not write the password to Clipboard")
                    }
                }
                KeyCode::Esc => self.exit = true,
                _ => (),
            }
        }
    }
}

impl CreateNewEntry {
    fn add_new_info_line(&mut self) {
        self.item.info.push((String::new(), String::new()));
        self.cursor_text_pos.info.push((0, 0));
    }
    fn compute_key(&mut self, key_event: KeyEvent) {
        let (mut_string, text_index) = match self.selected {
            CouncilItemSelect::Name => (&mut self.item.name, &mut self.cursor_text_pos.name),
            CouncilItemSelect::Password if self.generate_phase => (
                &mut self.item.password_gen_rules,
                &mut self.cursor_text_pos.password_gen,
            ),
            CouncilItemSelect::Password => {
                (&mut self.item.password, &mut self.cursor_text_pos.password)
            }
            CouncilItemSelect::First(i) => (
                &mut self.item.info[i].0,
                &mut self.cursor_text_pos.info[i].0,
            ),
            CouncilItemSelect::Second(i) => (
                &mut self.item.info[i].1,
                &mut self.cursor_text_pos.info[i].1,
            ),
        };

        match key_event.code {
            KeyCode::Char(ch) => {
                let end = mut_string.split_off(mut_string.len() - *text_index as usize);
                mut_string.push(ch);
                mut_string.push_str(end.as_str());
            }
            KeyCode::Backspace => {
                let end = mut_string.split_off(mut_string.len() - *text_index as usize);
                mut_string.pop();
                mut_string.push_str(end.as_str());
            }
            KeyCode::Right => *text_index = text_index.saturating_sub(1),
            KeyCode::Left => *text_index = (*text_index + 1).min(mut_string.len() as u16),
            KeyCode::End => *text_index = 0,
            KeyCode::Home => *text_index = mut_string.len() as u16,
            KeyCode::Tab if !self.item.name.is_empty() => match self.selected {
                CouncilItemSelect::Name => self.selected = CouncilItemSelect::Password,
                CouncilItemSelect::Password => {
                    if self.generate_phase {
                        self.generate_password()
                    } else {
                        self.selected = CouncilItemSelect::First(0);
                        if self.item.info.is_empty() {
                            self.add_new_info_line()
                        }
                    }
                }
                CouncilItemSelect::First(i) => self.selected = CouncilItemSelect::Second(i),
                CouncilItemSelect::Second(i) => {
                    if self.item.info.len() < i + 2 {
                        self.add_new_info_line();
                    }
                    self.selected = CouncilItemSelect::First(i + 1)
                }
            },
            KeyCode::BackTab => match self.selected {
                CouncilItemSelect::Name => (),
                CouncilItemSelect::Password => self.selected = CouncilItemSelect::Name,
                CouncilItemSelect::First(0) => self.selected = CouncilItemSelect::Password,
                CouncilItemSelect::First(i) => self.selected = CouncilItemSelect::Second(i - 1),
                CouncilItemSelect::Second(i) => self.selected = CouncilItemSelect::First(i),
            },
            _ => (),
        }
    }
    fn clean_up(&mut self) {
        self.item.info = self
            .item
            .info
            .iter()
            .cloned()
            .filter(|(name, data)| !name.is_empty() || !data.is_empty())
            .collect()
    }
    fn is_ready(&self) -> bool {
        !self.item.name.is_empty() && !self.item.password.is_empty() && !self.generate_phase
    }
    fn generate_password(&mut self) {
        let chars = self.item.password_gen_rules.chars().collect::<Vec<char>>();
        let password = (0..self.password_gen_len).fold(String::new(), |mut acc, _| {
            let rng_index = random_range(0..self.item.password_gen_rules.len());
            acc.push(chars[rng_index]);
            acc
        });
        self.generate_phase = false;
        self.item.password = password;
    }
}
