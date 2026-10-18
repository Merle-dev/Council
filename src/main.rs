use std::{
    env,
    sync::{Arc, Mutex},
};

use crate::{
    crypto::{Salt, Vault},
    ui::run,
};
use anyhow::{Result, anyhow};
use arboard::Clipboard;

mod app_state;
mod crypto;
mod store;
mod ui;
mod ui_helper;

pub use app_state::*;

const STD_PW_CHARS: [char; 93] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.',
    '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}',
];
fn main() -> Result<()> {
    let args = env::args()
        .skip(1)
        // .map(|str| str.as_str())
        .collect::<Vec<String>>();
    let args = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    match args.as_slice() {
        ["init"] => {
            let password = rpassword::prompt_password("Council Password: ")?;
            store::create_folder(password)?;
            Ok(())
        }
        ["change_pw"] => {
            let password = rpassword::prompt_password("Council Password: ")?;
            let new_pw_1 = rpassword::prompt_password("new Council Password: ")?;
            let new_pw_2 = rpassword::prompt_password("new Council Password: ")?;
            if new_pw_1 != new_pw_2 {
                return Err(anyhow!("new passwords are not the same, try again"));
            }
            let new_password = new_pw_1;

            let items = {
                let (salt, bytes) = store::read()?;
                let vault = Vault::new(salt, password)?;
                let text = vault.decrypt(bytes)?;
                store::council_list_items(text)?
            };
            let salt = Salt::new()?;
            let vault = Vault::new(salt, new_password)?;
            store::write_to_disk(items, Arc::new(vault))?;
            Ok(())
        }
        [] => {
            let password = rpassword::prompt_password("Council Password: ")?;
            let (salt, bytes) = store::read()?;
            let vault = Vault::new(salt, password)?;
            let text = vault.decrypt(bytes)?;
            let items = store::council_list_items(text)?;

            let mut state = AppState {
                exit: false,
                search: String::new(),
                list_index: 0,
                items,
                new_item_input: None,
                confirm_deletion_menu: None,
                clipboard: Arc::new(Mutex::new(Clipboard::new()?)),
                vault: Arc::new(vault),
            };
            ratatui::run(|term| run(term, &mut state))?;
            Ok(())
        }
        _ => {
            let text = r#"
    Council Password Manager

        commands:
            init            create a vault to store your passwords
            change_pw       swap the current password with the previous

        keys:
            N               create a new entry
            E               edit a entry
            del             remove a entry
            esc             exit app
            Enter           copy field
            "#;

            println!("{text}");
            Ok(())
        }
    }
}
