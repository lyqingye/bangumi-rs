use reqwest::{Method, Response, StatusCode, Url, header};
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};

use crate::{
    LoginState,
    error::{ApiError, Error, Result},
    ext::{Cookie, ResponseExt},
};
const NONE: Option<&'static ()> = Option::None;

pub struct Client {
    cli: reqwest::Client,
    endpoint: Url,
    state: Mutex<LoginState>,
}

impl Client {
    #[cfg(test)]
    pub fn new_from_env() -> Result<Self> {
        use crate::model::Credential;
        dotenv::dotenv().ok();
        let endpoint = std::env::var("QBITTORRENT_ENDPOINT").unwrap();
        let endpoint = Url::parse(&endpoint).unwrap();
        let cli = reqwest::Client::new();
        let state = Mutex::new(LoginState::NotLoggedIn {
            credential: Credential::new(
                std::env::var("QBITTORRENT_USERNAME").unwrap(),
                std::env::var("QBITTORRENT_PASSWORD").unwrap(),
            ),
        });
        Ok(Self {
            cli,
            endpoint,
            state,
        })
    }

    pub async fn login(&self, force: bool) -> Result<()> {
        let re_login = force || { self.state().as_cookie().is_none() };
        if re_login {
            debug!("Cookie not found, logging in");
            self.cli
                .request(Method::POST, self.url("auth/login"))
                .pipe(|req| {
                    req.form(
                        self.state()
                            .as_credential()
                            .expect("Credential should be set if cookie is not set"),
                    )
                })
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::IpBanned)),
                    _ => None,
                })?
                .extract::<Cookie>()?
                .pipe(|Cookie(cookie)| self.state.lock().unwrap().add_cookie(cookie));

            debug!("Log in success");
        } else {
            trace!("Already logged in, skipping");
        }

        Ok(())
    }

    pub async fn logout(&self) -> Result<()> {
        self.request(Method::POST, "auth/logout", NONE).await?.end()
    }

    fn url(&self, path: &'static str) -> Url {
        self.endpoint
            .join("api/v2/")
            .unwrap()
            .join(path)
            .expect("Invalid API endpoint")
    }

    async fn get(&self, path: &'static str) -> Result<Response> {
        self.request(Method::GET, path, NONE).await
    }

    async fn request(
        &self,
        method: Method,
        path: &'static str,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        for i in 0..3 {
            // If it's not the first attempt, we need to re-login
            self.login(i != 0).await?;

            let state = self.state();
            let cookie = state.as_cookie().expect("Cookie should be set after login");
            let mut req = self
                .cli
                .request(method.clone(), self.url(path))
                .header(header::COOKIE, cookie);

            if let Some(ref body) = body {
                match method {
                    Method::GET => req = req.query(body),
                    Method::POST => req = req.form(body),
                    _ => unreachable!("Only GET and POST are supported"),
                }
            }
            trace!(request = ?req, "Sending request");
            let res = req
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
                    _ => None,
                })
                .tap_ok(|response| trace!(?response));

            match res {
                Err(Error::ApiError(ApiError::NotLoggedIn)) => {
                    // Retry
                    warn!("Cookie is not valid, retrying");
                }
                Err(e) => return Err(e),
                Ok(t) => return Ok(t),
            }
        }

        Err(Error::ApiError(ApiError::NotLoggedIn))
    }

    fn state(&self) -> MutexGuard<'_, LoginState> {
        self.state.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_login() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_target(true)
            .init();
        let client = Client::new_from_env().unwrap();
        client.login(false).await.unwrap();
        println!("login success: {:?}", client.state().as_cookie());
        client.logout().await.unwrap();
    }
}
