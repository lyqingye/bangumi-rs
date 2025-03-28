// API 错误类型
export class ApiError extends Error {
  constructor(
    message: string,
    public code: number = -1,
    public status?: number
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

// 基础响应类型
export interface ApiResponse<T> {
  code: number
  msg: string | null
  data: T
}

// 分页响应类型
export interface PaginatedResponse<T> {
  list: T[]
  total: number
}

// 订阅状态枚举
export enum SubscribeStatus {
  None = 'None',
  Subscribed = 'Subscribed',
  Downloaded = 'Downloaded'
}

// 下载状态枚举
export enum DownloadStatus {
  Pending = 'Pending',
  Downloading = 'Downloading',
  Completed = 'Completed',
  Failed = 'Failed',
  Retrying = 'Retrying',
  Cancelled = 'Cancelled',
  Paused = 'Paused'
}

// 下载状态枚举
export enum State {
  Missing = 'Missing',
  Ready = 'Ready',
  Downloading = 'Downloading',
  Downloaded = 'Downloaded',
  Failed = 'Failed',
  Retrying = 'Retrying'
}

// 订阅参数
export interface SubscribeParams {
  status: SubscribeStatus
  start_episode_number?: number | undefined
  resolution_filter?: string | undefined
  language_filter?: string | undefined
  release_group_filter?: string | undefined
  collector_interval?: number | undefined
  metadata_interval?: number | undefined
  enforce_torrent_release_after_broadcast?: boolean | undefined
  preferred_downloader?: string | undefined
  allow_fallback?: boolean | undefined
}

// 番剧信息
export interface Bangumi {
  id: number
  name: string
  description: string
  bangumi_tv_id: number
  tmdb_id: number
  mikan_id: number
  poster_image_url: string
  backdrop_image_url: string
  air_date: string
  air_week: number
  rating: number
  ep_count: number
  subscribe_status: SubscribeStatus
  created_at: string
  updated_at: string
  season_number: number
  start_episode_number: number | null
  resolution_filter: string | null
  language_filter: string | null
  release_group_filter: string | null
  enforce_torrent_release_after_broadcast: boolean | null
  preferred_downloader: string | null
  allow_fallback: boolean
}

// 剧集信息
export interface Episode {
  id: number
  bangumi_id: number
  number: number
  sort_number: number | null
  name: string | null
  image_url: string | null
  description: string | null
  air_date: string | null
  duration_seconds: number | null
  kind: string
  created_at: string
  updated_at: string
  // 下载状态相关字段
  download_state: State | null
  ref_torrent_info_hash: string | null
  task_created_at: string | null
  task_updated_at: string | null
}

// 种子信息
export interface Torrent {
  info_hash: string
  title: string
  size: number
  magnet: string
  pub_date: string

  // 文件解析信息
  release_group: string | null
  season_number: number | null
  episode_number: number | null
  language: string | null
  video_resolution: string | null
  parser_status: string | null

  // 下载信息
  download_status: DownloadStatus | null
  downloader_name: string | null
  task_created_at: string | null
}

// 下载任务查询参数
export interface QueryDownloadTask {
  offset: number
  limit: number
  status?: DownloadStatus
}

// 下载任务信息
export interface DownloadTask {
  bangumi_id: number
  name: string
  episode_number: number
  info_hash: string
  file_name: string
  file_size: number
  download_status: DownloadStatus
  downloader: string
  created_at: string
  updated_at: string
  err_msg: string | null
  retry_count: number
}

// TMDB 元数据信息
export interface TMDBMetadata {
  id: number
  name: string
  poster_image_url: string | null
  air_date: string | null
  seasons: TMDBSeason[]
  description: string | null
  kind: BgmKind
}

// TMDB 季度信息
export interface TMDBSeason {
  number: number
  name: string
  air_date: string | null
  ep_count: number
}

// 番剧类型
export enum BgmKind {
  Anime = 'anime',
  Movie = 'movie'
}

// 更新番剧元数据参数
export interface UpdateMDBParams {
  bangumi_id: number
  kind: BgmKind
  tmdb_id?: number | null
  mikan_id?: number | null
  bangumi_tv_id?: number | null
  season_number?: number | null
}

// Metrics 相关类型定义
export enum WorkerState {
  Collecting = 'Collecting',
  Idle = 'Idle'
}

export interface WorkerMetrics {
  name: string
  state: WorkerState
  last_collection_time: string | null
}

export interface SchedulerMetrics {
  workers: WorkerMetrics[]
}

export interface DownloaderMetrics {
  num_of_tasks: number
}

export interface ProcessMetrics {
  used: number
  run_time_sec: number
}

export interface ServiceStatus {
  success: boolean
  error: string | null
}

export interface ServiceMetrics {
  name: string
  status: ServiceStatus
}

export interface MetadataMetrics {
  services: ServiceMetrics[]
  last_refresh_time: number
}

export interface Metrics {
  downloader: DownloaderMetrics
  scheduler: SchedulerMetrics
  process: ProcessMetrics
  metadata: MetadataMetrics
}

// 配置相关类型定义
export interface ProxyConfig {
  enabled: boolean
  http: string
  https: string
  no_proxy: string[]
}

export interface SentryConfig {
  enabled: boolean
  dsn: string
}

export interface TelegramConfig {
  enabled: boolean
  token: string
  chat_id: string
}

export interface NotifyConfig {
  telegram: TelegramConfig
}

// 通用下载器配置
export interface GenericDownloaderConfig {
  download_dir: string
  max_retry_count: number
  retry_min_interval: string
  retry_max_interval: string
  download_timeout: string
  delete_task_on_completion: boolean
  priority: number
}

// 115网盘下载器配置
export interface Pan115Config {
  enabled: boolean
  cookies: string
  max_requests_per_second: number
  download_dir: string
  max_retry_count: number
  retry_min_interval: string
  retry_max_interval: string
  download_timeout: string
  delete_task_on_completion: boolean
  priority: number
}

// qBittorrent下载器配置
export interface QbittorrentConfig {
  enabled: boolean
  url: string
  username: string
  password: string
  download_dir: string
  max_retry_count: number
  retry_min_interval: string
  retry_max_interval: string
  download_timeout: string
  delete_task_on_completion: boolean
  priority: number
  mount_path?: string
}

// Transmission下载器配置
export interface TransmissionConfig {
  enabled: boolean
  url: string
  username: string
  password: string
  download_dir: string
  max_retry_count: number
  retry_min_interval: string
  retry_max_interval: string
  download_timeout: string
  delete_task_on_completion: boolean
  priority: number
  mount_path?: string
}

// 下载器配置
export interface DownloaderConfig {
  pan115: Pan115Config
  qbittorrent: QbittorrentConfig
  transmission: TransmissionConfig
}

export interface SiliconflowConfig {
  enabled: boolean
  api_key: string
  model: string
  base_url: string
}

export interface DeepseekConfig {
  enabled: boolean
  api_key: string
  model: string
  base_url: string
}

export interface DeepbricksConfig {
  enabled: boolean
  api_key: string
  model: string
  base_url: string
}

export interface RawParserConfig {
  enabled: boolean
}

export interface ParserConfig {
  siliconflow: SiliconflowConfig
  deepseek: DeepseekConfig
  deepbricks: DeepbricksConfig
  raw: RawParserConfig
}

// 日志级别枚举
export enum LogLevel {
  Error = 'error',
  Warn = 'warn',
  Info = 'info',
  Debug = 'debug',
  Trace = 'trace'
}

export interface LogConfig {
  level: LogLevel
}

export interface ServerConfig {
  assets_path: string
  listen_addr: string
  database_url: string
}

export interface MikanConfig {
  endpoint: string
}

export interface TMDBConfig {
  api_key: string
  base_url: string
  image_base_url: string
  language: string
}

export interface BangumiTvConfig {
  endpoint: string
  image_base_url: string
}

export interface Config {
  log: LogConfig
  server: ServerConfig
  mikan: MikanConfig
  bangumi_tv: BangumiTvConfig
  tmdb: TMDBConfig
  parser: ParserConfig
  downloader: DownloaderConfig
  notify: NotifyConfig
  proxy: ProxyConfig
  sentry: SentryConfig
}

// 番剧查询参数
export interface QueryBangumiParams {
  name?: string
  offset: number
  limit: number
  status?: SubscribeStatus
  calendar_season?: string
}

export interface MikanSearchResultItem {
  id: number
  title: string
  image_url: string,
  bangumi_tv_id: number
}

export interface AddBangumiParams {
  title: string
  mikan_id: number
  bgm_tv_id?: number | null
  tmdb_id?: number | null
}

export enum FileType {
  Video = 'Video',
  Subtitle = 'Subtitle',
  Unknown = 'Unknown'
}

export interface DownloadedFile {
  file_id: string
  file_name: string
  file_size: number
  file_type: string
}

export interface DownloaderInfo {
  name: string
  priority: number
}
