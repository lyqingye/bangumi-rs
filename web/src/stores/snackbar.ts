import { reactive } from 'vue'

export interface SnackbarOptions {
  text: string
  color?: string
  timeout?: number
  location?: 'top' | 'top right' | 'top left' | 'bottom' | 'bottom right' | 'bottom left'
}

export interface SnackbarState {
  show: boolean
  text: string
  color: string
  timeout: number
  location: 'top' | 'top right' | 'top left' | 'bottom' | 'bottom right' | 'bottom left'
}

const state = reactive<SnackbarState>({
  show: false,
  text: '',
  color: 'success',
  timeout: 3000,
  location: 'top right'
})

export const useSnackbarStore = () => {
  const showSnackbar = (options: SnackbarOptions) => {
    state.text = options.text
    state.color = options.color || 'success'
    state.timeout = options.timeout || 3000
    state.location = options.location || 'top right'
    state.show = true
  }

  return {
    state,
    showSnackbar
  }
} 