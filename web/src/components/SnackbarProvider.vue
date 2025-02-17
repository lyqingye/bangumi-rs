<script setup lang="ts">
import { useSnackbarStore } from '../stores/snackbar'

const { state } = useSnackbarStore()
</script>

<template>
  <v-snackbar v-model="state.show" :color="state.color" :timeout="state.timeout" :location="state.location">
    {{ state.text }}
  </v-snackbar>
</template>
