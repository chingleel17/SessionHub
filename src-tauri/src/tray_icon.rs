//! 動態繪製系統匣圖示：依 quota 用量與顯示模式在底圖上疊加視覺指示。
//!
//! 為了可攜性與 KISS，Percentage 模式的數字採用內建 3x5 點陣手繪，不引入外部字型檔。

use image::{imageops::FilterType, load_from_memory, Rgba, RgbaImage};
use tauri::{AppHandle, Manager};

use crate::types::{AppSettings, QuotaCache, QuotaSnapshot, TrayQuotaMode};

/// 系統匣圖示的固定 id，供執行期以 `app.tray_by_id()` 取回更新
pub(crate) const TRAY_ICON_ID: &str = "main-tray";

/// 底圖：32x32 的 SessionHub icon（與 lib.rs 系統匣初始圖示同源）
const BASE_ICON_BYTES: &[u8] = include_bytes!("../icons/32x32.png");

/// macOS template 底圖來源：高解析度原圖，執行期縮小並轉成單色 alpha mask
const TEMPLATE_SOURCE_BYTES: &[u8] = include_bytes!("../icons/128x128.png");

/// template 畫布邊長（px）。繪製座標以 32px 為基準，乘上 `TEMPLATE_SIZE / 32`
const TEMPLATE_SIZE: u32 = 64;

/// macOS 以 template image 呈現選單列圖示，顏色交由系統依外觀與桌布亮度決定
pub(crate) const USE_TEMPLATE_ICON: bool = cfg!(target_os = "macos");

/// template 繪製用：不透明黑
const TEMPLATE_INK: Rgba<u8> = Rgba([0, 0, 0, 255]);
/// template 繪製用：全透明（挖空）
const TEMPLATE_CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

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

