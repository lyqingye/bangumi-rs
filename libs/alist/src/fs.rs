use crate::client::AListClient;
use crate::model::{
    AllFilesList, FsGetRequest, FsGetResponse, FsListRequest, FsListResponse, RecursiveFileItem,
    RecursiveFilesList,
};
use crate::Result;
use std::fmt::Debug;
use tracing::{debug, info, instrument};

impl AListClient {
    /// 获取文件列表
    ///
    /// 根据给定路径获取文件和目录列表。
    ///
    /// # 参数
    ///
    /// * `path` - 要获取列表的目录路径
    /// * `password` - 可选的文件夹密码
    /// * `page` - 分页页码，从1开始
    /// * `per_page` - 每页数量
    /// * `refresh` - 是否刷新缓存
    ///
    /// # 返回
    ///
    /// 成功时返回包含文件列表的响应
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
    /// // 获取根目录文件列表
    /// let files = client.list_files("/", None, 1, 100, false).await?;
    /// for file in &files.content {
    ///     println!("文件名: {}, 大小: {}, 是否文件夹: {}",
    ///         file.name, file.size, file.is_dir);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, path, password), err)]
    pub async fn list_files(
        &self,
        path: impl Into<String> + Debug,
        password: Option<impl Into<String> + Debug>,
        page: u32,
        per_page: u32,
        refresh: bool,
    ) -> Result<FsListResponse> {
        let url = format!("{}/api/fs/list", self.base_url.trim_end_matches('/'));

        let request = FsListRequest {
            path: path.into(),
            password: password.map(|p| p.into()),
            page,
            per_page,
            refresh,
        };

        self.post_json(&url, &request).await
    }

    /// 获取目录下所有文件（递归遍历所有页）
    ///
    /// 获取指定路径下的所有文件，通过多次分页请求获取全部内容。
    ///
    /// # 参数
    ///
    /// * `path` - 要获取列表的目录路径
    /// * `password` - 可选的文件夹密码
    /// * `refresh` - 是否刷新缓存
    ///
    /// # 返回
    ///
    /// 成功时返回包含所有文件列表的响应
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
    /// // 获取根目录下所有文件
    /// let all_files = client.list_all_files("/", None, false).await?;
    /// println!("总文件数: {}, 总大小: {}", all_files.total_count, all_files.total_size);
    /// for file in &all_files.files {
    ///     println!("文件名: {}, 大小: {}", file.name, file.size);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, path, password), err)]
    pub async fn list_all_files(
        &self,
        path: impl Into<String> + Debug + Clone,
        password: Option<impl Into<String> + Debug + Clone>,
        refresh: bool,
    ) -> Result<AllFilesList> {
        const PER_PAGE: u32 = 100;
        let path_str = path.into();
        let password_opt = password.map(|p| p.into());

        // 先获取第一页以确定总数
        let first_page = self
            .list_files(path_str.clone(), password_opt.clone(), 1, PER_PAGE, refresh)
            .await?;

        let total = first_page.total as u32;
        let provider = first_page.provider.clone();
        let mut all_files = first_page.content;

        // 计算需要的页数
        let pages_needed = total.div_ceil(PER_PAGE);

        // 如果有多页，继续获取其余页面
        if pages_needed > 1 {
            for page in 2..=pages_needed {
                info!("获取第 {}/{} 页", page, pages_needed);
                let page_result = self
                    .list_files(
                        path_str.clone(),
                        password_opt.clone(),
                        page,
                        PER_PAGE,
                        refresh,
                    )
                    .await?;
                all_files.extend(page_result.content);
            }
        }

        // 计算总大小
        let total_size = all_files.iter().map(|file| file.size).sum();

        Ok(AllFilesList {
            total_count: all_files.len(),
            total_size,
            provider,
            files: all_files,
        })
    }

