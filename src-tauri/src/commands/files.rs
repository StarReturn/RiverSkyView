use std::path::PathBuf;
use tauri::State;

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::files::*;
use crate::services::file_service::FileService;
use crate::services::project_service::ProjectService;
use crate::state::AppState;

#[tauri::command]
pub async fn list_directory(
    state: State<'_, AppState>,
    project_id: String,
    relative_dir: Option<String>,
) -> AppResult<Vec<FileNode>> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);

    if !project_path.exists() {
        return Err(AppError::new(ErrorCode::PathNotFound, "项目目录不存在"));
    }

    let settings = state.settings.read().await;
    let service = FileService::new(
        settings.settings.show_hidden_files,
        settings.settings.markdown_max_size_mb,
        settings.settings.image_max_size_mb,
    );

    let rel_dir = relative_dir.unwrap_or_default();
    service.list_directory(&project_path, &rel_dir)
}

#[tauri::command]
pub async fn read_file_for_preview(
    state: State<'_, AppState>,
    project_id: String,
    relative_path: String,
) -> AppResult<FilePreview> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);

    let settings = state.settings.read().await;
    let service = FileService::new(
        settings.settings.show_hidden_files,
        settings.settings.markdown_max_size_mb,
        settings.settings.image_max_size_mb,
    );

    service.read_file_for_preview(&project_path, &relative_path)
}

#[tauri::command]
pub async fn get_absolute_path(
    state: State<'_, AppState>,
    project_id: String,
    relative_path: String,
) -> AppResult<String> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);
    FileService::get_absolute_path(&project_path, &relative_path)
}

#[tauri::command]
pub async fn open_in_explorer(
    state: State<'_, AppState>,
    project_id: String,
    relative_path: Option<String>,
) -> AppResult<()> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);

    let path = match relative_path {
        Some(rel) if !rel.is_empty() => {
            crate::security::path_guard::PathGuard::resolve_relative(&project_path, &rel)?
        }
        _ => project_path,
    };

    FileService::open_in_explorer(&path)
}

#[tauri::command]
pub async fn reveal_in_explorer(
    state: State<'_, AppState>,
    project_id: String,
    relative_path: String,
) -> AppResult<()> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);
    let path =
        crate::security::path_guard::PathGuard::resolve_relative(&project_path, &relative_path)?;
    FileService::reveal_in_explorer(&path)
}

#[tauri::command]
pub async fn copy_file_to_clipboard(
    state: State<'_, AppState>,
    project_id: String,
    relative_path: String,
) -> AppResult<()> {
    let project = ProjectService::get(&state.db, &project_id).await?;
    let project_path = PathBuf::from(&project.path);
    let path =
        crate::security::path_guard::PathGuard::resolve_relative(&project_path, &relative_path)?;

    if !path.exists() {
        return Err(AppError::new(ErrorCode::FileNotFound, "文件不存在"));
    }

    // 使用 Windows 文件剪贴板 API 复制文件（直接 FFI）
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        // FFI 声明
        extern "system" {
            fn GlobalAlloc(flags: u32, size: usize) -> *mut core::ffi::c_void;
            fn GlobalLock(hmem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
            fn GlobalUnlock(hmem: *mut core::ffi::c_void) -> i32;
            fn GlobalFree(hmem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
            fn OpenClipboard(hwnd: *mut core::ffi::c_void) -> i32;
            fn EmptyClipboard() -> i32;
            fn CloseClipboard() -> i32;
            fn SetClipboardData(
                format: u32,
                hmem: *mut core::ffi::c_void,
            ) -> *mut core::ffi::c_void;
        }

        const GMEM_MOVEABLE: u32 = 0x0002;
        const CF_HDROP: u32 = 15;

        // 构建 DROPFILES + 宽字符路径
        let wide_path: Vec<u16> = std::ffi::OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();

        let byte_len = wide_path.len() * 2;
        let struct_size: usize = 20; // DROPFILES struct size
        let total_size = struct_size + byte_len;

        unsafe {
            let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size);
            if hglobal.is_null() {
                return Err(AppError::new(ErrorCode::Internal, "分配剪贴板内存失败"));
            }

            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                GlobalFree(hglobal);
                return Err(AppError::new(ErrorCode::Internal, "锁定剪贴板内存失败"));
            }

            let ptr_u8 = ptr as *mut u8;
            std::ptr::write_bytes(ptr_u8, 0, total_size);
            // DROPFILES.pFiles = 20 (offset to file list)
            *(ptr_u8 as *mut u32) = struct_size as u32;
            // DROPFILES.fWide = TRUE (offset 8, but it's at offset 16 in 64-bit due to padding)
            *(ptr_u8.add(8) as *mut i32) = 0; // pt.x = 0
            *(ptr_u8.add(12) as *mut i32) = 0; // pt.y = 0
            *(ptr_u8.add(16) as *mut i32) = 1; // fNC = FALSE, fWide = TRUE... wait

            // Actually DROPFILES layout:
            // 0:  pFiles (DWORD, 4 bytes) = offset to file list
            // 4:  pt.x (LONG, 4 bytes)
            // 8:  pt.y (LONG, 4 bytes)
            // 12: fNC (BOOL, 4 bytes)
            // 16: fWide (BOOL, 4 bytes)
            // Total: 20 bytes
            *(ptr_u8.add(4) as *mut i32) = 0; // pt.x
            *(ptr_u8.add(8) as *mut i32) = 0; // pt.y
            *(ptr_u8.add(12) as *mut i32) = 0; // fNC
            *(ptr_u8.add(16) as *mut i32) = 1; // fWide = TRUE

            // Copy path data after struct
            let data_ptr = ptr_u8.add(struct_size);
            std::ptr::copy_nonoverlapping(wide_path.as_ptr() as *const u8, data_ptr, byte_len);

            GlobalUnlock(hglobal);

            // 打开剪贴板并设置数据
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                GlobalFree(hglobal);
                return Err(AppError::new(ErrorCode::Internal, "无法打开剪贴板"));
            }
            EmptyClipboard();

            let result = SetClipboardData(CF_HDROP, hglobal);
            CloseClipboard();

            if result.is_null() {
                GlobalFree(hglobal);
                return Err(AppError::new(ErrorCode::Internal, "设置剪贴板数据失败"));
            }
            // SetClipboardData 成功后，系统拥有内存，不需要释放
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        Err(AppError::new(ErrorCode::Internal, "此功能仅支持 Windows"))
    }
}
