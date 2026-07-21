//! 動態繪製系統匣圖示：依 quota 用量與顯示模式在底圖上疊加視覺指示。
//!
//! 為了可攜性與 KISS，Percentage 模式的數字採用內建 3x5 點陣手繪，不引入外部字型檔。

use image::{load_from_memory, Rgba, RgbaImage};
use tauri::{AppHandle, Manager};

use crate::types::{AppSettings, QuotaCache, QuotaSnapshot, TrayQuotaMode};

/// 系統匣圖示的固定 id，供執行期以 `app.tray_by_id()` 取回更新
pub(crate) const TRAY_ICON_ID: &str = "main-tray";

/// 底圖：32x32 的 SessionHub icon（與 lib.rs 系統匣初始圖示同源）
const BASE_ICON_BYTES: &[u8] = include_bytes!("../icons/32x32.png");

/// 綠：用量 < 50%
const COLOR_OK: Rgba<u8> = Rgba([46, 204, 113, 255]);
/// 黃：50% <= 用量 < 80%
const COLOR_WARN: Rgba<u8> = Rgba([241, 196, 15, 255]);
/// 紅：用量 >= 80%
const COLOR_DANGER: Rgba<u8> = Rgba([231, 76, 60, 255]);

/// 依用量百分比（0.0–1.0）回傳三段顏色
fn quota_color(pct: f64) -> Rgba<u8> {
    if pct >= 0.8 {
        COLOR_DANGER
    } else if pct >= 0.5 {
        COLOR_WARN
    } else {
        COLOR_OK
    }
}

/// 載入底圖為可編輯的 RGBA buffer；載入失敗時回傳全透明 32x32
fn base_icon() -> RgbaImage {
    match load_from_memory(BASE_ICON_BYTES) {
        Ok(img) => img.to_rgba8(),
        Err(_) => RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 0])),
    }
}

/// 將 src 以 alpha 疊到 dst 的 (x, y)（單像素 over 混合）
fn blend_pixel(dst: &mut RgbaImage, x: u32, y: u32, src: Rgba<u8>) {
    if x >= dst.width() || y >= dst.height() {
        return;
    }
    let a = src.0[3] as f64 / 255.0;
    if a <= 0.0 {
        return;
    }
    let bg = dst.get_pixel(x, y).0;
    let out = [
        (src.0[0] as f64 * a + bg[0] as f64 * (1.0 - a)).round() as u8,
        (src.0[1] as f64 * a + bg[1] as f64 * (1.0 - a)).round() as u8,
        (src.0[2] as f64 * a + bg[2] as f64 * (1.0 - a)).round() as u8,
        ((src.0[3] as f64) + (bg[3] as f64) * (1.0 - a))
            .round()
            .min(255.0) as u8,
    ];
    dst.put_pixel(x, y, Rgba(out));
}

/// 填滿實心矩形
fn fill_rect(img: &mut RgbaImage, x0: u32, y0: u32, w: u32, h: u32, color: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            blend_pixel(img, x0 + dx, y0 + dy, color);
        }
    }
}

/// 畫填滿圓（以整數距離近似）
fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 {
                    blend_pixel(img, x as u32, y as u32, color);
                }
            }
        }
    }
}

/// 3x5 點陣字形（0-9 與 `%`），每個字元 5 列、每列低 3 位表示 3 欄。
/// bit2=最左欄, bit0=最右欄。
fn glyph_rows(ch: char) -> [u8; 5] {
    match ch {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        _ => [0, 0, 0, 0, 0],
    }
}

