use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
    ShowWindow, SW_RESTORE,
};

pub struct FocusState {
    pub target: String,
    pub found: HWND,
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam as usize as *mut FocusState);
    if IsWindowVisible(hwnd) == 0 {
        return TRUE;
    }

    let mut buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    let title = OsString::from_wide(&buf[..len as usize])
        .to_string_lossy()
        .to_lowercase();

    let mut cls_buf = [0u16; 256];
    let cls_len = GetClassNameW(hwnd, cls_buf.as_mut_ptr(), cls_buf.len() as i32);
    let class = OsString::from_wide(&cls_buf[..cls_len as usize])
        .to_string_lossy()
        .to_lowercase();

    let is_terminal =
        class.contains("windowsterminal") || class.contains("pseudoconsolewindow");

    let target_lower = state.target.to_lowercase();
    if is_terminal && title.contains(&target_lower) {
        state.found = hwnd;
        return 0;
    }

    TRUE
}

/// 依標題關鍵字尋找 Windows Terminal 並帶到前景
pub fn focus_window_by_title(title_hint: &str) -> Result<(), String> {
    let mut state = FocusState {
        target: title_hint.to_string(),
        found: std::ptr::null_mut(),
    };
    unsafe {
        EnumWindows(Some(enum_proc), &mut state as *mut FocusState as usize as LPARAM);
        if !state.found.is_null() {
            ShowWindow(state.found, SW_RESTORE);
            SetForegroundWindow(state.found);
            Ok(())
        } else {
            Err("Terminal window not found".to_string())
        }
    }
}
