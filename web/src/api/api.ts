import axios, { AxiosError } from 'axios'
import type { AxiosResponse } from 'axios'
import type {
  ApiResponse,
  Bangumi,
  Episode,
  Torrent,
  SubscribeStatus,
  SubscribeParams,
  DownloadTask,
  QueryDownloadTask,
  PaginatedResponse,
  TMDBMetadata,
  UpdateMDBParams,
  Metrics,
  Config,
  QueryBangumiParams
} from './model'
import { ApiError } from './model'
import { useSnackbar } from '../composables/useSnackbar'

const { showSnackbar } = useSnackbar()

// 创建 axios 实例
const api = axios.create({
  baseURL: '/api',
  timeout: 30000
})

// 统一的错误处理函数
function handleError(error: unknown, defaultMessage: string): never {
  let errorMessage = defaultMessage
  let errorCode = -1
  let statusCode: number | undefined

  if (error instanceof AxiosError) {
    // 处理 Axios 错误
    statusCode = error.response?.status
    const apiError = error.response?.data as ApiResponse<unknown>
    errorMessage = apiError?.msg || error.message || defaultMessage
    errorCode = apiError?.code || -1
  } else if (error instanceof Error) {
    errorMessage = error.message || defaultMessage
  }

  // 显示错误提示
  showSnackbar({
    text: errorMessage,
    color: 'error',
    location: 'top right',
    timeout: 3000
  })

  throw new ApiError(errorMessage, errorCode, statusCode)
}

// 统一的响应处理函数
function handleResponse<T>(response: AxiosResponse<ApiResponse<T>>, defaultError: string): T {
  const { code, msg, data } = response.data

  if (code === 0) {
    return data
  }

  throw new ApiError(msg || defaultError, code)
}

// 日历相关 API
export async function fetchCalendar(): Promise<Bangumi[]> {
  try {
    const response = await api.get<ApiResponse<Bangumi[]>>('/calendar')
    return handleResponse(response, '获取日历数据失败')
  } catch (error) {
    return handleError(error, '获取日历数据失败')
  }
}

export async function fetchCalendarSeason(): Promise<string> {
  try {
    const response = await api.get<ApiResponse<string>>('/calendar/season')
    return handleResponse(response, '获取日历季节数据失败')
  } catch (error) {
    return handleError(error, '获取日历季节数据失败')
  }
}


// 番剧相关 API
export async function getBangumiById(id: number): Promise<Bangumi> {
  try {
    const response = await api.get<ApiResponse<Bangumi>>(`/bangumi/${id}`)
    return handleResponse(response, '获取番剧详情失败')
  } catch (error) {
    return handleError(error, '获取番剧详情失败')
  }
}

// 订阅相关 API
export async function subscribeBangumi(id: number, params: SubscribeParams): Promise<void> {
  try {
    const response = await api.post<ApiResponse<null>>(`/bangumi/${id}/subscribe`, params)
    handleResponse(response, '订阅操作失败')
  } catch (error) {
    handleError(error, '订阅操作失败')
  }
}

// 剧集相关 API
export async function getBangumiEpisodes(id: number): Promise<Episode[]> {
  try {
    const response = await api.get<ApiResponse<Episode[]>>(`/bangumi/${id}/episodes`)
    return handleResponse(response, '获取剧集列表失败')
  } catch (error) {
    return handleError(error, '获取剧集列表失败')
  }
}

// 种子相关 API
export async function getBangumiTorrents(id: number): Promise<Torrent[]> {
  try {
    const response = await api.get<ApiResponse<Torrent[]>>(`/bangumi/${id}/torrents`)
    return handleResponse(response, '获取种子列表失败')
  } catch (error) {
    return handleError(error, '获取种子列表失败')
  }
}

