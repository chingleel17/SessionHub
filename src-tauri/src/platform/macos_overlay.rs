use std::ffi::c_void;
use std::sync::LazyLock;

use objc2::ffi::object_setClass;
use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
use objc2::{msg_send, sel, ClassType};
use objc2_app_kit::{
    NSPanel, NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};

extern "C-unwind" fn cannot_become_key(_this: &AnyObject, _sel: Sel) -> Bool {
    Bool::NO
}

/// 非啟動型 NSPanel 子類別：不搶焦點，才能附著在其他 App 的全螢幕 Space 上。
static OVERLAY_PANEL_CLASS: LazyLock<&'static AnyClass> = LazyLock::new(|| {
    let mut builder = ClassBuilder::new(c"SessionHubOverlayPanel", NSPanel::class())
        .expect("SessionHubOverlayPanel 類別重複註冊");
    // SAFETY: 方法簽章與 NSWindow 的 canBecomeKeyWindow / canBecomeMainWindow 一致。
    unsafe {
        builder.add_method(
            sel!(canBecomeKeyWindow),
            cannot_become_key as extern "C-unwind" fn(_, _) -> _,
        );
        builder.add_method(
            sel!(canBecomeMainWindow),
            cannot_become_key as extern "C-unwind" fn(_, _) -> _,
        );
    }
    builder.register()
});

/// 讓視窗能顯示在其他 App 的全螢幕 Space 之上。
///
/// 全螢幕 App 會獨佔一個 Space，一般 NSWindow（即使 always_on_top）不會出現在其中。
/// 必須是 NSPanel + NonactivatingPanel，並搭配 `CanJoinAllSpaces`、`FullScreenAuxiliary`
/// 與 status window 層級。須在主執行緒呼叫，可重複呼叫。
pub fn allow_over_fullscreen(ns_window: *mut c_void) {
    if ns_window.is_null() {
        return;
    }
    let object = ns_window as *mut AnyObject;
    // SAFETY: 指標來自 Tauri 的 `ns_window()`，視窗存活期間有效；此函式於主執行緒執行。
    // 僅換成 NSPanel 子類別，不新增 ivar，tao 的 `focusable` ivar 對應的方法已被覆寫。
    unsafe {
        let panel_class: &AnyClass = *OVERLAY_PANEL_CLASS;
        let current: &AnyClass = (*object).class();
        if !std::ptr::eq(current, panel_class) {
            object_setClass(object.cast(), (panel_class as *const AnyClass).cast());
        }

        let window: &NSWindow = &*(ns_window as *const NSWindow);
        window.setStyleMask(window.styleMask() | NSWindowStyleMask::NonactivatingPanel);
        let panel: &NSPanel = &*(ns_window as *const NSPanel);
        panel.setFloatingPanel(true);
        panel.setHidesOnDeactivate(false);
        panel.setWorksWhenModal(true);
        // 讓 WindowServer 也記錄為不啟動 App（私有 API，缺少時略過）
        let responds: Bool = msg_send![object, respondsToSelector: sel!(_setPreventsActivation:)];
        if responds.as_bool() {
            let _: () = msg_send![object, _setPreventsActivation: Bool::YES];
        }
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
        window.setLevel(NSStatusWindowLevel);
    }
}
