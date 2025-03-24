use crate::model::{LoginRequest, LoginResponse, UserInfo};
use tracing::instrument;

use crate::client::AListClient;
use crate::Result;

impl AListClient {
    /// 登录AList获取Token
    ///
    /// 发送用户凭证到AList服务器，获取认证token。
    ///
    /// # 参数
    ///
    /// * `username` - 用户名
    /// * `password` - 密码
    /// * `otp_code` - 可选的二次验证码
    ///
    /// # 返回
    ///
    /// 成功时返回包含token的响应
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use alist::{AListClient, LoginRequest};
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// let client = AListClient::new("https://alist.example.com", "");
    ///
    /// // 无二次验证登录
    /// let login_result = client.login("admin", "password123", None).await?;
    /// println!("Token: {}", login_result.data.token);
    ///
    /// // 带二次验证登录
    /// let login_result = client.login("admin", "password123", Some("123456")).await?;
    /// println!("Token: {}", login_result.data.token);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn login(
        &self,
        username: impl Into<String>,
        password: impl Into<String>,
        otp_code: Option<impl Into<String>>,
    ) -> Result<LoginResponse> {
        let url = format!("{}/api/auth/login", self.base_url.trim_end_matches('/'));

        let request = LoginRequest {
            username: username.into(),
            password: password.into(),
            otp_code: otp_code.map(|code| code.into()),
        };

        self.post_json(&url, &request).await
    }

    /// 获取当前用户信息
    ///
    /// 使用已有的token获取当前登录用户的详细信息。
    ///
    /// # 返回
    ///
    /// 成功时返回包含用户信息的响应
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use alist::AListClient;
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// let client = AListClient::new("https://alist.example.com", "your_token_here");
    ///
    /// let user_info = client.get_me().await?;
    /// println!("用户名: {}", user_info.data.username);
    /// println!("角色: {}", user_info.data.role);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), err)]
    pub async fn get_me(&self) -> Result<UserInfo> {
        let url = format!("{}/api/me", self.base_url.trim_end_matches('/'));
        self.get(&url).await
    }
}
