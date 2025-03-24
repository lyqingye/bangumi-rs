use anyhow::{Context, Result};
use lru::LruCache;
use serde::Serialize;
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{error, info, warn};

use crate::{Notifier, telegram};

// 主题定义
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Topic {
    Download,
    System,
    Error,
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topic::Download => write!(f, "下载"),
            Topic::System => write!(f, "系统"),
            Topic::Error => write!(f, "错误"),
        }
    }
}

// 通知消息
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub topic: String,
    pub title: String,
    pub content: String,
    pub timestamp: chrono::NaiveDateTime,
}

impl Message {
    pub fn new(topic: Topic, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            topic: topic.to_string(),
            title: title.into(),
            content: content.into(),
            timestamp: chrono::Local::now().naive_utc(),
        }
    }

    fn format_for_notification(&self) -> String {
        format!(
            "*[{}通知]* {}\n\n{}\n\n_发送时间: {}_",
            self.topic,
            self.title,
            self.content,
            self.timestamp.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

// 主题配置
#[derive(Debug, Clone)]
pub struct TopicConfig {
    cooldown: Duration,
    max_cache_size: usize,
}

impl Default for TopicConfig {
    fn default() -> Self {
        Self {
            cooldown: Duration::from_secs(300),
            max_cache_size: 50,
        }
    }
}

// 内部消息类型
#[derive(Debug)]
enum WorkerMessage {
    Notify(Message),
    Shutdown(oneshot::Sender<()>),
}

// 消息处理器
struct MessageProcessor {
    notifiers: Arc<Mutex<Vec<Box<dyn Notifier>>>>,
    topic_configs: HashMap<Topic, TopicConfig>,
    message_caches: HashMap<Topic, Arc<Mutex<LruCache<String, chrono::NaiveDateTime>>>>,
}

impl MessageProcessor {
    fn new(
        notifiers: Arc<Mutex<Vec<Box<dyn Notifier>>>>,
        topic_configs: HashMap<Topic, TopicConfig>,
        message_caches: HashMap<Topic, Arc<Mutex<LruCache<String, chrono::NaiveDateTime>>>>,
    ) -> Self {
        Self {
            notifiers,
            topic_configs,
            message_caches,
        }
    }

    async fn process_message(&self, message: Message) -> Result<()> {
        let topic = Topic::from_str(&message.topic)
            .with_context(|| format!("无效的主题: {}", message.topic))?;

        if !self
            .should_send_message(&topic, &message.title, &message.content)
            .await?
        {
            info!("消息在冷却中，跳过发送: [{:?}] {}", topic, message.title);
            return Ok(());
        }

        self.send_to_notifiers(&message).await
    }

    async fn should_send_message(&self, topic: &Topic, title: &str, content: &str) -> Result<bool> {
        let cache = self
            .message_caches
            .get(topic)
            .with_context(|| format!("未找到主题缓存: {:?}", topic))?;
        let config = self
            .topic_configs
            .get(topic)
            .with_context(|| format!("未找到主题配置: {:?}", topic))?;

        let key = Self::generate_message_key(topic, title, content);
        let mut cache = cache.lock().await;

        if let Some(last_time) = cache.get(&key) {
            let now = chrono::Local::now().naive_utc();
            if now.signed_duration_since(*last_time) < chrono::Duration::from_std(config.cooldown)?
            {
                return Ok(false);
            }
        }

        cache.put(key, chrono::Local::now().naive_utc());
        Ok(true)
    }

    async fn send_to_notifiers(&self, message: &Message) -> Result<()> {
        let text = message.format_for_notification();
        let notifiers = self.notifiers.lock().await;

        for notifier in notifiers.iter() {
            if let Err(e) = notifier.send_formatted_message(&text, "Markdown").await {
                warn!("通知器发送消息失败: {}", e);
            }
        }
        Ok(())
    }

    fn generate_message_key(topic: &Topic, title: &str, content: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        topic.hash(&mut hasher);
        title.hash(&mut hasher);
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Topic {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "下载" => Ok(Topic::Download),
            "系统" => Ok(Topic::System),
            "错误" => Ok(Topic::Error),
            _ => Err(anyhow::anyhow!("无效的主题: {}", s)),
        }
    }
}

#[derive(Clone, Default)]
pub struct Worker {
    notifiers: Arc<Mutex<Vec<Box<dyn Notifier>>>>,
    topic_configs: HashMap<Topic, TopicConfig>,
    message_caches: HashMap<Topic, Arc<Mutex<LruCache<String, chrono::NaiveDateTime>>>>,
    tx: Option<mpsc::Sender<WorkerMessage>>,
    is_spawned: Arc<std::sync::atomic::AtomicBool>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            notifiers: Arc::new(Mutex::new(Vec::new())),
            topic_configs: HashMap::new(),
            message_caches: HashMap::new(),
            tx: None,
            is_spawned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn from_env() -> Result<Self> {
        let mut worker = Self::new();
        worker.add_notifier(Box::new(telegram::TelegramNotifier::from_env()?));
        worker.init_default_configs();
        Ok(worker)
    }

    fn init_default_configs(&mut self) {
        self.set_topic_config(Topic::Download, TopicConfig::default());
        self.set_topic_config(Topic::System, TopicConfig::default());
        self.set_topic_config(
            Topic::Error,
            TopicConfig {
                cooldown: Duration::from_secs(60),
                max_cache_size: 2000,
            },
        );
    }

    pub fn add_notifier(&mut self, notifier: Box<dyn Notifier>) {
        if let Ok(mut notifiers) = self.notifiers.try_lock() {
            notifiers.push(notifier);
        }
    }

    pub fn set_topic_config(&mut self, topic: Topic, config: TopicConfig) {
        self.topic_configs.insert(topic.clone(), config.clone());
        self.message_caches.insert(
            topic,
            Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_cache_size).unwrap(),
            ))),
        );
    }

    pub async fn spawn(&mut self) -> Result<()> {
        if self.is_spawned.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(anyhow::anyhow!("通知服务已经启动"));
        }

        // 确保所有主题都有配置
        for topic in [Topic::Download, Topic::System, Topic::Error] {
            if !self.topic_configs.contains_key(&topic) {
                self.set_topic_config(topic, TopicConfig::default());
            }
        }

        let (tx, mut rx) = mpsc::channel(100);
        self.tx = Some(tx);

        let processor = MessageProcessor::new(
            self.notifiers.clone(),
            self.topic_configs.clone(),
            self.message_caches.clone(),
        );

        let is_spawned = self.is_spawned.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    WorkerMessage::Notify(message) => {
                        if let Err(e) = processor.process_message(message).await {
                            error!("处理通知消息失败: {}", e);
                        }
                    }
                    WorkerMessage::Shutdown(done_tx) => {
                        info!("通知服务收到停机信号");
                        let _ = done_tx.send(());
                        break;
                    }
                }
            }
            is_spawned.store(false, std::sync::atomic::Ordering::SeqCst);
            info!("通知服务已停止");
        });

        self.is_spawned
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub async fn notify(
        &self,
        topic: Topic,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<()> {
        if !self.is_spawned.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(anyhow::anyhow!("通知服务未启动"));
        }

        let message = Message::new(topic, title, content);
        if let Some(tx) = &self.tx {
            tx.send(WorkerMessage::Notify(message))
                .await
                .context("发送通知消息失败")?;
        }

        Ok(())
    }

    pub async fn notify_error(
        &self,
        title: impl Into<String>,
        error: impl std::fmt::Display,
    ) -> Result<()> {
        self.notify(Topic::Error, title, format!("错误信息: {}", error))
            .await
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("开始停止通知 Worker...");

        if let Some(tx) = &self.tx {
            let (done_tx, done_rx) = oneshot::channel();
            tx.send(WorkerMessage::Shutdown(done_tx))
                .await
                .context("发送停机信号失败")?;

            // 等待 worker 确认停止
            done_rx.await.context("等待 worker 停止失败")?;

            info!("通知 Worker 已停止");
        }
        Ok(())
    }
}
