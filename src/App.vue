<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface MouseState {
  position: [number, number];
  buttons: boolean[];
  element_size: [number, number] | null;
}

const mouseState = ref<MouseState>({
  position: [0, 0],
  buttons: [false, false, false],
  element_size: null
});

// Function to update mouse state
async function updateMouseState() {
  const state = await invoke<MouseState>("get_mouse_state");
  mouseState.value = state;
}

// Function to handle clicks and update element size
async function handleClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  const width = target.offsetWidth;
  const height = target.offsetHeight;
  await invoke("update_element_size", { width, height });
}

// Set up polling interval
let intervalId: number | null = null;

onMounted(() => {
  // Update mouse state every 16ms (approximately 60fps)
  intervalId = window.setInterval(updateMouseState, 16);
  // Add click listener to the whole document
  document.addEventListener('click', handleClick);
});

onUnmounted(() => {
  if (intervalId !== null) {
    clearInterval(intervalId);
  }
  document.removeEventListener('click', handleClick);
});

const getButtonName = (index: number) => {
  switch (index) {
    case 0: return "Left";
    case 1: return "Right";
    case 2: return "Middle";
    default: return `Button ${index}`;
  }
};
</script>

<template>
  <main class="container">
    <h1>Mouse Tracker</h1>
    <div class="mouse-info">
      <div class="mouse-position">
        Position: ({{ mouseState.position[0] }}, {{ mouseState.position[1] }})
      </div>
      <div class="mouse-buttons">
        <div v-for="(pressed, index) in mouseState.buttons" 
             :key="index"
             :class="['button-state', { active: pressed }]">
          {{ getButtonName(index) }}
        </div>
      </div>
      <div v-if="mouseState.element_size" class="element-size">
        Clicked Element Size: {{ mouseState.element_size[0] }}px × {{ mouseState.element_size[1] }}px
      </div>
    </div>
  </main>
</template>

<style scoped>
.mouse-info {
  font-size: 1.2em;
  margin-top: 2em;
  padding: 1em;
  background-color: #f0f0f0;
  border-radius: 8px;
}

.mouse-buttons {
  display: flex;
  gap: 1em;
  margin-top: 1em;
  justify-content: center;
}

.button-state {
  padding: 0.5em 1em;
  border-radius: 4px;
  background-color: #ddd;
  transition: background-color 0.2s;
}

.button-state.active {
  background-color: #4CAF50;
  color: white;
}

.element-size {
  margin-top: 1em;
  text-align: center;
  color: #666;
}
</style>