    /// 获取文件下载链接
    ///
    /// 获取指定文件的下载链接。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    /// * `password` - 可选的文件夹密码
    ///
    /// # 返回
    ///
    /// 成功时返回包含下载URL的响应
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
    /// // 获取文件下载链接
    /// let file = client.get_file("/path/to/file.mp4", None).await?;
    /// println!("下载URL: {}", file.raw_url);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, path, password), err)]
    pub async fn get_file(
        &self,
        path: impl Into<String> + Debug,
        password: Option<impl Into<String> + Debug>,
    ) -> Result<FsGetResponse> {
        let url = format!("{}/api/fs/get", self.base_url.trim_end_matches('/'));

        let request = FsGetRequest {
            path: path.into(),
            password: password.map(|p| p.into()),
        };

        self.post_json(&url, &request).await
    }

    /// 递归获取目录下所有文件
    ///
    /// 递归遍历指定路径及其所有子目录，获取所有文件。
    ///
    /// # 参数
    ///
    /// * `path` - 要遍历的目录路径
    /// * `password` - 可选的文件夹密码
    /// * `refresh` - 是否刷新缓存
    /// * `max_depth` - 最大递归深度，None表示无限制
    ///
    /// # 返回
    ///
    /// 成功时返回包含所有文件的递归结果
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
    /// // 递归获取根目录下所有文件
    /// let all_files = client.list_recursive_files("/", None, false, Some(3)).await?;
    /// println!("总文件数: {}, 文件夹数: {}, 总大小: {}",
    ///     all_files.total_count, all_files.total_dirs, all_files.total_size);
    ///
    /// // 输出所有文件夹
    /// for dir in &all_files.directories {
    ///     println!("文件夹: {}", dir);
    /// }
    ///
    /// // 输出所有文件
    /// for file in &all_files.files {
    ///     println!("文件: {}, 大小: {}", file.name, file.size);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, path, password), err)]
    pub async fn list_recursive_files(
        &self,
        path: impl Into<String> + Debug + Clone,
        password: Option<impl Into<String> + Debug + Clone>,
        refresh: bool,
        max_depth: Option<usize>,
    ) -> Result<RecursiveFilesList> {
        let root_path = path.into();
        let password_opt = password.map(|p| p.into());
        let mut result = RecursiveFilesList::default();

        // 递归上下文结构体
        struct TraverseContext<'a> {
            client: &'a AListClient,
            password: Option<String>,
            refresh: bool,
            max_depth: Option<usize>,
            result: &'a mut RecursiveFilesList,
            visited_paths: std::collections::HashSet<String>,
        }

        // 递归辅助函数
        async fn traverse_dir(
            ctx: &mut TraverseContext<'_>,
            path: String,
            current_depth: usize,
        ) -> Result<()> {
            // 检查递归深度
            if let Some(max) = ctx.max_depth {
                if current_depth > max {
                    debug!("达到最大递归深度 {}, 不再继续遍历路径: {}", max, path);
                    return Ok(());
                }
            }

            // 检查路径是否已被访问（防止符号链接导致的循环）
            if !ctx.visited_paths.insert(path.clone()) {
                info!("检测到可能的循环引用，路径已被访问过: {}", path);
                return Ok(());
            }

            // 获取当前目录的所有文件和子目录
            let resp = ctx
                .client
                .list_all_files(&path, ctx.password.clone(), ctx.refresh)
                .await?;

            // 记录当前目录
            if !path.eq("/") {
                ctx.result.directories.push(path.clone());
                ctx.result.total_dirs += 1;
            }

            debug!("正在处理目录: {}, 包含 {} 个项目", path, resp.files.len());

            // 处理文件和子目录
            for item in &resp.files {
                if item.is_dir {
                    // 构建子目录路径
                    let sub_path = if path.ends_with('/') {
                        format!("{}{}", path, item.name)
                    } else {
                        format!("{}/{}", path, item.name)
                    };

                    // 递归处理子目录
                    // 使用Box::pin包装异步递归调用
                    let future = Box::pin(traverse_dir(ctx, sub_path, current_depth + 1));
                    future.await?;
                } else {
                    // 构建完整文件路径
                    let full_path = if path.ends_with('/') {
                        format!("{}{}", path, item.name)
                    } else {
                        format!("{}/{}", path, item.name)
                    };

                    // 记录文件与完整路径
                    ctx.result.files.push(RecursiveFileItem {
                        file: item.clone(),
                        full_path,
                    });
                    ctx.result.total_count += 1;
                    ctx.result.total_size += item.size;
                }
            }

            Ok(())
        }

