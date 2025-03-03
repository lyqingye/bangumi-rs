use std::{collections::HashMap, path::PathBuf, time::Duration};

mod mock_store;
use mock_store::MockStore;
use downloader::Store;
use downloader::{config::Config, worker::Worker, MockThirdPartyDownloader, RemoteTaskStatus};
use model::sea_orm_active_enums::DownloadStatus;

#[tokio::test]
async fn test_worker() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .init();
    let mock_store = MockStore::new();
    let mut mock_downloader = MockThirdPartyDownloader::new();
    mock_downloader.expect_add_task().returning(|_, _| Ok(None));
    mock_downloader
        .expect_name()
        .returning(|| "mock_downloader");
    let mut tasks = HashMap::new();
    tasks.insert(
        "123".to_string(),
        RemoteTaskStatus {
            status: DownloadStatus::Failed,
            err_msg: Some("error msg".to_string()),
            result: None,
        },
    );
    mock_downloader
        .expect_list_tasks()
        .returning(move |_| Ok(tasks.clone()));
    mock_downloader.expect_remove_task().returning(|_| Ok(()));
    let mut worker = Worker::new_with_conn(
        Box::new(mock_store.clone()),
        Box::new(mock_downloader),
        Config {
            max_retry_count: 1,
            sync_interval: Duration::from_nanos(1),
            retry_processor_interval: Duration::from_secs(30),
            retry_min_interval: Duration::from_nanos(1),
            retry_max_interval: Duration::from_nanos(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    worker.spawn().await.unwrap();
    let worker_clone = worker.clone();

    worker_clone
        .add_task("123", PathBuf::from("test"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    worker_clone.sync_remote_task_status().await.unwrap();

    worker_clone.shutdown().await.unwrap();
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
