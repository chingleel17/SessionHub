#[cfg(target_os = "windows")]
pub(crate) const CREATE_NEW_CONSOLE: u32 = 0x00000010;

#[cfg(target_os = "windows")]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x08000000;

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_false() -> bool {
    false
}

mod analytics;
mod claude;
mod misc;
mod opencode;
mod provider_integration;
mod quota;
mod session;
mod settings;
mod sisyphus_openspec;

pub(crate) use analytics::*;
pub(crate) use claude::*;
pub(crate) use misc::*;
pub(crate) use opencode::*;
pub(crate) use provider_integration::*;
pub(crate) use quota::*;
pub(crate) use session::*;
pub(crate) use settings::*;
pub(crate) use sisyphus_openspec::*;