/// 在 (x0, y0) 繪製單一 3x5 點陣字元，可放大 scale 倍
fn draw_glyph(img: &mut RgbaImage, ch: char, x0: u32, y0: u32, scale: u32, color: Rgba<u8>) {
    let rows = glyph_rows(ch);
    for (ry, row) in rows.iter().enumerate() {
        for cx in 0..3u32 {
            let bit = (row >> (2 - cx)) & 1;
            if bit == 1 {
                fill_rect(
                    img,
                    x0 + cx * scale,
                    y0 + ry as u32 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

/// 在圖示下緣置中繪製文字（僅支援 glyph_rows 定義的字元）
fn draw_text_bottom(img: &mut RgbaImage, text: &str, scale: u32, color: Rgba<u8>) {
    let glyph_w = 3 * scale;
    let gap = scale;
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return;
    }
    let total_w = chars.len() as u32 * glyph_w + (chars.len() as u32 - 1) * gap;
    let start_x = if total_w >= img.width() {
        0
    } else {
        (img.width() - total_w) / 2
    };
    let glyph_h = 5 * scale;
    let y0 = img.height().saturating_sub(glyph_h + scale);

    // 先鋪一層半透明深色底，提升小字在任意底圖上的可讀性
    fill_rect(
        img,
        start_x.saturating_sub(scale),
        y0.saturating_sub(scale),
        total_w + 2 * scale,
        glyph_h + 2 * scale,
        Rgba([0, 0, 0, 170]),
    );

    let mut x = start_x;
    for ch in chars {
        draw_glyph(img, ch, x, y0, scale, color);
        x += glyph_w + gap;
    }
}

/// 依 quota 用量與模式繪製系統匣圖示，回傳 PNG 位元組。
///
/// - `pct`：主要 provider 的用量比例（0.0–1.0）
/// - `mode`：顯示模式
pub(crate) fn render_tray_icon_png(pct: f64, mode: TrayQuotaMode) -> Vec<u8> {
    let clamped = pct.clamp(0.0, 1.0);
    let mut img = base_icon();
    let color = quota_color(clamped);

    match mode {
        TrayQuotaMode::Hidden => {}
        TrayQuotaMode::IconOnly => {
            // 右下角疊一個帶白邊的彩色圓點
            let w = img.width() as i32;
            let h = img.height() as i32;
            fill_circle(&mut img, w - 8, h - 8, 6, Rgba([255, 255, 255, 230]));
            fill_circle(&mut img, w - 8, h - 8, 4, color);
        }
        TrayQuotaMode::Bar => {
            // 底部彩色進度條（高 4px），底層鋪暗軌
            let w = img.width();
            let h = img.height();
            let bar_h = 4u32;
            let y0 = h - bar_h;
            fill_rect(&mut img, 0, y0, w, bar_h, Rgba([0, 0, 0, 140]));
            let fill_w = (w as f64 * clamped).round() as u32;
            if fill_w > 0 {
                fill_rect(&mut img, 0, y0, fill_w, bar_h, color);
            }
        }
        TrayQuotaMode::Percentage => {
            let pct_int = (clamped * 100.0).round() as u32;
            // 100% 時只顯示 "99+" 以免超出寬度；其餘顯示數字（不含 %，空間不足）
            let text = if pct_int >= 100 {
                "99".to_string()
            } else {
                format!("{pct_int}")
            };
            draw_text_bottom(&mut img, &text, 2, Rgba([255, 255, 255, 255]));
        }
    }

    let mut out = std::io::Cursor::new(Vec::new());
    if image::DynamicImage::ImageRgba8(img)
        .write_to(&mut out, image::ImageFormat::Png)
        .is_ok()
    {
        out.into_inner()
    } else {
        BASE_ICON_BYTES.to_vec()
    }
}

/// provider 內部 key → 顯示名稱
fn provider_label(provider: &str) -> &str {
    match provider {
        "claude" => "Claude",
        "copilot" => "Copilot",
        "opencode" => "OpenCode",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        other => other,
    }
}

/// 取單一 snapshot 中所有 window 的最高用量（0.0–1.0）；無 window 時回 None
fn snapshot_max_utilization(snap: &QuotaSnapshot) -> Option<f64> {
    let windows = snap.windows.as_ref()?;
    windows
        .iter()
        .map(|w| w.utilization)
        .fold(None, |acc, u| Some(acc.map_or(u, |a: f64| a.max(u))))
}

/// 依設定計算系統匣要反映的主要用量百分比（0.0–1.0）。
///
/// - 指定 `primary_provider` 時，取該 provider status=ok 的最高 window 用量
/// - 未指定時，掃描所有 status=ok 的 snapshot，取全域最高 window 用量
///
/// 找不到任何 ok 資料時回傳 0.0。
pub(crate) fn compute_primary_pct(
    snapshots: &[QuotaSnapshot],
    primary_provider: Option<&str>,
) -> f64 {
    let candidates = snapshots.iter().filter(|s| s.status == "ok");
    let max = match primary_provider {
        Some(target) => candidates
            .filter(|s| s.provider == target)
            .filter_map(snapshot_max_utilization)
            .fold(None, |acc, u| Some(acc.map_or(u, |a: f64| a.max(u)))),
        None => candidates
            .filter_map(snapshot_max_utilization)
            .fold(None, |acc, u| Some(acc.map_or(u, |a: f64| a.max(u)))),
    };
    max.unwrap_or(0.0).clamp(0.0, 1.0)
}

/// 依 snapshot 建構多行 tooltip 摘要（格式見 design.md §1.3）。
/// window 名稱一律取自 `QuotaWindow.label`；local_scan 期間取自 `LocalTokenUsage.period_label`。
pub(crate) fn build_tooltip(snapshots: &[QuotaSnapshot]) -> String {
    let mut lines = vec!["SessionHub".to_string()];
    for snap in snapshots {
        let name = provider_label(&snap.provider);
        match snap.status.as_str() {
            "ok" => {
                if let Some(local) = &snap.local_tokens {
                    let total = local.input_tokens + local.output_tokens;
                    lines.push(format!("{name}: {total} tok（{}）", local.period_label));
                } else if let Some(windows) = &snap.windows {
                    let parts: Vec<String> = windows
                        .iter()
                        .map(|w| {
                            format!("{}% ({})", (w.utilization * 100.0).round() as i64, w.label)
                        })
                        .collect();
                    if !parts.is_empty() {
                        lines.push(format!("{name}: {}", parts.join(" · ")));
                    }
                }
            }
            "no_auth" => lines.push(format!("{name}: 未登入")),
            "error" => lines.push(format!("{name}: 讀取失敗")),
            _ => {}
        }
    }
    lines.join("\n")
}

/// 依當前 QuotaCache 與設定重繪系統匣圖示與 tooltip。
/// `tray_quota_mode == Hidden` 或 quota monitoring 關閉時，還原原始底圖並清空 tooltip。
pub(crate) fn update_tray_from_cache(app: &AppHandle, settings: &AppSettings) {
    let Some(tray) = app.tray_by_id(TRAY_ICON_ID) else {
        return;
    };

    let snapshots: Vec<QuotaSnapshot> = {
        let cache = app.state::<QuotaCache>();
        let Ok(guard) = cache.snapshots.lock() else {
            return;
        };
        guard.values().cloned().collect()
    };

    let hidden = !settings.enable_quota_monitoring
        || matches!(settings.tray_quota_mode, TrayQuotaMode::Hidden);

    let pct = compute_primary_pct(&snapshots, settings.tray_quota_primary_provider.as_deref());

    let png = if hidden {
        BASE_ICON_BYTES.to_vec()
    } else {
        render_tray_icon_png(pct, settings.tray_quota_mode)
    };

    if let Ok(image) = tauri::image::Image::from_bytes(&png) {
        let _ = tray.set_icon(Some(image));
    }

    if hidden {
        let _ = tray.set_tooltip(Some("SessionHub"));
    } else {
        let _ = tray.set_tooltip(Some(&build_tooltip(&snapshots)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_snapshot(provider: &str, utils: &[f64]) -> QuotaSnapshot {
        QuotaSnapshot {
            provider: provider.to_string(),
            status: "ok".to_string(),
            source: "remote_api".to_string(),
            fetched_at: "2026-01-01T00:00:00Z".to_string(),
            error_message: None,
            windows: Some(
                utils
                    .iter()
                    .enumerate()
                    .map(|(i, u)| crate::types::QuotaWindow {
                        window_key: format!("w{i}"),
                        label: format!("window-{i}"),
                        utilization: *u,
                        resets_at: None,
                        group: None,
                    })
                    .collect(),
            ),
            local_tokens: None,
            extra_credits: None,
            reset_credits: None,
        }
    }

    #[test]
    fn compute_primary_pct_auto_picks_global_max() {
        let snaps = vec![
            ok_snapshot("claude", &[0.3, 0.72]),
            ok_snapshot("copilot", &[0.55]),
        ];
        assert!((compute_primary_pct(&snaps, None) - 0.72).abs() < 1e-9);
    }

    #[test]
    fn compute_primary_pct_respects_target_provider() {
        let snaps = vec![
            ok_snapshot("claude", &[0.9]),
            ok_snapshot("copilot", &[0.4]),
        ];
        assert!((compute_primary_pct(&snaps, Some("copilot")) - 0.4).abs() < 1e-9);
    }

    #[test]
    fn compute_primary_pct_ignores_non_ok_and_defaults_zero() {
        let mut snap = ok_snapshot("claude", &[0.8]);
        snap.status = "error".to_string();
        assert_eq!(compute_primary_pct(&[snap], None), 0.0);
    }

    #[test]
    fn build_tooltip_uses_window_labels() {
        let snaps = vec![ok_snapshot("claude", &[0.72])];
        let tip = build_tooltip(&snaps);
        assert!(tip.starts_with("SessionHub"));
        assert!(tip.contains("Claude"));
        assert!(tip.contains("window-0"));
        assert!(tip.contains("72%"));
    }

    #[test]
    fn quota_color_thresholds() {
        assert_eq!(quota_color(0.0), COLOR_OK);
        assert_eq!(quota_color(0.49), COLOR_OK);
        assert_eq!(quota_color(0.5), COLOR_WARN);
        assert_eq!(quota_color(0.79), COLOR_WARN);
        assert_eq!(quota_color(0.8), COLOR_DANGER);
        assert_eq!(quota_color(1.0), COLOR_DANGER);
    }

    #[test]
    fn render_produces_valid_png_for_all_modes() {
        for mode in [
            TrayQuotaMode::IconOnly,
            TrayQuotaMode::Percentage,
            TrayQuotaMode::Bar,
            TrayQuotaMode::Hidden,
        ] {
            let bytes = render_tray_icon_png(0.72, mode);
            assert!(!bytes.is_empty(), "png bytes should not be empty");
            // PNG magic number
            assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "should be PNG");
        }
    }

    #[test]
    fn render_clamps_out_of_range_pct() {
        let over = render_tray_icon_png(1.5, TrayQuotaMode::Bar);
        let under = render_tray_icon_png(-0.3, TrayQuotaMode::Bar);
        assert!(!over.is_empty());
        assert!(!under.is_empty());
    }
}