        // 创建上下文
        let mut ctx = TraverseContext {
            client: self,
            password: password_opt,
            refresh,
            max_depth,
            result: &mut result,
            visited_paths: std::collections::HashSet::new(),
        };

        // 开始递归
        info!("开始递归遍历目录: {}", root_path);
        traverse_dir(&mut ctx, root_path, 0).await?;

        info!(
            "递归遍历完成, 找到 {} 个文件, {} 个目录, 总大小: {}字节",
            result.total_count, result.total_dirs, result.total_size
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::AListClient;

    async fn create_client() -> AListClient {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();
        let client = AListClient::new("http://localhost:5244", "");
        let result = client
            .login("admin", "123456", None::<String>)
            .await
            .unwrap();
        // 创建新客户端，使用获取到的token
        AListClient::new("http://localhost:5244", result.token)
    }

    #[ignore]
    #[tokio::test]
    async fn test_list_files() {
        let client = create_client().await;
        let files = client
            .list_files("/downloads", None::<String>, 1, 100, false)
            .await
            .unwrap();
        println!("文件总数: {}", files.total);
        for file in &files.content {
            println!(
                "名称: {}, 大小: {}, 类型: {}, 修改时间: {}",
                file.name,
                file.size,
                if file.is_dir { "文件夹" } else { "文件" },
                file.modified
            );
        }
        println!("提供者: {}, 可写: {}", files.provider, files.write);
    }

    #[ignore]
    #[tokio::test]
    async fn test_list_all_files() {
        let client = create_client().await;
        let all_files = client
            .list_all_files("/", None::<String>, false)
            .await
            .unwrap();
        println!(
            "总文件数: {}, 总大小: {}字节",
            all_files.total_count, all_files.total_size
        );
        println!("提供者: {}", all_files.provider);

        // 输出前10个文件
        for (i, file) in all_files.files.iter().take(10).enumerate() {
            println!(
                "#{}: 名称: {}, 大小: {}, 类型: {}",
                i + 1,
                file.name,
                file.size,
                if file.is_dir { "文件夹" } else { "文件" }
            );
        }

        if all_files.total_count > 10 {
            println!("... 还有 {} 个文件未显示", all_files.total_count - 10);
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_list_recursive_files() {
        let client = create_client().await;

        // 使用有限深度以防止遍历太深
        let all_files = client
            .list_recursive_files("/downloads", None::<String>, false, Some(2))
            .await
            .unwrap();

        println!(
            "总文件数: {}, 文件夹数: {}, 总大小: {}字节",
            all_files.total_count, all_files.total_dirs, all_files.total_size
        );

        // 输出所有文件夹
        println!("=== 文件夹列表 ===");
        for (i, dir) in all_files.directories.iter().take(10).enumerate() {
            println!("#{}: {}", i + 1, dir);
        }

        if all_files.directories.len() > 10 {
            println!(
                "... 还有 {} 个文件夹未显示",
                all_files.directories.len() - 10
            );
        }

        // 输出部分文件
        println!("\n=== 文件列表 ===");
        for (i, file) in all_files.files.iter().take(10).enumerate() {
            println!(
                "#{}: 完整路径: {}\n    名称: {}, 大小: {}字节, 修改时间: {}",
                i + 1,
                file.full_path,
                file.file.name,
                file.file.size,
                file.file.modified
            );

            // 获取并显示下载链接
            let file_info = client
                .get_file(&file.full_path, None::<String>)
                .await
                .unwrap();
            println!("    下载URL: {}", file_info.raw_url);
        }

        if all_files.files.len() > 10 {
            println!("... 还有 {} 个文件未显示", all_files.files.len() - 10);
        }
    }
}
