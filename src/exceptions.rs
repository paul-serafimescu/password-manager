#[macro_export]
macro_rules! define_exception {
  ($name: ident, $message: expr) => {
    #[derive(Clone, Copy, Debug)]
    pub struct $name();

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, $message)
      }
    }

    impl std::error::Error for $name {}
  };
}

define_exception!(MultipleArgError, "too many arguments entered.");
define_exception!(
  InvalidJSONError,
  "file does not contain proper JSON schema."
);
