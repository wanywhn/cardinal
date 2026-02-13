//! Linux 平台的文件预览功能实现
//! 由于 Linux 没有原生的 QuickLook API，我们使用外部工具或简单的文件信息显示

use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager};
use tracing::error;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxPreviewItemInput {
    pub path: String,
    pub rect: Option<ScreenRect>,
    pub transition_image: Option<String>,
}

// Linux 上的预览状态管理
static mut IS_PREVIEW_VISIBLE: bool = false;
static mut CURRENT_PREVIEW_ITEMS: Option<Vec<LinuxPreviewItemInput>> = None;

/// 尝试使用系统默认应用打开文件进行预览
pub fn toggle_preview_panel(app_handle: AppHandle, items: Vec<LinuxPreviewItemInput>) {
    unsafe {
        if IS_PREVIEW_VISIBLE {
            close_preview_panel(app_handle);
        } else {
            show_preview_panel(app_handle, items);
        }
    }
}

/// 显示预览面板 - 在 Linux 上使用系统默认应用打开文件
fn show_preview_panel(app_handle: AppHandle, items: Vec<LinuxPreviewItemInput>) {
    unsafe {
        IS_PREVIEW_VISIBLE = true;
        CURRENT_PREVIEW_ITEMS = Some(items.clone());
    }

    // 尝试使用系统默认应用打开第一个文件
    if let Some(first_item) = items.first() {
        if let Err(e) = open_file_with_default_app(&first_item.path) {
            error!("Failed to open file with default application: {e:?}");
            
            // 如果无法打开文件，则尝试显示文件信息
            show_file_info(&first_item.path);
        }
    }
}

/// 关闭预览面板
pub fn close_preview_panel(_app_handle: AppHandle) {
    unsafe {
        IS_PREVIEW_VISIBLE = false;
        CURRENT_PREVIEW_ITEMS = None;
    }
}

/// 更新预览面板内容
pub fn update_preview_panel(app_handle: AppHandle, items: Vec<LinuxPreviewItemInput>) {
    unsafe {
        if !IS_PREVIEW_VISIBLE {
            return;
        }
        
        CURRENT_PREVIEW_ITEMS = Some(items.clone());
    }

    // 对于 Linux，我们简单地重新打开第一个文件
    if let Some(first_item) = items.first() {
        if let Err(e) = open_file_with_default_app(&first_item.path) {
            error!("Failed to update preview: {e:?}");
        }
    }
}

/// 使用系统默认应用打开文件
fn open_file_with_default_app(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        // 尝试使用 xdg-open 命令
        Command::new("xdg-open").arg(file_path).spawn()?;
    }
    
    Ok(())
}

/// 显示文件信息作为备选方案
fn show_file_info(file_path: &str) {
    println!("File preview for: {}", file_path);
    
    // 获取文件的基本信息
    if let Ok(metadata) = std::fs::metadata(file_path) {
        println!("Size: {} bytes", metadata.len());
        println!("Modified: {:?}", metadata.modified());
        println!("Created: {:?}", metadata.created());
        println!("Type: {}", if metadata.is_dir() { "Directory" } else { "File" });
    } else {
        println!("Could not read file metadata");
    }
}

/// 检查当前是否有预览面板可见
pub fn is_preview_visible() -> bool {
    unsafe { IS_PREVIEW_VISIBLE }
}