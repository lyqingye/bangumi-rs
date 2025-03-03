use std::{collections::HashMap, path::PathBuf, time::Duration};

mod mock_store;
use chrono::Local;
use downloader::Store;
use downloader::{config::Config, worker::Worker, MockThirdPartyDownloader, RemoteTaskStatus};
use mock_store::MockStore;
use model::sea_orm_active_enums::DownloadStatus;
use model::torrent_download_tasks;

// 初始化测试环境
fn init_test_env() {
    dotenv::dotenv().ok();
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::INFO)
    //     .with_target(true)
    //     .init();
}

// 创建测试用的配置
fn create_test_config() -> Config {
    Config {
        max_retry_count: 1,
        sync_interval: Duration::from_nanos(1),
        retry_processor_interval: Duration::from_secs(30),
        retry_min_interval: Duration::from_nanos(1),
        retry_max_interval: Duration::from_nanos(1),
        ..Default::default()
    }
}

// 创建模拟下载器，可自定义任务状态
fn create_mock_downloader() -> MockThirdPartyDownloader {
    let mut mock_downloader = MockThirdPartyDownloader::new();
    mock_downloader.expect_add_task().returning(|_, _| Ok(None));
    mock_downloader
        .expect_name()
        .returning(|| "mock_downloader");
    mock_downloader.expect_cancel_task().returning(|_| Ok(()));

    mock_downloader.expect_remove_task().returning(|_| Ok(()));

    mock_downloader
}

// 创建Worker实例
async fn create_test_worker(
    mock_store: MockStore,
    mock_downloader: MockThirdPartyDownloader,
    config: Config,
) -> Worker {
    Worker::new_with_conn(
        Box::new(mock_store.clone()),
        Box::new(mock_downloader),
        config,
    )
    .await
    .unwrap()
}

// 创建失败状态的任务集合
fn create_failed_tasks() -> HashMap<String, RemoteTaskStatus> {
    let mut tasks = HashMap::new();
    tasks.insert(
        "123".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: Some("error msg".to_string()),
            result: None,
        },
    );
    tasks
}

#[tokio::test]
async fn test_retry_exceed_max_count() {
    // 初始化测试环境
    init_test_env();

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let failed_tasks = create_failed_tasks();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(failed_tasks.clone()));
    let config = create_test_config();

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务
    worker_clone
        .add_task("123", PathBuf::from("test"))
        .await
        .unwrap();

    // 等待同步
    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();

    // 关闭worker
    worker_clone.shutdown().await.unwrap();

    // 验证结果
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Failed])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "123");
    assert_eq!(tasks[0].download_status, DownloadStatus::Failed);
    assert_eq!(
        tasks[0].err_msg,
        Some("重试次数超过上限: error msg".to_string())
    );
    assert_eq!(tasks[0].dir, "/test");
}

// 可以添加更多测试用例，复用上面的辅助函数
#[tokio::test]
async fn test_worker_retry_success() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut failed_remote_tasks = HashMap::new();
    failed_remote_tasks.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: None,
            result: None,
        },
    );

    let mut success_remote_tasks = HashMap::new();
    success_remote_tasks.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Completed,
            err_msg: None,
            result: Some("completed".to_string()),
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .once()
        .returning(move |_| Ok(failed_remote_tasks.clone()));
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(success_remote_tasks.clone()));
    let config = create_test_config();

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task("456", PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Completed])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "456");
    assert_eq!(tasks[0].context, Some("completed".to_string()));
}

#[tokio::test]
async fn test_worker_add_task_success() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));
    let config = create_test_config();

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task("456", PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "456");
}

#[tokio::test]
async fn test_worker_add_cancel_downloading_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));
    let config = create_test_config();

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task("456", PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.cancel_task("456").await.unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Cancelled])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "456");
}

#[tokio::test]
async fn test_worker_add_retry_failed_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut failed_remote_task = HashMap::new();
    failed_remote_task.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: None,
            result: None,
        },
    );

    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .once()
        .returning(move |_| Ok(failed_remote_task.clone()));
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));
    let mut config = create_test_config();
    config.max_retry_count = 0;

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task("456", PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.retry_task("456").await.unwrap();
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "456");
}

#[tokio::test]
async fn test_worker_recover_pending_tasks() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        "456".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    mock_store
        .insert_task(torrent_download_tasks::Model {
            info_hash: "456".to_string(),
            download_status: DownloadStatus::Pending,
            downloader: Some("mock_downloader".to_string()),
            dir: "test2".to_string(),
            context: None,
            err_msg: None,
            retry_count: 0,
            next_retry_at: Local::now().naive_utc(),
            created_at: Local::now().naive_utc(),
            updated_at: Local::now().naive_utc(),
        })
        .await
        .unwrap();
    let mut mock_downloader = create_mock_downloader();
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));
    let config = create_test_config();

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader, config).await;
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, "456");
}