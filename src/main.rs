use std::{
    env,
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use crate::{crypto::Vault, ui::run};

mod app_state;
mod crypto;
mod store;
mod ui;
mod ui_helper;

use anyhow::Result;
pub use app_state::*;
use arboard::Clipboard;

const STD_PW_CHARS: [char; 93] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.',
    '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}',
];
fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    match args.get(1).map(|s| &s[..]) {
        Some("init") => {
            let password = rpassword::prompt_password("Council Password: ")?;
            store::create_folder(password)?;
        }
        None => {
            let password = rpassword::prompt_password("Council Password: ")?;
            let (salt, bytes) = store::read()?;
            let vault = Vault::new(salt, password)?;
            let text = vault.decrypt(bytes)?;
            let items = store::council_list_items(text)?;

            let mut state = AppState {
                exit: false,
                list_index: 0,
                items,
                new_item_input: None,
                confirm_deletion_menu: None,
                clipboard: Arc::new(Mutex::new(Clipboard::new()?)),
            };
            let run_result = ratatui::run(|term| run(term, &mut state));

            store::write_to_disk(state.items, &vault)?;
            run_result?;
        }
        _ => println!("print help"),
    }
    Ok(())
}
