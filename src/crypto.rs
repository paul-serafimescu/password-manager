pub use dirs::home_dir;
use fernet::{DecryptionError, Fernet};
pub use std::fs::File;
pub use std::io::prelude::*;
pub use std::path::{Path, PathBuf};

const KEY_FILE: &str = ".psswrdmngr_key";

pub trait AsBytes {
  fn _as_bytes(&self) -> &[u8];
}

impl AsBytes for String {
  fn _as_bytes(&self) -> &[u8] {
    self.as_bytes()
  }
}

impl AsBytes for &String {
  fn _as_bytes(&self) -> &[u8] {
    self.as_bytes()
  }
}

pub struct Cipher {
  key: String,
  _cipher: Fernet,
}

impl Cipher {
  pub(super) fn new() -> Self {
    let mut exists = true;
    let key_path = home_dir()
      .expect("your OS does not seem to have a $HOME")
      .join(KEY_FILE);
    let key = if let Ok(mut key_file) = File::open(&key_path) {
      let mut contents = String::new();
      key_file
        .read_to_string(&mut contents)
        .expect("buffer is not valid utf-8");
      contents.trim().to_owned()
    } else {
      exists = false;
      Fernet::generate_key()
    };
    let instance = Self {
      _cipher: Fernet::new(&key).expect("key is not 32 bit base64 encoded"),
      key,
    };
    if !exists {
      instance
        .dump_key(key_path)
        .expect("lacking write permissions");
    }
    instance
  }

  pub(super) fn encrypt<B: AsBytes>(&self, data: B) -> String {
    self._cipher.encrypt(data._as_bytes())
  }

  pub(super) fn decrypt<B: AsBytes>(
    &self,
    encrypted_data: &String,
  ) -> Result<Vec<u8>, DecryptionError> {
    self._cipher.decrypt(encrypted_data.as_str())
  }

  pub(super) fn dump_key<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    File::create(path)?.write(self.key.as_bytes())?;
    Ok(())
  }
}
