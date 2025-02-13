<template>
  <v-app>
    <v-main>
      <router-view />
    </v-main>
    <SnackbarProvider ref="snackbarRef" />
  </v-app>
</template>

<script lang="ts" setup>
import { useSnackbar } from './composables/useSnackbar'
import SnackbarProvider from './components/SnackbarProvider.vue'

const { snackbarRef } = useSnackbar()
</script>
