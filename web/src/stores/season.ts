import { reactive, computed } from 'vue'
import { fetchCalendarSeason } from '@/api/api'

// 定义选项类型
export interface SelectOption<T> {
  title: string
  value: T
}

// 季节状态接口
export interface SeasonState {
  selectedYear: number | null
  selectedSeason: string | null
  availableSeasons: string[]
  initialized: boolean
}

// 创建响应式状态
const state = reactive<SeasonState>({
  selectedYear: null,
  selectedSeason: null,
  availableSeasons: [],
  initialized: false
})

// 创建并导出store
export const useSeasonStore = () => {
  // 年份选项
  const yearOptions = computed<SelectOption<number | null>[]>(() => {
    const options: SelectOption<number | null>[] = [{ title: '全部年份', value: null }]
    
    // 获取当前年份
    const currentYear = new Date().getFullYear()
    
    // 从当前年份到2015年
    for (let year = currentYear; year >= 2015; year--) {
      options.push({
        title: `${year}年`,
        value: year
      })
    }
    
    return options
  })

  // 季节选项
  const seasonOptions = computed<SelectOption<string | null>[]>(() => {
    // 基础季节选项
    const baseOptions: SelectOption<string | null>[] = [
      { title: '冬季番组', value: '冬季番组' },
      { title: '春季番组', value: '春季番组' },
      { title: '夏季番组', value: '夏季番组' },
      { title: '秋季番组', value: '秋季番组' }
    ]
    
    // 如果没有选择具体年份，则只能选择"全部季节"
    if (state.selectedYear === null) {
      return [{ title: '全部季节', value: null }]
    }
    
    // 如果选择了具体年份，则只能选择具体季节
    return baseOptions
  })

  // 设置年份
  const setYear = (year: number | null) => {
    state.selectedYear = year
    
    // 如果选择了"全部年份"，则季节必须是"全部季节"
    if (year === null) {
      state.selectedSeason = null
    } 
    // 如果选择了具体年份，但季节是"全部季节"，则自动选择第一个具体季节
    else if (state.selectedSeason === null) {
      state.selectedSeason = '冬季番组'
    }
  }

  // 设置季节
  const setSeason = (season: string | null) => {
    state.selectedSeason = season
  }

  // 获取当前选择的季度值
  const getCalendarSeason = (): string | undefined => {
    if (!state.selectedYear && !state.selectedSeason) return undefined
    if (!state.selectedYear) return state.selectedSeason || undefined
    if (!state.selectedSeason) return String(state.selectedYear)
    return `${state.selectedYear} ${state.selectedSeason}`
  }

  // 初始化季节信息
  const initializeSeasonInfo = async () => {
    if (state.initialized) return

    try {
      const latestSeason = await fetchCalendarSeason()
      if (latestSeason) {
        // 保存可用的季度信息
        state.availableSeasons = [latestSeason]
        
        // 解析季度信息，格式如：2025 冬季番组
        const parts = latestSeason.split(' ')
        if (parts.length === 2) {
          const year = parseInt(parts[0])
          const season = parts[1]
          // 设置年份和季节选择
          state.selectedYear = year
          state.selectedSeason = season
        } else {
          // 如果没有获取到完整的季度信息，则使用当前年份和默认季节
          state.selectedYear = new Date().getFullYear()
          state.selectedSeason = '冬季番组'
        }
      } else {
        // 如果没有获取到季度信息，则使用当前年份和默认季节
        state.selectedYear = new Date().getFullYear()
        state.selectedSeason = '冬季番组'
      }
    } catch (e) {
      console.error('获取最新季度信息失败:', e)
      // 出错时使用当前年份和默认季节
      state.selectedYear = new Date().getFullYear()
      state.selectedSeason = '冬季番组'
    } finally {
      state.initialized = true
    }
  }

  return {
    state,
    yearOptions,
    seasonOptions,
    setYear,
    setSeason,
    getCalendarSeason,
    initializeSeasonInfo
  }
}

// 创建一个组合式函数，方便在组件中使用
export const useSeason = useSeasonStore 