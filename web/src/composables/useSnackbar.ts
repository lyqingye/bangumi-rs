import { useSnackbarStore } from '../stores/snackbar'
export type { SnackbarOptions } from '../stores/snackbar'

export function useSnackbar() {
  const { showSnackbar } = useSnackbarStore()
  return {
    showSnackbar
  }
}
