import { ref } from 'vue'

interface SnackbarOptions {
  text: string
  color?: string
  timeout?: number
  location?: string
}

const snackbarRef = ref()

export function useSnackbar() {
  const showSnackbar = (options: SnackbarOptions) => {
    if (snackbarRef.value) {
      snackbarRef.value.showSnackbar(options)
    }
  }

  return {
    snackbarRef,
    showSnackbar
  }
}
