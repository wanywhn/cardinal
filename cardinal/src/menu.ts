import { getName } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
import { openUrl } from '@tauri-apps/plugin-opener';

const HELP_UPDATES_URL = 'https://github.com/cardisoft/cardinal/releases';

let menuInitPromise: Promise<void> | null = null;

export function initializeAppMenu(): Promise<void> {
  if (!menuInitPromise) {
    menuInitPromise = buildAppMenu().catch((error) => {
      console.error('Failed to initialize app menu', error);
      menuInitPromise = null;
    });
  }

  return menuInitPromise;
}

async function buildAppMenu(): Promise<void> {
  const name = (await getName().catch(() => null)) ?? 'Cardinal';
  const aboutItem = await PredefinedMenuItem.new({
    item: { About: null },
    text: `About ${name}`,
  });
  const preferencesItem = await MenuItem.new({
    id: 'menu.preferences',
    text: 'Preference',
    accelerator: 'CmdOrCtrl+,',
    action: () => {},
  });
  const hideItem = await MenuItem.new({
    id: 'menu.hide',
    text: 'Hide',
    accelerator: 'Esc',
    action: () => {
      void invoke('hide_main_window');
    },
  });
  const appSubmenu = await Submenu.new({
    id: 'menu.application',
    text: name,
    items: [
      aboutItem,
      await PredefinedMenuItem.new({ item: 'Separator' }),
      preferencesItem,
      hideItem,
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Quit' }),
    ],
  });

  const getUpdatesItem = await MenuItem.new({
    id: 'menu.help_updates',
    text: 'Get Updates',
    action: () => void openUpdatesPage(),
  });
  const helpSubmenu = await Submenu.new({
    id: 'menu.help-root',
    text: 'Help',
    items: [getUpdatesItem],
  });

  await helpSubmenu.setAsHelpMenuForNSApp().catch(() => {});

  const menu = await Menu.new({
    items: [appSubmenu, helpSubmenu],
  });
  await menu.setAsAppMenu();
}

async function openUpdatesPage(): Promise<void> {
  try {
    await openUrl(HELP_UPDATES_URL);
  } catch (error) {
    console.error('Failed to open updates page', error);
  }
}
