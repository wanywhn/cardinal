import React from 'react';
import ReactDOM from 'react-dom/client';
import './i18n/config';
import App from './App';
import { initializeAppMenu } from './menu';
import { initializeTray } from './tray';
import { initializeGlobalShortcuts } from './utils/globalShortcuts';
import { initializeThemePreference } from './theme';

initializeThemePreference();
void initializeTray();
void initializeGlobalShortcuts();
void initializeAppMenu();

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('Unable to initialize application: #root element is missing.');
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
