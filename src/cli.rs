use std::io::{stdin, stdout, Write};
use clap::{Arg, App, ArgMatches};

macro_rules! build_arg {
  ($matches: expr, $key: expr) => {
    $matches.value_of($key).map(|value| value.to_owned())
  };
}

pub fn readline() -> String {
  let mut line = String::new();
  stdin().read_line(&mut line).expect("unable to read from stdin");
  line
}

#[derive(Debug, PartialEq)]
pub enum UnionType<T, U> {
  First(T), Second(U)
}

impl<T, U> UnionType<T, U> {
  fn first(self) -> T {
    match self {
      Self::First(value) => value,
      _ => panic!()
    }
  }

  fn second(self) -> U {
    match self {
      Self::Second(value) => value,
      _ => panic!()
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
  Add, Remove, Get
}

impl From<Command> for String {
  fn from(cmd: Command) -> Self {
    match cmd {
      Command::Add => "add",
      Command::Remove => "remove",
      Command::Get => "get"
    }.to_owned()
  }
}

impl From<&str> for Command {
  fn from(string: &str) -> Self {
    match string {
      "r" | "remove" => Command::Remove,
      "g" | "get" => Command::Get,
      "a" | "add" => Command::Add,
      _ => panic!()
    }
  }
}

#[derive(Clone, Debug)]
pub struct Arguments {
  pub file: Option<String>,
  pub name: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
  pub command: Command,
}

impl Arguments {
  pub fn new() -> Self {
    Self {
      file: None,
      name: None,
      username: None,
      password: None,
      command: Command::Get,
    }
  }

  pub fn set(&mut self, key: &str, value: UnionType<Option<String>, Command>) {
    match key {
      "name" => self.name = value.first(),
      "file" => self.file = value.first(),
      "username" => self.username = value.first(),
      "password" => self.password = value.first(),
      "command" => self.command = value.second(),
      _ => ()
    }
  }

  pub fn is_empty(&self) -> bool {
    self.file == None &&
      self.name == None &&
      self.username == None &&
      self.password == None &&
      self.command == Command::Get
  }

  pub fn is_incomplete(&self) -> bool {
    match self {
      Self {
        username: Some(_),
        password: Some(_),
        command: Command::Add,
        name: Some(_),
        ..
      } | Self {
        command: Command::Remove,
        name: Some(_),
        ..
      } | Self {
        command: Command::Get,
        name: Some(_),
        ..
      } => false,
      _ => true
    }
  }

  fn missing(&self) -> Vec<&str> {
    match self.command {
      Command::Add => {
        match (self.name.is_some(), self.username.is_some(), self.password.is_some()) {
          (true, true, true) => vec![],
          (true, true, false) => vec!["password"],
          (true, false, false) => vec!["username", "password"],
          (false, false, false) => vec!["name", "username", "password"],
          (true, false, true) => vec!["username"],
          (false, false, true) => vec!["name", "username"],
          (false, true, false) => vec!["name", "password"],
          (false, true, true) => vec!["name"],
        }
      },
      Command::Remove => {
        if let Some(_) = self.name {
          vec![]
        } else {
          vec!["name"]
        }
      },
      Command::Get => {
        if let Some(_) = self.name {
          vec![]
        } else {
          vec!["name"]
        }
      }
    }
  }

  fn interactive_fill(&mut self) {
    for key in self.clone().missing() {
      print!("Enter {}: ", key);
      stdout().flush().expect("stdout flush failed!");
      let _read = readline();
      let line = _read.trim_end();
      self.set(key, UnionType::First(Some(if key == "name" {
        line.to_lowercase()
      } else { line.to_owned() })))
    }
  }

  pub fn fill(mut self) -> Self {
    if self.is_empty() {
      println!(
        "Enter action:
          r - remove
          a - add
          g - get");
      self.set("command", UnionType::Second(Command::from(readline().trim_end())));
    }
    self.interactive_fill();
    println!();
    self
  }
}

#[derive(Clone, Copy, Debug)]
pub struct MultipleArgError();

impl std::fmt::Display for MultipleArgError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "too many arguments entered.")
  }
}

impl std::error::Error for MultipleArgError {}

fn repetition_exists(matches: &ArgMatches, names: &[&str]) -> (Command, bool) {
  let mut count = 0;
  let mut name = "";
  for arg in names {
    if matches.is_present(arg) {
      name = *arg;
      count += 1
    }
  }
  (match name {
    "remove" => Command::Remove,
    "add" => Command::Add,
    _ => Command::Get
  }, count > 1)
}

pub fn parse_args() -> Result<Arguments, MultipleArgError> {
  let mut arguments = Arguments::new();
  let matches = App::new("Password Manager")
                  .version("1.0")
                  .author("Paul Serafimescu <paulserafimescu@gmail.com>")
                  .about("Store and manage encrypted passwords on your device.")
                  .arg(Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .value_name("FILE")
                    .help("Select location of password on disk")
                    .takes_value(true))
                  .arg(Arg::with_name("add")
                    .short("a")
                    .long("add")
                    .help("Add password"))
                  .arg(Arg::with_name("remove")
                    .short("r")
                    .long("remove")
                    .help("Remove password"))
                  .arg(Arg::with_name("get")
                    .short("g")
                    .long("get")
                    .help("Retrieve password for application (default option)"))
                  .arg(Arg::with_name("name")
                    .short("n")
                    .long("name")
                    .value_name("APPLICATION NAME")
                    .help("Name of application")
                    .takes_value(true))
                  .arg(Arg::with_name("username")
                    .short("u")
                    .long("username")
                    .value_name("USERNAME")
                    .help("Username for application")
                    .takes_value(true))
                  .arg(Arg::with_name("password")
                    .short("p")
                    .long("password")
                    .value_name("PASSWORD")
                    .help("Password to be entered")
                    .takes_value(true))
                  .get_matches();
  arguments = Arguments {
    file: build_arg!(matches, "file"),
    name: matches.value_of("name").map(|name| name.to_owned().to_lowercase()),
    password: build_arg!(matches, "password"),
    username: build_arg!(matches, "username"),
    ..arguments
  };
  let (command, repetition) = repetition_exists(&matches, &["get", "remove", "add"]);
  if repetition {
    return Err(MultipleArgError())
  } else {
    arguments.command = command;
  }
  Ok(if arguments.is_incomplete() {
    println!("Input is invalid/incomplete...");
    println!();
    arguments.fill()
  } else { arguments })
}
