#![allow(dead_code)]

use model::Credential;
mod client;
mod error;
mod ext;
mod model;

#[derive(Clone)]
enum LoginState {
    CookieProvided {
        cookie: String,
    },
    NotLoggedIn {
        credential: Credential,
    },
    LoggedIn {
        cookie: String,
        credential: Credential,
    },
}

impl LoginState {
    fn as_cookie(&self) -> Option<&str> {
        match self {
            Self::CookieProvided { cookie } => Some(cookie),
            Self::NotLoggedIn { .. } => None,
            Self::LoggedIn { cookie, .. } => Some(cookie),
        }
    }

    fn as_credential(&self) -> Option<&Credential> {
        match self {
            Self::CookieProvided { .. } => None,
            Self::NotLoggedIn { credential } => Some(credential),
            Self::LoggedIn { credential, .. } => Some(credential),
        }
    }

    fn add_cookie(&mut self, cookie: String) {
        match self {
            Self::CookieProvided { .. } => {}
            Self::LoggedIn { credential, .. } | Self::NotLoggedIn { credential } => {
                *self = Self::LoggedIn {
                    cookie,
                    credential: credential.clone(),
                };
            }
        }
    }
}
