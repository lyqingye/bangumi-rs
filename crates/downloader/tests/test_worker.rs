use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf, time::Duration};

mod mock_store;
use chrono::Local;
use downloader::config::GenericConfig;
use downloader::{config::Config, worker::Worker, MockThirdPartyDownloader, RemoteTaskStatus};
use downloader::{resource::Resource, Store};
use mock_store::MockStore;
use model::sea_orm_active_enums::DownloadStatus;
use model::torrent_download_tasks;

// 初始化测试环境
fn init_test_env() {
    dotenv::dotenv().ok();
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .try_init();
}

// 创建测试用的配置
fn create_test_config() -> GenericConfig {
    GenericConfig {
        max_retry_count: 1,
        retry_min_interval: chrono::Duration::nanoseconds(1),
        retry_max_interval: chrono::Duration::nanoseconds(1),
        download_dir: PathBuf::from("test"),
        ..Default::default()
    }
}

fn create_test_resource() -> Resource {
    Resource::from_info_hash("f6ebf8a1f26d01f317c8e94ec40ebb3dd1a75d40").unwrap()
}

// 创建模拟下载器，可自定义任务状态
fn create_mock_downloader(config: GenericConfig) -> MockThirdPartyDownloader {
    let mut mock_downloader = MockThirdPartyDownloader::new();
    mock_downloader.expect_add_task().returning(|_, _| Ok(None));
    mock_downloader
        .expect_name()
        .returning(|| "mock_downloader");
    mock_downloader.expect_cancel_task().returning(|_| Ok(()));

    mock_downloader
        .expect_remove_task()
        .returning(|_, _| Ok(()));

    mock_downloader.expect_pause_task().returning(|_| Ok(()));

    mock_downloader.expect_resume_task().returning(|_| Ok(()));
    mock_downloader.expect_config().return_const(config);
    mock_downloader
}

// 创建Worker实例
fn create_test_worker(mock_store: MockStore, mock_downloader: MockThirdPartyDownloader) -> Worker {
    Worker::new_with_conn(
        Box::new(mock_store.clone()),
        Config {
            sync_interval: Duration::from_millis(100),
            ..Default::default()
        },
        vec![Arc::new(Box::new(mock_downloader))],
    )
    .unwrap()
}

// 创建失败状态的任务集合
fn create_failed_tasks() -> HashMap<String, RemoteTaskStatus> {
    let mut tasks = HashMap::new();
    let resource = create_test_resource();
    tasks.insert(
        resource.info_hash().to_string(),
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
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(failed_tasks.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    let resource = create_test_resource();
    // 添加任务
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test"))
        .await
        .unwrap();

    // 等待同步
    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;

    // 关闭worker
    worker_clone.shutdown().await.unwrap();

    // 验证结果
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Failed])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
    assert_eq!(tasks[0].download_status, DownloadStatus::Failed);
    assert_eq!(
        tasks[0].err_msg,
        Some("重试次数超过上限(1): error msg".to_string())
    );
    assert_eq!(tasks[0].dir, "test");
}

#[tokio::test]
async fn test_download_timeout_no_retry() {
    // 初始化测试环境
    init_test_env();

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let resource = create_test_resource();
    let mut pending_tasks = create_failed_tasks();
    pending_tasks.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );
    let mut config = create_test_config();
    config.max_retry_count = 0;
    config.download_timeout = chrono::Duration::nanoseconds(1);
    let mut mock_downloader = create_mock_downloader(config);
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_tasks.clone()));
    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test"))
        .await
        .unwrap();

    // 等待同步
    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;

    // 关闭worker
    worker_clone.shutdown().await.unwrap();

    // 验证结果
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Failed])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
    assert_eq!(tasks[0].download_status, DownloadStatus::Failed);
    assert_eq!(
        tasks[0].err_msg,
        Some("重试次数超过上限(0): 下载超时".to_string())
    );
    assert_eq!(tasks[0].dir, "test");
}

// 可以添加更多测试用例，复用上面的辅助函数
#[tokio::test]
async fn test_worker_retry_success() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut failed_remote_tasks = HashMap::new();
    let resource = create_test_resource();
    failed_remote_tasks.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: None,
            result: None,
        },
    );

    let mut success_remote_tasks = HashMap::new();
    success_remote_tasks.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Completed,
            err_msg: None,
            result: Some("completed".to_string()),
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .once()
        .returning(move |_| Ok(failed_remote_tasks.clone()));
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(success_remote_tasks.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Completed])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
    assert_eq!(tasks[0].context, Some("completed".to_string()));
}

#[tokio::test]
async fn test_worker_add_task_success() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_add_cancel_downloading_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let mut pending_remote_task = HashMap::new();
    let resource = create_test_resource();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.cancel_task(resource.info_hash()).unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Cancelled])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_add_retry_failed_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut failed_remote_task = HashMap::new();
    failed_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: None,
            result: None,
        },
    );

    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut config = create_test_config();
    config.max_retry_count = 0;
    let mut mock_downloader = create_mock_downloader(config);
    mock_downloader
        .expect_list_tasks()
        .once()
        .returning(move |_| Ok(failed_remote_task.clone()));
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.retry_task(resource.info_hash()).unwrap();
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_recover_pending_tasks() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
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
            info_hash: resource.info_hash().to_string(),
            download_status: DownloadStatus::Pending,
            downloader: "mock_downloader".to_string(),
            dir: "test2".to_string(),
            context: None,
            err_msg: None,
            retry_count: 0,
            next_retry_at: Local::now().naive_utc(),
            created_at: Local::now().naive_utc(),
            updated_at: Local::now().naive_utc(),
            magnet: None,
            resource_type: resource.get_type(),
        })
        .await
        .unwrap();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_pause_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Downloading,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.pause_task(resource.info_hash()).unwrap();
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Paused])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_resume_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Paused,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));
    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.resume_task(resource.info_hash()).unwrap();
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Downloading])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}

#[tokio::test]
async fn test_worker_user_manual_pause_task() {
    // 初始化测试环境
    init_test_env();

    // 创建自定义状态的任务
    let resource = create_test_resource();
    let mut pending_remote_task = HashMap::new();
    pending_remote_task.insert(
        resource.info_hash().to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Paused,
            err_msg: None,
            result: None,
        },
    );

    // 准备测试数据和依赖
    let mock_store = MockStore::new();
    let mut mock_downloader = create_mock_downloader(create_test_config());
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(pending_remote_task.clone()));

    // 创建并启动worker
    let mut worker = create_test_worker(mock_store.clone(), mock_downloader);
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    // 添加任务并同步
    worker_clone
        .add_task(resource.clone(), PathBuf::from("test2"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await;
    worker_clone.shutdown().await.unwrap();

    // 验证下载中的任务状态
    let tasks = mock_store
        .list_by_status(&[DownloadStatus::Paused])
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].info_hash, resource.info_hash());
}
