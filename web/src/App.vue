<template>
  <v-app>
    <v-main>
      <router-view />
    </v-main>
    <SnackbarProvider />
  </v-app>
</template>

<script lang="ts" setup>
import SnackbarProvider from './components/SnackbarProvider.vue'
</script>
