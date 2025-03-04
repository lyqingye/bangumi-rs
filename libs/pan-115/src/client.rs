use std::{
    collections::HashMap,
    env,
    num::NonZero,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::{
    decode, encode,
    errors::Pan115Error,
    gen_key,
    iter::FileStream,
    model::{
        BasicResp, DownloadResp, FileInfo, FileListResp, LoginResp, OfflineAddUrlResponse,
        OfflineTask, OfflineTaskResp,
    },
};
use anyhow::Result;
use governor::{Jitter, Quota, RateLimiter};
use reqwest::{
    cookie::{CookieStore, Jar},
    Url,
};
use serde::{Deserialize, Serialize};

pub const API_LOGIN_CHECK: &str = "https://passportapi.115.com/app/1.0/web/1.0/check/sso";
pub const USER_AGENT: &str = "Mozilla/5.0 115Browser/27.0.5.7";
pub const APP_VER: &str = "27.0.5.7";

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_second: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 1,
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub cli: reqwest::Client,
    pub user_id: i64,
    pub limiter: Arc<
        RateLimiter<
            governor::state::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
        >,
    >,
}

impl Client {
    pub fn new(
        cookies: &str,
        rate_limit_config: Option<RateLimitConfig>,
    ) -> Result<Self, Pan115Error> {
        let mut jar = Jar::default();
        Self::import_cookie(cookies, &mut jar)?;

        let cli = reqwest::Client::builder()
            .cookie_provider(Arc::new(jar))
            .user_agent(USER_AGENT)
            .build()?;

        let rate_limit_config = rate_limit_config.unwrap_or_default();
        let quota =
            Quota::per_second(NonZero::new(rate_limit_config.max_requests_per_second).unwrap())
                .allow_burst(NonZero::new(1).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));
        Ok(Self {
            cli,
            user_id: 0,
            limiter,
        })
    }

    pub fn new_from_env() -> Result<Self, Pan115Error> {
        let cookies = env::var("COOKIES")
            .map_err(|_| Pan115Error::CookieParseFailed("COOKIES 未设置".to_string()))?;
        Self::new(&cookies, None)
    }

    pub async fn acquire(&self) {
        let _ = self.limiter.until_n_ready(NonZero::new(1).unwrap()).await;
    }

    fn import_cookie(cookie: &str, cookie_jar: &mut Jar) -> Result<(), Pan115Error> {
        let url = "https://115.com/".parse::<Url>().unwrap();
        let cookies = cookie.split("; ").collect::<Vec<&str>>();
        for cookie in cookies {
            let cookie_values = cookie.split("=").collect::<Vec<&str>>();
            if cookie_values.len() != 2 {
                return Err(Pan115Error::CookieParseFailed(format!(
                    "cookie parse failed: {}",
                    cookie
                )));
            }
            cookie_jar.add_cookie_str(format!("{}; Domain=.115.com", cookie).as_str(), &url);
        }

        Ok(())
    }

    pub async fn login_check(&mut self) -> Result<(), Pan115Error> {
        self.acquire().await;

        let resp: LoginResp = self
            .cli
            .get(API_LOGIN_CHECK)
            .query(&[(
                "_",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            )])
            .send()
            .await?
            .json()
            .await?;

        resp.is_ok()?;

        if let Some(data) = resp.data {
            self.user_id = data.user_id;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use reqwest::{header, Url};
    use std::{env, sync::Arc};
    use tokio_stream::{self as stream, StreamExt};

    #[tokio::test]
    #[ignore]
    async fn test_login_check() -> Result<()> {
        dotenv::dotenv().ok();
        let mut client = Client::new_from_env()?;
        client.login_check().await?;
        Ok(())
    }
}
