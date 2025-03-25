use crate::model::{LoginRequest, LoginResponse, UserInfo};
use tracing::instrument;

use crate::Result;
use crate::client::AListClient;

impl AListClient {
    pub async fn login(&mut self) -> Result<()> {
        let url = format!("{}/api/auth/login", self.base_url.trim_end_matches('/'));

        let request = LoginRequest {
            username: self.user_name.clone(),
            password: self.user_password.clone(),
            otp_code: None,
        };

        let login_result: LoginResponse = self.post_json(&url, &request).await?;
        self.token = login_result.token;
        Ok(())
    }

    #[instrument(skip(self), err)]
    pub async fn get_me(&self) -> Result<UserInfo> {
        let url = format!("{}/api/me", self.base_url.trim_end_matches('/'));
        self.get(&url).await
    }
}
