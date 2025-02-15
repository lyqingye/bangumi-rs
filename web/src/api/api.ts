import axios, { AxiosError } from 'axios'
import type { AxiosResponse } from 'axios'
import type { ApiResponse, Bangumi, Episode, Torrent, SubscribeStatus, SubscribeParams } from './model'
import { ApiError } from './model'
import { useSnackbar } from '../composables/useSnackbar'

const { showSnackbar } = useSnackbar()

// 创建 axios 实例
const api = axios.create({
  baseURL: 'http://localhost:3001/api',
  timeout: 5000
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
export async function refreshBangumi(id: number): Promise<void> {
  try {
    const response = await api.get<ApiResponse<null>>(`/bangumi/${id}/refresh`)
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

export async function getOnlineWatchUrl(bangumiId: number, episodeId: number): Promise<string> {
    return api.defaults.baseURL + `/bangumi/${bangumiId}/${episodeId}/online_watch`
}