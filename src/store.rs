use std::{
    env,
    fs::{File, create_dir},
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{
    CouncilListItem,
    crypto::{Salt, Vault},
};

#[derive(Serialize, Deserialize)]
struct CouncilStoreItem {
    pub name: String,
    pub password: String,
    pub info: Vec<(String, String)>,
}

fn file_dir() -> Result<PathBuf> {
    let mut path = env::home_dir().ok_or(anyhow!("Could not get your home directory"))?;
    path.push(".council");
    path.push("default");
    Ok(path)
}

pub fn write_to_disk(mut items: Vec<CouncilListItem>, vault: &Vault) -> Result<()> {
    items.sort_by_key(|entry| entry.name.clone());
    let write_data = items
        .iter()
        .cloned()
        .map(
            |CouncilListItem {
                 name,
                 password,
                 info,
                 ..
             }| CouncilStoreItem {
                name,
                password,
                info,
            },
        )
        .collect::<Vec<CouncilStoreItem>>();
    let json_body = serde_json::to_string(&write_data)?;
    let mut encrypted = vault.encrypt(json_body)?;
    encrypted.append(&mut vault.salt.to_vec());

    let mut file = File::create(file_dir()?)?;
    file.write_all(&encrypted)?;

    Ok(())
}

pub fn create_folder(password: String) -> Result<()> {
    let mut path = env::home_dir().ok_or(anyhow!("Could not get your home directory"))?;
    path.push(".council");
    create_dir(path.clone())?;
    path.push("default");
    let mut file = File::create_new(path)?;
    let salt = Salt::new()?;
    let vault = Vault::new(salt, password)?;
    let mut bytes = vault.encrypt(String::from("[]"))?;
    bytes.append(&mut vault.salt.to_vec());
    file.write_all(&bytes)?;
    Ok(())
}

pub fn read() -> Result<(Salt, Vec<u8>)> {
    let mut file = File::open(file_dir()?)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let salt = buf.split_off(buf.len() - 16);
    let salt = Salt::from(salt)?;
    Ok((salt, buf))
}

pub fn council_list_items(text: String) -> Result<Vec<CouncilListItem>> {
    let council_store_items: Vec<CouncilStoreItem> = serde_json::from_str(&text)?;
    Ok(council_store_to_council_list(council_store_items))
}

fn council_store_to_council_list(mut vec: Vec<CouncilStoreItem>) -> Vec<CouncilListItem> {
    vec.sort_by_key(|entry| entry.name.clone());
    vec.into_iter()
        .map(
            |CouncilStoreItem {
                 name,
                 password,
                 info,
             }| CouncilListItem {
                name,
                password,
                password_gen_rules: "".to_string(),
                info,
                selected_info: 0,
            },
        )
        .collect()
}
