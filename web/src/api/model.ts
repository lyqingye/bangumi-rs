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

// 订阅状态枚举
export enum SubscribeStatus {
  None = 'None',
  Subscribed = 'Subscribed',
  Downloaded = 'Downloaded',
}

// 下载状态枚举
export enum DownloadStatus {
  Pending = 'Pending',
  Downloading = 'Downloading',
  Completed = 'Completed',
  Failed = 'Failed',
}

// 下载状态枚举
export enum State {
  Missing = 'Missing',
  Ready = 'Ready',
  Downloading = 'Downloading',
  Downloaded = 'Downloaded',
  Failed = 'Failed',
  Retrying = 'Retrying',
}

// 订阅参数
export interface SubscribeParams {
  status: SubscribeStatus
  start_episode_number?: number
  resolution_filter?: string
  language_filter?: string
  release_group_filter?: string
  collector_interval?: number
  metadata_interval?: number
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
