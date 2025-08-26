import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// 简化的上下文菜单 hook，同时处理文件和头部菜单
export function useContextMenu(autoFitColumns = null) {
  const [contextMenu, setContextMenu] = useState({ 
    visible: false, 
    x: 0, 
    y: 0, 
    type: null,
    data: null 
  });
  
  const [headerContextMenu, setHeaderContextMenu] = useState({ 
    visible: false, 
    x: 0, 
    y: 0 
  });

  // 文件上下文菜单
  const showContextMenu = useCallback((e, path) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ 
      visible: true, 
      x: e.clientX, 
      y: e.clientY, 
      type: 'file',
      data: path
    });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, visible: false }));
  }, []);

  // 头部上下文菜单
  const showHeaderContextMenu = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    setHeaderContextMenu({ 
      visible: true, 
      x: e.clientX, 
      y: e.clientY 
    });
  }, []);

  const closeHeaderContextMenu = useCallback(() => {
    setHeaderContextMenu(prev => ({ ...prev, visible: false }));
  }, []);

  // 文件菜单项
  const menuItems = contextMenu.type === 'file' ? [
    {
      label: 'Open in Finder',
      action: () => invoke('open_in_finder', { path: contextMenu.data }),
    },
  ] : [];

  // 头部菜单项
  const headerMenuItems = [
    {
      label: 'Reset Column Widths',
      action: () => {
        if (autoFitColumns) autoFitColumns();
      },
    },
  ];

  return {
    contextMenu,
    showContextMenu,
    closeContextMenu,
    menuItems,
    headerContextMenu,
    showHeaderContextMenu,
    closeHeaderContextMenu,
    headerMenuItems
  };
}
