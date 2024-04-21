use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashMap;

use anyhow::{anyhow, Result};

pub struct Council {
    hm: Option<HashMap<String, String>>,
    path: PathBuf,
    pub args: Vec<String>,
}

impl Council {
    pub fn new(
        args: Vec<String>,
        cocoon_unwrap: impl FnOnce(Vec<u8>) -> Result<String>,
    ) -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or(anyhow::Error::msg("No Home Folder"))?;
        let council_file = home_dir.join(".council");
        if council_file.exists() {
            let content = std::fs::read(council_file.clone())?;
            let hm = serde_json::from_str(&cocoon_unwrap(content)?)?;
            Ok(Self {
                hm,
                path: council_file,
                args,
            })
        } else {
            Ok(Self {
                hm: None,
                path: council_file,
                args,
            })
        }
    }
    pub fn init(&mut self, cocoon_wrap: impl FnOnce(String) -> Result<Vec<u8>>) -> Result<()> {
        let hm: HashMap<String, String> = HashMap::new();
        OpenOptions::new()
            .create_new(true) // Ensures only new file creation
            .write(true)
            .open(self.path.clone())?
            .write_all(&cocoon_wrap(serde_json::to_string_pretty(&hm)?)?)?;
        self.hm = Some(hm);
        Ok(())
    }
    pub fn save(&mut self, cocoon_wrap: impl FnOnce(String) -> Result<Vec<u8>>) -> Result<()> {
        if self.args.len() < 4 {
            return Err(anyhow!("Too few arguments"));
        }
        // should be safe
        if self.hm.clone().unwrap().get(&self.args[2]).is_none() {
            if let Some(ref mut hashmap) = self.hm {
                hashmap.insert(self.args[2].clone(), self.args[3].clone());
            } else {
                return Err(anyhow!("Couldn't save"));
            }
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(self.path.clone())?
                .write(&cocoon_wrap(serde_json::to_string_pretty(&self.hm)?)?)?;
            println!("\nSaved.");
        } else {
            println!("\nAlready set");
        }
        Ok(())
    }
    pub fn update(&mut self, cocoon_wrap: impl FnOnce(String) -> Result<Vec<u8>>) -> Result<()> {
        if self.args.len() < 4 {
            return Err(anyhow!("Too few arguments"));
        }
        if let Some(ref mut hashmap) = self.hm {
            if hashmap.get(&self.args[2]).is_some() {
                hashmap.insert(
                    format!("{}::old", &self.args[2]),
                    hashmap.get(&self.args[2]).unwrap().to_string(),
                );
                hashmap.insert(self.args[2].clone(), self.args[3].clone());
                OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(self.path.clone())?
                    .write(&cocoon_wrap(serde_json::to_string_pretty(&self.hm)?)?)?;
                println!("Updated [ {} ]", &self.args[2]);
            } else {
                println!("\nthere wasn't something to be updated");
            }
        } else {
            return Err(anyhow!("Couldn't save"));
        }
        // should be safe
        Ok(())
    }
    pub fn delete(&mut self, cocoon_wrap: impl FnOnce(String) -> Result<Vec<u8>>) -> Result<()> {
        if self.args.len() < 3 {
            return Err(anyhow!("Too few arguments"));
        }
        if let Some(ref mut hashmap) = self.hm {
            let remove = hashmap.remove(&self.args[2]);
            match remove {
                Some(v) => {
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(self.path.clone())?
                        .write(&cocoon_wrap(serde_json::to_string_pretty(&self.hm)?)?)?;
                    println!("deleted key [ {} ] with value [ {} ]", &self.args[2], v)
                }
                None => println!("Nothing was deleted"),
            }
        } else {
            return Err(anyhow!("Couldn't save"));
        }
        // should be safe
        Ok(())
    }
    pub fn list(&self) -> Result<()> {
        if let Some(hashmap) = &self.hm {
            println!("");
            for key in hashmap.keys() {
                println!("\t{key}");
            }
            Ok(())
        } else {
            Err(anyhow!("No file"))
        }
    }
    pub fn get(&self) -> Result<()> {
        if self.args.len() < 3 {
            return Err(anyhow!("Too few arguments"));
        }
        if let Some(hashmap) = &self.hm {
            match hashmap.get(&self.args[2]) {
                Some(val) => println!("\n[ {val} ]"),
                None => println!("\nNothing saved with the key {}", self.args[2]),
            }
            Ok(())
        } else {
            Err(anyhow!("No file"))
        }
    }
    pub fn copy_to_clipboard(&self) -> Result<()> {
        if self.args.len() < 3 {
            return Err(anyhow!("Too few arguments"));
        }
        if let Some(hashmap) = &self.hm {
            match hashmap.get(&self.args[2]) {
                Some(val) => match cli_clipboard::set_contents(val.to_string()) {
                    Err(err) => println!("\nCouldn't set clipboard due to {err}."),
                    Ok(_) => println!("\nSet clipboard."),
                },
                None => println!("\nNothing saved with the key {}", self.args[2]),
            }
            Ok(())
        } else {
            Err(anyhow!("No file"))
        }
    }
}
