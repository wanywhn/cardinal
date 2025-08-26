import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * 统一的上下文菜单 hook，处理文件和头部两种类型的菜单
 */
export function useUnifiedContextMenu(autoFitColumns) {
  const [contextMenu, setContextMenu] = useState({ visible: false, x: 0, y: 0 });
  const [headerContextMenu, setHeaderContextMenu] = useState({ visible: false, x: 0, y: 0 });
  const [contextPath, setContextPath] = useState(null);

  // 文件右键菜单
  const showContextMenu = useCallback((e, path) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ visible: true, x: e.clientX, y: e.clientY });
    setContextPath(path);
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, visible: false }));
    setContextPath(null);
  }, []);

  // 头部右键菜单
  const showHeaderContextMenu = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    setHeaderContextMenu({ visible: true, x: e.clientX, y: e.clientY });
  }, []);

  const closeHeaderContextMenu = useCallback(() => {
    setHeaderContextMenu(prev => ({ ...prev, visible: false }));
  }, []);

  // 文件菜单项
  const menuItems = [
    {
      label: 'Open in Finder',
      action: () => {
        if (contextPath) {
          invoke('open_in_finder', { path: contextPath });
        }
      },
    },
  ];

  // 头部菜单项
  const headerMenuItems = [
    {
      label: 'Reset',
      action: autoFitColumns,
    },
  ];

  return {
    // 文件菜单
    contextMenu,
    showContextMenu,
    closeContextMenu,
    menuItems,
    // 头部菜单
    headerContextMenu,
    showHeaderContextMenu,
    closeHeaderContextMenu,
    headerMenuItems,
  };
}
