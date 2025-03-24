use crate::client::AListClient;
use crate::model::{
    AllFilesList, FsGetRequest, FsGetResponse, FsListRequest, FsListResponse, RecursiveFileItem,
    RecursiveFilesList,
};
use crate::Result;
use std::fmt::Debug;
use tracing::{debug, info, instrument};

impl AListClient {
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

    #[instrument(skip(self, path, password), err)]
    pub async fn get_file(
        &self,
        path: impl Into<String> + Debug,
        password: Option<impl Into<String> + Debug>,
    ) -> Result<Option<FsGetResponse>> {
        let url = format!("{}/api/fs/get", self.base_url.trim_end_matches('/'));

        let request = FsGetRequest {
            path: path.into(),
            password: password.map(|p| p.into()),
        };

        self.post_json(&url, &request).await
    }

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
        let mut client = AListClient::new("http://localhost:5244", "admin", "123456");
        client.login().await.unwrap();
        client
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
            println!("    下载URL: {}", file_info.unwrap().raw_url);
        }

        if all_files.files.len() > 10 {
            println!("... 还有 {} 个文件未显示", all_files.files.len() - 10);
        }
    }
}
