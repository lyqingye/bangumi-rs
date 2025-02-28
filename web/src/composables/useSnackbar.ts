import { useSnackbarStore, type SnackbarOptions } from '@/stores/snackbar'

export const useSnackbar = () => {
  const { state, showSnackbar } = useSnackbarStore()

  return {
    state,
    showSnackbar
  }
}