/// 用量三段等級：0 = <50%、1 = 50–80%、2 = >=80%（與 `quota_color` 門檻一致）
fn quota_level(pct: f64) -> u8 {
    if pct >= 0.8 {
        2
    } else if pct >= 0.5 {
        1
    } else {
        0
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

/// 直接覆寫像素（不做 alpha 混合），template 繪製只會用到 `TEMPLATE_INK` 與 `TEMPLATE_CLEAR`
fn set_rect(img: &mut RgbaImage, x0: u32, y0: u32, w: u32, h: u32, color: Rgba<u8>) {
    for y in y0..(y0 + h).min(img.height()) {
        for x in x0..(x0 + w).min(img.width()) {
            img.put_pixel(x, y, color);
        }
    }
}

/// 直接覆寫圓形區域；`lower_half_only` 為 true 時只覆寫圓心以下（含圓心列）
fn set_circle(
    img: &mut RgbaImage,
    cx: i32,
    cy: i32,
    radius: i32,
    color: Rgba<u8>,
    lower_half_only: bool,
) {
    let start = if lower_half_only { 0 } else { -radius };
    for dy in start..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            let (x, y) = (cx + dx, cy + dy);
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

/// 產生 macOS template 底圖：品牌圖示縮小後轉成純黑 + alpha 的單色 mask。
///
/// 原圖為深色底、白色字母；深色區域轉為不透明、白色字母挖空，
/// 使系統著色後字母以透明鏤空呈現。
fn template_base_icon() -> RgbaImage {
    let Ok(source) = load_from_memory(TEMPLATE_SOURCE_BYTES) else {
        return RgbaImage::from_pixel(TEMPLATE_SIZE, TEMPLATE_SIZE, TEMPLATE_CLEAR);
    };
    let mut img = image::imageops::resize(
        &source.to_rgba8(),
        TEMPLATE_SIZE,
        TEMPLATE_SIZE,
        FilterType::Lanczos3,
    );
    for pixel in img.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let luma = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0;
        // 亮度 <0.35 視為實心、>0.65 視為挖空，中間線性過渡以保留抗鋸齒邊緣
        let ink = ((0.65 - luma) / 0.30).clamp(0.0, 1.0);
        let alpha = (ink * a as f64).round() as u8;
        *pixel = Rgba([0, 0, 0, alpha]);
    }
    img
}

/// macOS template 繪製：輸出只含黑色與透明。
///
/// - `IconOnly`：右下角圓環，內部以空心／半實／實心表示三段用量
/// - `Bar`：品牌圖示下方一條單色進度條
/// - `Percentage`：僅繪製置中的數字（小尺寸下與圖示疊合難以辨識）
fn render_template_icon(pct: f64, mode: TrayQuotaMode) -> RgbaImage {
    let clamped = pct.clamp(0.0, 1.0);
    let scale = TEMPLATE_SIZE / 32;
    let s = scale as i32;
    let (w, h) = (TEMPLATE_SIZE, TEMPLATE_SIZE);

    match mode {
        TrayQuotaMode::Hidden => template_base_icon(),
        TrayQuotaMode::IconOnly => {
            let mut img = template_base_icon();
            let (cx, cy) = (w as i32 - 8 * s, h as i32 - 8 * s);
            set_circle(&mut img, cx, cy, 7 * s, TEMPLATE_CLEAR, false);
            set_circle(&mut img, cx, cy, 5 * s, TEMPLATE_INK, false);
            set_circle(&mut img, cx, cy, 3 * s, TEMPLATE_CLEAR, false);
            match quota_level(clamped) {
                0 => {}
                1 => set_circle(&mut img, cx, cy, 3 * s, TEMPLATE_INK, true),
                _ => set_circle(&mut img, cx, cy, 3 * s, TEMPLATE_INK, false),
            }
            img
        }
        TrayQuotaMode::Bar => {
            let mut img = template_base_icon();
            // 清出底部帶狀區域，避免進度條與實心底圖黏在一起
            set_rect(&mut img, 0, h - 9 * scale, w, 9 * scale, TEMPLATE_CLEAR);
            let (x0, y0) = (2 * scale, h - 7 * scale);
            let (outer_w, outer_h, border) = (w - 4 * scale, 6 * scale, scale);
            set_rect(&mut img, x0, y0, outer_w, outer_h, TEMPLATE_INK);
            let (inner_w, inner_h) = (outer_w - 2 * border, outer_h - 2 * border);
            set_rect(
                &mut img,
                x0 + border,
                y0 + border,
                inner_w,
                inner_h,
                TEMPLATE_CLEAR,
            );
            let fill_w = (inner_w as f64 * clamped).round() as u32;
            set_rect(
                &mut img,
                x0 + border,
                y0 + border,
                fill_w,
                inner_h,
                TEMPLATE_INK,
            );
            img
        }
        TrayQuotaMode::Percentage => {
            let mut img = RgbaImage::from_pixel(w, h, TEMPLATE_CLEAR);
            let pct_int = (clamped * 100.0).round() as u32;
            let text = if pct_int >= 100 {
                "99".to_string()
            } else {
                format!("{pct_int}")
            };
            let glyph_scale = 3 * scale;
            let count = text.chars().count() as u32;
            let total_w = count * 3 * glyph_scale + (count - 1) * glyph_scale;
            let x0 = w.saturating_sub(total_w) / 2;
            let y0 = h.saturating_sub(5 * glyph_scale) / 2;
            for (i, ch) in text.chars().enumerate() {
                let x = x0 + i as u32 * 4 * glyph_scale;
                draw_glyph(&mut img, ch, x, y0, glyph_scale, TEMPLATE_INK);
            }
            img
        }
    }
}

/// 將 RGBA buffer 編碼為 PNG
fn encode_png(img: RgbaImage) -> Option<Vec<u8>> {
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()?;
    Some(out.into_inner())
}

/// 目前平台的系統匣底圖 PNG（macOS 為單色 template、其他平台為彩色原圖）
pub(crate) fn tray_base_png() -> Vec<u8> {
    if USE_TEMPLATE_ICON {
        if let Some(bytes) = encode_png(template_base_icon()) {
            return bytes;
        }
    }
    BASE_ICON_BYTES.to_vec()
}

/// 依 quota 用量與模式繪製系統匣圖示，回傳 PNG 位元組。
///
/// - `pct`：主要 provider 的用量比例（0.0–1.0）
/// - `mode`：顯示模式
pub(crate) fn render_tray_icon_png(pct: f64, mode: TrayQuotaMode) -> Vec<u8> {
    if USE_TEMPLATE_ICON {
        return encode_png(render_template_icon(pct, mode)).unwrap_or_else(tray_base_png);
    }
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

    encode_png(img).unwrap_or_else(|| BASE_ICON_BYTES.to_vec())
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
/// window 名稱一律取自 `QuotaWindow.label`；無任何 window 的 provider 顯示無額度資料說明。
pub(crate) fn build_tooltip(snapshots: &[QuotaSnapshot]) -> String {
    let mut lines = vec!["SessionHub".to_string()];
    for snap in snapshots {
        let name = provider_label(&snap.provider);
        match snap.status.as_str() {
            "ok" => {
                let parts: Vec<String> = snap
                    .windows
                    .iter()
                    .flatten()
                    .map(|w| format!("{}% ({})", (w.utilization * 100.0).round() as i64, w.label))
                    .collect();
                if parts.is_empty() {
                    lines.push(format!("{name}: 無額度資料"));
                } else {
                    lines.push(format!("{name}: {}", parts.join(" · ")));
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
        tray_base_png()
    } else {
        render_tray_icon_png(pct, settings.tray_quota_mode)
    };

    if let Ok(image) = tauri::image::Image::from_bytes(&png) {
        // macOS 需與 icon 一併設定 template 狀態，分開呼叫會造成閃爍
        let _ = tray.set_icon_with_as_template(Some(image), USE_TEMPLATE_ICON);
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
            extra_credits: None,
            reset_credits: None,
            plan: None,
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

    fn assert_template_pixels(img: &RgbaImage) {
        for p in img.pixels() {
            assert_eq!(&p.0[0..3], &[0, 0, 0], "template 像素 RGB 必須為黑色");
        }
    }

    fn count_opaque(img: &RgbaImage) -> usize {
        img.pixels().filter(|p| p.0[3] > 0).count()
    }

    #[test]
    fn quota_level_thresholds() {
        assert_eq!(quota_level(0.0), 0);
        assert_eq!(quota_level(0.49), 0);
        assert_eq!(quota_level(0.5), 1);
        assert_eq!(quota_level(0.79), 1);
        assert_eq!(quota_level(0.8), 2);
        assert_eq!(quota_level(1.0), 2);
    }

    #[test]
    fn template_base_is_black_with_transparent_letter() {
        let img = template_base_icon();
        assert_eq!(img.dimensions(), (TEMPLATE_SIZE, TEMPLATE_SIZE));
        assert_template_pixels(&img);
        // 中心的字母挖空、字母外的圖示主體為實心
        let center = img.get_pixel(TEMPLATE_SIZE / 2, TEMPLATE_SIZE / 2).0[3];
        let body = img.get_pixel(TEMPLATE_SIZE / 2, 6).0[3];
        assert!(body > 200, "圖示主體應為實心，實際 alpha={body}");
        assert!(center < 255, "字母區域應被挖空");
    }

    #[test]
    fn template_icon_only_uses_three_distinct_fill_levels() {
        let low = render_template_icon(0.3, TrayQuotaMode::IconOnly);
        let mid = render_template_icon(0.65, TrayQuotaMode::IconOnly);
        let high = render_template_icon(0.9, TrayQuotaMode::IconOnly);
        for img in [&low, &mid, &high] {
            assert_template_pixels(img);
        }
        let (a, b, c) = (count_opaque(&low), count_opaque(&mid), count_opaque(&high));
        assert!(a < b && b < c, "填充量需隨等級遞增：{a} < {b} < {c}");
        // 門檻邊界：0.49 與 0.5、0.79 與 0.8 必須落在不同等級
        assert_eq!(render_template_icon(0.49, TrayQuotaMode::IconOnly), low);
        assert_ne!(
            render_template_icon(0.49, TrayQuotaMode::IconOnly),
            render_template_icon(0.5, TrayQuotaMode::IconOnly)
        );
        assert_ne!(
            render_template_icon(0.79, TrayQuotaMode::IconOnly),
            render_template_icon(0.8, TrayQuotaMode::IconOnly)
        );
    }

    #[test]
    fn template_bar_and_percentage_are_monochrome() {
        for pct in [0.0, 0.42, 1.0] {
            assert_template_pixels(&render_template_icon(pct, TrayQuotaMode::Bar));
            assert_template_pixels(&render_template_icon(pct, TrayQuotaMode::Percentage));
        }
        let empty = count_opaque(&render_template_icon(0.0, TrayQuotaMode::Bar));
        let full = count_opaque(&render_template_icon(1.0, TrayQuotaMode::Bar));
        assert!(full > empty, "進度條填滿時不透明像素應較多");
    }

    #[test]
    fn non_template_platforms_keep_colored_output() {
        if USE_TEMPLATE_ICON {
            return;
        }
        let png = render_tray_icon_png(0.9, TrayQuotaMode::IconOnly);
        let img = load_from_memory(&png).expect("valid png").to_rgba8();
        assert!(
            img.pixels().any(|p| p.0 == COLOR_DANGER.0),
            "非 macOS 平台仍須輸出紅色圓點"
        );
    }
}
