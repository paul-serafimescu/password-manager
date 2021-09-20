mod cli;
mod crypto;
mod exceptions;

extern crate clap;
extern crate dirs;
extern crate fernet;
extern crate serde_json;

use crate::cli::{parse_args, Command};
use crate::crypto::{home_dir, Cipher, File, Path, PathBuf, Read, Write};
use crate::exceptions::*;
use crate::serde_json::{Map, Value};

const DEFAULT_FILE_STORAGE: &str = ".psswrdmngr.json";

fn all_some<T>(vector: &Vec<Option<T>>) -> bool {
  for val in vector {
    if val.is_none() {
      return false;
    }
  }
  true
}

fn get_dump_file(file: Option<&String>) -> PathBuf {
  if let Some(_file) = file {
    Path::new(_file).join("")
  } else {
    home_dir()
      .expect("your OS does not seem to have a $HOME")
      .join(DEFAULT_FILE_STORAGE)
  }
}

fn load_json_content(file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
  let mut contents = String::new();
  let mut created = false;
  if let Ok(f) = File::open(file_path) {
    f
  } else {
    created = true;
    File::create(file_path)?;
    File::open(file_path)?
  }
  .read_to_string(&mut contents)?;
  if created {
    contents.push_str("{}")
  }
  Ok(contents)
}

fn json_parse(contents: &String) -> Result<Value, Box<dyn std::error::Error>> {
  Ok(serde_json::from_str(contents.as_str())?)
}

fn load_json_map(contents: String) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
  let parsed = json_parse(&contents)?;
  let mut main_map = Map::new();
  for (key, value) in parsed.as_object().unwrap().clone().into_iter() {
    main_map.insert(key, value);
  }
  Ok(main_map)
}

fn index_credentials<'a, 'b>(
  json_structure: &Value,
  name: &String,
  keys: &'a [&'a str; 2],
) -> Option<Vec<String>> {
  let indexed: Vec<Option<String>> = keys
    .into_iter()
    .map(|key| {
      json_structure[name][key]
        .as_str()
        .map(|some| some.to_owned())
    })
    .collect();
  if all_some(&indexed) {
    Some(
      indexed
        .into_iter()
        .map(|element| element.unwrap())
        .collect(),
    )
  } else {
    None
  }
}

fn fetch_credentials(
  cipher: &Cipher,
  name: &String,
  file: Option<&String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let _file = get_dump_file(file);
  let mut contents = String::new();
  File::open(_file)?.read_to_string(&mut contents)?;
  let parsed = json_parse(&contents)?;
  if let Some(creds) = index_credentials(&parsed, name, &["username", "password"]) {
    Ok(
      creds
        .into_iter()
        .map(|element| {
          std::str::from_utf8(cipher.decrypt::<String>(&element).unwrap().as_slice())
            .unwrap()
            .to_owned()
        })
        .collect(),
    )
  } else {
    Err(Box::new(InvalidJSONError()))
  }
}

fn add_credentials(
  cipher: &Cipher,
  name: &String,
  file: Option<&String>,
  username: &String,
  password: &String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let encrypted = (
    name.to_owned(),
    cipher.encrypt(username),
    cipher.encrypt(password),
  );
  let _file = get_dump_file(file);
  let contents = load_json_content(&_file)?;
  let mut writable = File::create(_file)?;
  let mut new_item = Map::new();
  let mut main_map = load_json_map(contents)?;
  new_item.insert("username".to_owned(), Value::String(encrypted.1));
  new_item.insert("password".to_owned(), Value::String(encrypted.2));
  main_map.insert(encrypted.0, Value::Object(new_item));
  writable.write(serde_json::to_string_pretty(&main_map)?.as_bytes())?;
  Ok(Vec::new())
}

fn remove_credentials(
  name: &String,
  file: Option<&String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let _file = get_dump_file(file);
  let contents = load_json_content(&_file)?;
  let mut writable = File::create(_file)?;
  let mut main_map = load_json_map(contents)?;
  if main_map.remove(name) == None {
    println!("Entry not found, no action required.")
  }
  writable.write(serde_json::to_string_pretty(&main_map)?.as_bytes())?;
  Ok(Vec::new())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let arguments = parse_args()?;
  let cipher = Cipher::new();
  let result = match arguments.command {
    Command::Get => fetch_credentials(&cipher, &arguments.name.unwrap(), arguments.file.as_ref())?,
    Command::Add => add_credentials(
      &cipher,
      &arguments.name.unwrap(),
      arguments.file.as_ref(),
      &arguments.username.unwrap(),
      &arguments.password.unwrap(),
    )?,
    Command::Remove => remove_credentials(&arguments.name.unwrap(), arguments.file.as_ref())?,
    Command::Unknown => vec!["invalid option. quitting...".to_owned()],
  };
  for _r in result {
    println!("{}", _r)
  }
  Ok(())
}
