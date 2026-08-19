use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

/// 將視窗重新插回 topmost Z-order 的最前端。
///
/// Windows 上 topmost 樣式會被其他程式的 `SetForegroundWindow`、全螢幕應用或
/// explorer 重啟擠到後面，且不會自動恢復；此函式以 `SWP_NOACTIVATE` 重申，
/// 不搶焦點、不改變位置與尺寸，可安全地週期性呼叫。
pub fn reassert_topmost(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    unsafe {
        SetWindowPos(
            hwnd as HWND,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}
