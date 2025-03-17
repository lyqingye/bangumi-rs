use reqwest::{header, Method, Response, StatusCode, Url};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    borrow::Borrow,
    sync::{Mutex, MutexGuard},
};
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};

use crate::model::Credential;
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
    pub fn new_from_env() -> Self {
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
        Self {
            cli,
            endpoint,
            state,
        }
    }

    pub fn new<T: Into<String>>(
        cli: reqwest::Client,
        endpoint: Url,
        username: T,
        password: T,
    ) -> Self {
        Self {
            cli,
            endpoint,
            state: Mutex::new(LoginState::NotLoggedIn {
                credential: Credential::new(username.into(), password.into()),
            }),
        }
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
                            let p = reqwest::multipart::Part::bytes(torrent.data.to_vec())
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

    pub async fn stop_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/stop", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn start_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/start", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn set_force_start(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        value: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            value: bool,
        }

        self.post(
            "torrents/setForceStart",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                value,
            }),
        )
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
    use crate::model::Sep;

    use super::*;

    async fn create_client() -> Client {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_target(true)
            .init();
        let client = Client::new_from_env();
        client.login(false).await.unwrap();
        client
    }

    #[ignore]
    #[tokio::test]
    async fn test_login() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();
        let client = Client::new_from_env();
        client.login(false).await.unwrap();
        let arg = GetTorrentListArg::default();
        let torrents = client.get_torrent_list(arg).await.unwrap();
        println!("torrents: {:?}", torrents);
        println!("login success: {:?}", client.state().as_cookie());
        client.logout().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_add_torrent() {
        let client = create_client().await;
        let arg = AddTorrentArg {
            source: TorrentSource::Urls {
                urls: Sep::<Url, '\n'>::from(vec!["https://mikanani.me/Download/20250202/3eebfcc6839fefea06f0675958013659dfc6d80f.torrent".parse().unwrap()]),
            },
            savepath: Some("/downloads".to_string()),
            ..Default::default()
        };
        client.add_torrent(arg).await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_list_torrents() {
        let client = create_client().await;
        let list_arg = GetTorrentListArg {
            hashes: Some("3eebfcc6839fefea06f0675958013659dfc6d80f".to_string()),
            ..Default::default()
        };
        let torrents = client.get_torrent_list(list_arg).await.unwrap();
        println!("torrents: {:?}", torrents);
    }

    #[ignore]
    #[tokio::test]
    async fn test_stop_torrents() {
        let client = create_client().await;
        client
            .stop_torrents(Hashes::Hashes(Sep::from(vec![
                "3eebfcc6839fefea06f0675958013659dfc6d80f".to_string(),
            ])))
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_start_torrents() {
        let client = create_client().await;
        client
            .start_torrents(Hashes::Hashes(Sep::from(vec![
                "3eebfcc6839fefea06f0675958013659dfc6d80f".to_string(),
            ])))
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_delete_torrents() {
        let client = create_client().await;
        client
            .delete_torrents(
                Hashes::Hashes(Sep::from(vec![
                    "3eebfcc6839fefea06f0675958013659dfc6d80f".to_string()
                ])),
                true,
            )
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_add_magnet() {
        let client = create_client().await;
        let arg = AddTorrentArg {
            source: TorrentSource::Urls {
                urls: Sep::<Url, '\n'>::from(vec![
                    "magnet:?xt=urn:btih:3395bec505f46591094519d008f991f71734877d"
                        .parse()
                        .unwrap(),
                ]),
            },
            savepath: Some("/downloads".to_string()),
            ..Default::default()
        };
        client.add_torrent(arg).await.unwrap();
    }
}
