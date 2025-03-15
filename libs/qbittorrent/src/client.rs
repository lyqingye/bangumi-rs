use reqwest::{header, Method, Response, StatusCode, Url};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    borrow::Borrow,
    sync::{Mutex, MutexGuard},
};
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};

use crate::{
    error::{ApiError, Error, Result},
    ext::{Cookie, ResponseExt},
    model::torrent::{AddTorrentArg, GetTorrentListArg, Hashes, HashesArg, Torrent, TorrentSource},
    LoginState,
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
                    StatusCode::FORBIDDEN => Some(Error::Api(ApiError::IpBanned)),
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

    pub async fn get_torrent_list(&self, arg: GetTorrentListArg) -> Result<Vec<Torrent>> {
        self.get_with("torrents/info", &arg)
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn add_torrent(&self, arg: impl Borrow<AddTorrentArg> + Send + Sync) -> Result<()> {
        let a: &AddTorrentArg = arg.borrow();
        match &a.source {
            TorrentSource::Urls { urls: _ } => {
                self.post("torrents/add", Some(arg.borrow())).await?.end()
            }
            TorrentSource::TorrentFiles { torrents } => {
                for i in 0..3 {
                    // If it's not the first attempt, we need to re-login
                    self.login(i != 0).await?;
                    // Create a multipart form containing the torrent files and other arguments
                    let form = torrents.iter().fold(
                        serde_json::to_value(a)?
                            .as_object()
                            .unwrap()
                            .into_iter()
                            .fold(reqwest::multipart::Form::new(), |form, (k, v)| {
                                // If we directly call to_string() on a Value containing a string like "hello",
                                // it will include the quotes: "\"hello\"".
                                // We need to use as_str() first to get the inner string without quotes.
                                let v = match v.as_str() {
                                    Some(v_str) => v_str.to_string(),
                                    None => v.to_string(),
                                };
                                form.text(k.to_string(), v.to_string())
                            }),
                        |mut form, torrent| {
                            let p = reqwest::multipart::Part::bytes(torrent.data.clone())
                                .file_name(torrent.filename.to_string())
                                .mime_str("application/x-bittorrent")
                                .unwrap();
                            form = form.part("torrents", p);
                            form
                        },
                    );
                    let req = self
                        .cli
                        .request(Method::POST, self.url("torrents/add"))
                        .multipart(form)
                        .header(header::COOKIE, {
                            self.state()
                                .as_cookie()
                                .expect("Cookie should be set after login")
                        });

                    trace!(request = ?req, "Sending request");
                    let res = req
                        .send()
                        .await?
                        .map_status(|code| match code as _ {
                            StatusCode::FORBIDDEN => Some(Error::Api(ApiError::NotLoggedIn)),
                            _ => None,
                        })
                        .tap_ok(|response| trace!(?response));

                    match res {
                        Err(Error::Api(ApiError::NotLoggedIn)) => {
                            // Retry
                            warn!("Cookie is not valid, retrying");
                        }
                        Err(e) => return Err(e),
                        Ok(t) => return t.end(),
                    }
                }

                Err(Error::Api(ApiError::NotLoggedIn))
            }
        }
    }

    pub async fn pause_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/pause", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn resume_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/resume", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn delete_torrents(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        delete_files: impl Into<Option<bool>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        #[serde(rename_all = "camelCase")]
        struct Arg {
            hashes: Hashes,
            delete_files: Option<bool>,
        }
        self.post(
            "torrents/delete",
            Some(&Arg {
                hashes: hashes.into(),
                delete_files: delete_files.into(),
            }),
        )
        .await?
        .end()
    }
}

impl Client {
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

    async fn get_with(
        &self,
        path: &'static str,
        param: &(impl Serialize + Sync),
    ) -> Result<Response> {
        self.request(Method::GET, path, Some(param)).await
    }

    async fn post(
        &self,
        path: &'static str,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        self.request(Method::POST, path, body).await
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

            let cookie = {
                let state = self.state();
                state
                    .as_cookie()
                    .expect("Cookie should be set after login")
                    .to_owned()
            };
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
                    StatusCode::FORBIDDEN => Some(Error::Api(ApiError::NotLoggedIn)),
                    _ => None,
                })
                .tap_ok(|response| trace!(?response));

            match res {
                Err(Error::Api(ApiError::NotLoggedIn)) => {
                    // Retry
                    warn!("Cookie is not valid, retrying");
                }
                Err(e) => return Err(e),
                Ok(t) => return Ok(t),
            }
        }

        Err(Error::Api(ApiError::NotLoggedIn))
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
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();
        let client = Client::new_from_env().unwrap();
        client.login(false).await.unwrap();
        let arg = GetTorrentListArg::default();
        let torrents = client.get_torrent_list(arg).await.unwrap();
        println!("torrents: {:?}", torrents);
        println!("login success: {:?}", client.state().as_cookie());
        client.logout().await.unwrap();
    }
}
