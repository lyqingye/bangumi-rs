use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use tap::Pipe;

mod torrent;

/// Username and password used to authenticate with qBittorrent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    username: String,
    password: String,
}

impl Credential {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Return a dummy credential when you passed in the cookie instead of
    /// actual credential.
    pub fn dummy() -> Self {
        Self {
            username: "".to_owned(),
            password: "".to_owned(),
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.username.is_empty() && self.password.is_empty()
    }
}

/// A wrapper around `Vec<T>` that implements `FromStr` and `ToString` as
/// `C`-separated strings where `C` is a char.
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Sep<T, const C: char>(Vec<T>);

impl<T: FromStr, const C: char> FromStr for Sep<T, C> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(C)
            .map(T::from_str)
            .collect::<Result<Vec<_>, Self::Err>>()?
            .pipe(Sep::from)
            .pipe(Ok)
    }
}

/// A wrapper around `str` that ensures the string is non-empty.
pub struct NonEmptyStr<T>(T);

impl<T: AsRef<str>> NonEmptyStr<T> {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn new(s: T) -> Option<Self> {
        if s.as_ref().is_empty() {
            None
        } else {
            Some(NonEmptyStr(s))
        }
    }
}

impl<T: Display, const C: char> Display for Sep<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_slice() {
            [] => Ok(()),
            [x] => x.fmt(f),
            [x, xs @ ..] => {
                x.fmt(f)?;
                for x in xs {
                    f.write_char(C)?;
                    x.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}

impl<V: Into<Vec<T>>, T, const C: char> From<V> for Sep<T, C> {
    fn from(inner: V) -> Self {
        Sep(inner.into())
    }
}
