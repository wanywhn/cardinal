import { invoke } from '@tauri-apps/api/core';
import { register } from '@tauri-apps/plugin-global-shortcut';

const QUICK_LAUNCH_SHORTCUT = 'CommandOrControl+Shift+Space';
const HIDE_WINDOW_SHORTCUT = 'Escape';

export async function initializeGlobalShortcuts() {
  try {
    await register(QUICK_LAUNCH_SHORTCUT, (event) => {
      if (event.state === 'Released') {
        invoke('toggle_main_window');
      }
    });
    await register(HIDE_WINDOW_SHORTCUT, (event) => {
      if (event.state === 'Released') {
        invoke('hide_main_window');
      }
    });
  } catch (error) {
    console.error('Failed to register global shortcuts', error);
  }
}