// 刷新番剧元数据
export async function refreshBangumi(id: number, force: boolean): Promise<void> {
  try {
    const response = await api.get<ApiResponse<null>>(`/bangumi/${id}/refresh/${force}`)
    handleResponse(response, '刷新元数据失败')
  } catch (error) {
    handleError(error, '刷新元数据失败')
  }
}

// 删除番剧下载任务
export async function deleteBangumiDownloadTasks(id: number): Promise<void> {
  try {
    const response = await api.get<ApiResponse<null>>(`/bangumi/${id}/delete_download_tasks`)
    handleResponse(response, '删除下载任务失败')
  } catch (error) {
    handleError(error, '删除下载任务失败')
  }
}

export async function manualSelectTorrent(bangumiId: number, episodeNumber: number, infoHash: string): Promise<void> {
  try {
    const response = await api.get<ApiResponse<null>>(`/bangumi/${bangumiId}/${episodeNumber}/manual_select_torrent/${infoHash}`)
    handleResponse(response, '手动选择种子下载失败')
  } catch (error) {
    handleError(error, '手动选择种子下载失败')
  }
}
export async function getOnlineWatchUrl(bangumiId: number, episodeId: number): Promise<string> {
  return  `${window.location.origin}/api/bangumi/${bangumiId}/${episodeId}/online_watch`
}

// 下载任务相关 API
export async function fetchDownloadTasks(params: QueryDownloadTask): Promise<DownloadTask[]> {
  try {
    const response = await api.post<ApiResponse<DownloadTask[]>>('/downloads', params)
    return handleResponse(response, '获取下载任务列表失败')
  } catch (error) {
    return handleError(error, '获取下载任务列表失败')
  }
}

// 刷新放送列表
export async function refreshCalendar(): Promise<void> {
  try {
    const response = await api.get<ApiResponse<null>>('/calendar/refresh')
    handleResponse(response, '刷新放送列表失败')
  } catch (error) {
    handleError(error, '刷新放送列表失败')
  }
}

// TMDB 搜索相关 API
export async function searchBangumiAtTMDB(name: string): Promise<TMDBMetadata[]> {
  try {
    const response = await api.get<ApiResponse<TMDBMetadata[]>>(`/tmdb/search/${encodeURIComponent(name)}`)
    return handleResponse(response, '搜索 TMDB 数据失败')
  } catch (error) {
    return handleError(error, '搜索 TMDB 数据失败')
  }
}

// 更新番剧元数据
export async function updateBangumiMDB(params: UpdateMDBParams): Promise<void> {
  try {
    const response = await api.post<ApiResponse<null>>(`/bangumi/${params.bangumi_id}/mdb/update`, params)
    handleResponse(response, '更新番剧元数据失败')
  } catch (error) {
    handleError(error, '更新番剧元数据失败')
  }
}

// 获取系统指标
export async function fetchMetrics(): Promise<Metrics> {
  try {
    const response = await api.get<ApiResponse<Metrics>>('/metrics')
    return handleResponse(response, '获取系统指标失败')
  } catch (error) {
    return handleError(error, '获取系统指标失败')
  }
}

// 配置相关 API
export async function getConfig(): Promise<Config> {
  try {
    const response = await api.get<ApiResponse<Config>>('/config')
    return handleResponse(response, '获取配置失败')
  } catch (error) {
    return handleError(error, '获取配置失败')
  }
}

export async function updateConfig(config: Config): Promise<void> {
  try {
    const response = await api.post<ApiResponse<null>>('/config', config)
    handleResponse(response, '更新配置失败')
  } catch (error) {
    handleError(error, '更新配置失败')
  }
}

// 番剧列表查询 API
export async function fetchBangumiList(params: QueryBangumiParams): Promise<PaginatedResponse<Bangumi>> {
  try {
    const response = await api.post<ApiResponse<PaginatedResponse<Bangumi>>>('/bangumi/list', params)
    return handleResponse(response, '获取番剧列表失败')
  } catch (error) {
    return handleError(error, '获取番剧列表失败')
  }
}
