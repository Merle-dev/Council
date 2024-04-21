use std::env;

use anyhow::{anyhow, Result};
use cocoon::Cocoon;

mod council;
use council::Council;

fn main() -> Result<(), anyhow::Error> {
    if env::args().collect::<Vec<String>>().len() == 1 {
        print_help();
        return Ok(());
    }
    let password = rpassword::prompt_password("[Coucil password] ")?;

    let cocoon_wrap = |plain: String| -> Result<Vec<u8>> {
        let mut cocoon = Cocoon::new(password.as_bytes());
        match cocoon.wrap(plain.as_bytes()) {
            Ok(v) => Ok(v),
            Err(_err) => Err(anyhow!("encryption error")),
        }
    };
    let cocoon_unwrap = |crypt: Vec<u8>| -> Result<String> {
        let cocoon = Cocoon::new(password.as_bytes());
        match cocoon.unwrap(&crypt.as_slice()) {
            Ok(v) => Ok(String::from_utf8(v)?),
            Err(_err) => Err(anyhow!("decryption error")),
        }
    };

    let mut council = Council::new(env::args().collect(), cocoon_unwrap)?;
    if council.args.len() > 1 {
        match council.args[1].as_str() {
            "-t" | "--init" => council.init(cocoon_wrap)?,
            "-s" | "--save" => council.save(cocoon_wrap)?,
            "-u" | "--update" => council.update(cocoon_wrap)?,
            "-d" | "--delete" => council.delete(cocoon_wrap)?,
            "-l" | "--list" => council.list()?,
            "-p" | "--print" => council.get()?,
            "-c" | "--clip" => council.copy_to_clipboard()?,
            _ => {
                print_help();
                return Ok(());
            }
        }
    } else {
        print_help();
        return Ok(());
    }
    print_success();
    Ok(())
}

fn print_help() {
    println!(
        "\n
        \t-s | --save [key] [value]\t\t- Saves a key with a value.
        \t-u | --update [key] [value]\t\t- Updates a key and creates and \"::old\" key.
        \t-d | --delele [key] \t\t\t- Deletes a key.
        \t-p | --print [key]\t\t\t- Prints a value.
        \t-l | --list \t\t\t\t- Prints a list of all keys.
        \t-i | --init\t\t\t\t- Initlizes the program at the user home.
        "
    );
}

fn print_success() {
    println!(
        r#"
 ___ _   _  ___ ___ ___  ___ ___ 
/ __| | | |/ __/ __/ _ \/ __/ __|
\__ \ |_| | (_| (_|  __/\__ \__ \
|___/\__,_|\___\___\___||___/___/"#
    );
}
