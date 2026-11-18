use crate::layout::box_model::Rect;
use crate::paint::{Color, PaintCommand};
use super::bookmarks::BookmarkManager;
use super::tab::TabManager;

pub const TAB_BAR_HEIGHT: f32 = 34.0;
pub const TOOLBAR_HEIGHT: f32 = 40.0;
pub const BOOKMARKS_BAR_HEIGHT: f32 = 26.0;
pub const TOTAL_CHROME_HEIGHT: f32 = TAB_BAR_HEIGHT + TOOLBAR_HEIGHT + BOOKMARKS_BAR_HEIGHT; // 100.0 px

#[derive(Debug, Clone, PartialEq)]
pub enum ChromeClick {
    Tab(usize),
    CloseTab(usize),
    NewTab,
    Back,
    Forward,
    Reload,
    Omnibox,
    BookmarkToggle,
    ShieldToggle,
    DevToolsToggle,
    BookmarkUrl(String),
    None,
}

/// Generates fixed GPU paint commands to render the modern Chrome/Edge-style browser chrome.
pub fn build_chrome_display_list(
    viewport_width: f32,
    tab_manager: &TabManager,
    address_text: &str,
    is_bookmarked: bool,
    blocked_trackers: usize,
    bookmark_mgr: &BookmarkManager,
) -> Vec<PaintCommand> {
    let mut cmds = Vec::new();
    let width = viewport_width.max(600.0);

    // ==========================================
    // 1. TOP TAB STRIP (y: 0.0 .. 34.0)
    // ==========================================
    // Tab strip background
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 0.0, y: 0.0, width, height: TAB_BAR_HEIGHT },
        color: Color::new(0.047, 0.063, 0.094, 1.0), // #0c1018
        border_radius: 0.0,
        is_fixed: true,
    });

    let tab_stride = 170.0f32;
    let tab_w = 164.0f32;
    let tab_h = 29.0f32;
    let tab_y = 5.0f32;

    for (i, tab) in tab_manager.tabs.iter().enumerate() {
        let tab_x = 8.0 + i as f32 * tab_stride;
        let is_active = i == tab_manager.active_index;

        if is_active {
            // Active tab body (seamless with toolbar below)
            cmds.push(PaintCommand::DrawRect {
                rect: Rect { x: tab_x, y: tab_y, width: tab_w, height: tab_h },
                color: Color::new(0.098, 0.129, 0.188, 1.0), // #192130
                border_radius: 6.0,
                is_fixed: true,
            });
            // Top accent line
            cmds.push(PaintCommand::DrawRect {
                rect: Rect { x: tab_x + 2.0, y: tab_y, width: tab_w - 4.0, height: 2.0 },
                color: Color::new(0.22, 0.74, 0.97, 1.0), // #38bdf8
                border_radius: 1.0,
                is_fixed: true,
            });
            // Tab Title
            let title = if tab.title.len() > 17 { format!("{}...", &tab.title[..14]) } else { tab.title.clone() };
            cmds.push(PaintCommand::DrawText {
                text: title,
                rect: Rect { x: tab_x + 12.0, y: tab_y + 8.0, width: tab_w - 36.0, height: 18.0 },
                color: Color::new(0.95, 0.98, 1.0, 1.0),
                font_size: 13.0,
                is_fixed: true,
            });
            // Close Button
            cmds.push(PaintCommand::DrawText {
                text: "x".to_string(),
                rect: Rect { x: tab_x + tab_w - 20.0, y: tab_y + 7.0, width: 14.0, height: 14.0 },
                color: Color::new(0.60, 0.65, 0.75, 1.0),
                font_size: 12.0,
                is_fixed: true,
            });
        } else {
            // Inactive tab body
            cmds.push(PaintCommand::DrawRect {
                rect: Rect { x: tab_x, y: tab_y + 2.0, width: tab_w, height: tab_h - 2.0 },
                color: Color::new(0.065, 0.086, 0.125, 1.0), // #101620
                border_radius: 5.0,
                is_fixed: true,
            });
            let title = if tab.title.len() > 17 { format!("{}...", &tab.title[..14]) } else { tab.title.clone() };
            cmds.push(PaintCommand::DrawText {
                text: title,
                rect: Rect { x: tab_x + 12.0, y: tab_y + 9.0, width: tab_w - 36.0, height: 18.0 },
                color: Color::new(0.55, 0.60, 0.70, 1.0),
                font_size: 12.0,
                is_fixed: true,
            });
            // Close Button
            cmds.push(PaintCommand::DrawText {
                text: "x".to_string(),
                rect: Rect { x: tab_x + tab_w - 20.0, y: tab_y + 8.0, width: 14.0, height: 14.0 },
                color: Color::new(0.40, 0.45, 0.55, 1.0),
                font_size: 12.0,
                is_fixed: true,
            });
        }
    }

    // New Tab (+) Button
    let new_tab_x = 8.0 + tab_manager.tabs.len() as f32 * tab_stride;
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: new_tab_x, y: 7.0, width: 26.0, height: 24.0 },
        color: Color::new(0.098, 0.129, 0.188, 1.0),
        border_radius: 5.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "+".to_string(),
        rect: Rect { x: new_tab_x + 8.0, y: 10.0, width: 14.0, height: 14.0 },
        color: Color::new(0.70, 0.75, 0.85, 1.0),
        font_size: 14.0,
        is_fixed: true,
    });

    // ==========================================
    // 2. NAVIGATION & OMNIBOX TOOLBAR (y: 34.0 .. 74.0)
    // ==========================================
    // Toolbar background
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 0.0, y: TAB_BAR_HEIGHT, width, height: TOOLBAR_HEIGHT },
        color: Color::new(0.098, 0.129, 0.188, 1.0), // #192130 (same as active tab)
        border_radius: 0.0,
        is_fixed: true,
    });

    let btn_y = 39.0f32;
    let btn_w = 32.0f32;
    let btn_h = 30.0f32;

    // Back Button (<-)
    let can_back = tab_manager.active_tab().history.can_go_back();
    let back_bg = if can_back { Color::new(0.14, 0.18, 0.26, 1.0) } else { Color::new(0.08, 0.10, 0.15, 1.0) };
    let back_fg = if can_back { Color::new(0.9, 0.95, 1.0, 1.0) } else { Color::new(0.35, 0.40, 0.50, 1.0) };
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 10.0, y: btn_y, width: btn_w, height: btn_h },
        color: back_bg,
        border_radius: 6.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "<".to_string(),
        rect: Rect { x: 21.0, y: btn_y + 6.0, width: 14.0, height: 16.0 },
        color: back_fg,
        font_size: 14.0,
        is_fixed: true,
    });

    // Forward Button (->)
    let can_fwd = tab_manager.active_tab().history.can_go_forward();
    let fwd_bg = if can_fwd { Color::new(0.14, 0.18, 0.26, 1.0) } else { Color::new(0.08, 0.10, 0.15, 1.0) };
    let fwd_fg = if can_fwd { Color::new(0.9, 0.95, 1.0, 1.0) } else { Color::new(0.35, 0.40, 0.50, 1.0) };
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 46.0, y: btn_y, width: btn_w, height: btn_h },
        color: fwd_bg,
        border_radius: 6.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: ">".to_string(),
        rect: Rect { x: 57.0, y: btn_y + 6.0, width: 14.0, height: 16.0 },
        color: fwd_fg,
        font_size: 14.0,
        is_fixed: true,
    });

    // Reload Button (R)
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 82.0, y: btn_y, width: btn_w, height: btn_h },
        color: Color::new(0.14, 0.18, 0.26, 1.0),
        border_radius: 6.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "R".to_string(),
        rect: Rect { x: 92.0, y: btn_y + 6.0, width: 14.0, height: 16.0 },
        color: Color::new(0.22, 0.74, 0.97, 1.0),
        font_size: 13.0,
        is_fixed: true,
    });

    // Omnibox Capsule (Pill Shape)
    let omni_x = 124.0f32;
    let devtools_w = 76.0f32;
    let omni_w = (width - omni_x - devtools_w - 20.0).max(200.0);

    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: omni_x, y: btn_y, width: omni_w, height: btn_h },
        color: Color::new(0.05, 0.07, 0.10, 1.0), // #0d121a
        border_radius: 15.0, // Sleek capsule shape
        is_fixed: true,
    });

    // Security indicator inside Omnibox
    let is_internal = address_text.starts_with("kestrel://");
    let is_https = address_text.starts_with("https://");
    let lock_text = if is_internal { "kestrel" } else if is_https { "secure" } else { "web" };
    let lock_color = if is_internal {
        Color::new(0.22, 0.74, 0.97, 1.0) // Cyan
    } else if is_https {
        Color::new(0.10, 0.73, 0.50, 1.0) // Emerald
    } else {
        Color::new(0.60, 0.65, 0.75, 1.0)
    };
    cmds.push(PaintCommand::DrawText {
        text: lock_text.to_string(),
        rect: Rect { x: omni_x + 14.0, y: btn_y + 7.0, width: 50.0, height: 16.0 },
        color: lock_color,
        font_size: 11.0,
        is_fixed: true,
    });

    // Address / Search Query Text
    let max_text_len = ((omni_w - 140.0) / 7.5).max(10.0) as usize;
    let display_addr = if address_text.len() > max_text_len {
        format!("{}...", &address_text[..max_text_len - 3])
    } else {
        address_text.to_string()
    };
    cmds.push(PaintCommand::DrawText {
        text: format!("{} |", display_addr),
        rect: Rect { x: omni_x + 68.0, y: btn_y + 7.0, width: omni_w - 140.0, height: 16.0 },
        color: Color::new(0.90, 0.94, 1.0, 1.0),
        font_size: 13.0,
        is_fixed: true,
    });

    // Bookmark Star Action
    let star_x = omni_x + omni_w - 55.0;
    let star_fg = if is_bookmarked { Color::new(0.98, 0.80, 0.08, 1.0) } else { Color::new(0.40, 0.45, 0.55, 1.0) };
    cmds.push(PaintCommand::DrawText {
        text: if is_bookmarked { "★".to_string() } else { "☆".to_string() },
        rect: Rect { x: star_x, y: btn_y + 6.0, width: 18.0, height: 18.0 },
        color: star_fg,
        font_size: 14.0,
        is_fixed: true,
    });

    // Shield Badge Action
    let shield_x = omni_x + omni_w - 28.0;
    let shield_fg = if blocked_trackers > 0 { Color::new(0.95, 0.25, 0.35, 1.0) } else { Color::new(0.10, 0.73, 0.50, 1.0) };
    cmds.push(PaintCommand::DrawText {
        text: "S".to_string(),
        rect: Rect { x: shield_x, y: btn_y + 6.0, width: 18.0, height: 18.0 },
        color: shield_fg,
        font_size: 13.0,
        is_fixed: true,
    });

    // DevTools Button (Right of Omnibox)
    let devtools_x = width - devtools_w - 10.0;
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: devtools_x, y: btn_y, width: devtools_w, height: btn_h },
        color: Color::new(0.14, 0.18, 0.26, 1.0),
        border_radius: 6.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "DevTools".to_string(),
        rect: Rect { x: devtools_x + 10.0, y: btn_y + 8.0, width: 56.0, height: 16.0 },
        color: Color::new(0.22, 0.74, 0.97, 1.0),
        font_size: 11.0,
        is_fixed: true,
    });

    // ==========================================
    // 3. BOOKMARKS STRIP (y: 74.0 .. 100.0)
    // ==========================================
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 0.0, y: 74.0, width, height: BOOKMARKS_BAR_HEIGHT },
        color: Color::new(0.065, 0.086, 0.125, 1.0), // #101620
        border_radius: 0.0,
        is_fixed: true,
    });
    // Border line under chrome
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: 0.0, y: 99.0, width, height: 1.0 },
        color: Color::new(0.14, 0.18, 0.26, 1.0),
        border_radius: 0.0,
        is_fixed: true,
    });

    let mut bm_x = 12.0f32;
    let right_limit = width - 190.0;
    for bm in &bookmark_mgr.items {
        let chip_w = ((bm.title.len() * 7 + 16) as f32).max(50.0);
        cmds.push(PaintCommand::DrawRect {
            rect: Rect { x: bm_x, y: 77.0, width: chip_w, height: 20.0 },
            color: Color::new(0.12, 0.15, 0.22, 1.0),
            border_radius: 4.0,
            is_fixed: true,
        });
        cmds.push(PaintCommand::DrawText {
            text: bm.title.clone(),
            rect: Rect { x: bm_x + 8.0, y: 81.0, width: chip_w - 16.0, height: 14.0 },
            color: Color::new(0.38, 0.74, 0.98, 1.0),
            font_size: 11.0,
            is_fixed: true,
        });
        bm_x += chip_w + 8.0;
        if bm_x > right_limit {
            break;
        }
    }

    // Right-aligned Internal Browser Controls in Bookmarks Strip
    let about_w = 48.0f32;
    let settings_w = 58.0f32;
    let history_w = 52.0f32;

    let about_x = width - about_w - 12.0;
    let settings_x = about_x - settings_w - 8.0;
    let history_x = settings_x - history_w - 8.0;

    // History button
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: history_x, y: 77.0, width: history_w, height: 20.0 },
        color: Color::new(0.10, 0.13, 0.18, 1.0),
        border_radius: 4.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "History".to_string(),
        rect: Rect { x: history_x + 6.0, y: 81.0, width: history_w - 12.0, height: 14.0 },
        color: Color::new(0.60, 0.65, 0.75, 1.0),
        font_size: 11.0,
        is_fixed: true,
    });

    // Settings button
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: settings_x, y: 77.0, width: settings_w, height: 20.0 },
        color: Color::new(0.10, 0.13, 0.18, 1.0),
        border_radius: 4.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "Settings".to_string(),
        rect: Rect { x: settings_x + 6.0, y: 81.0, width: settings_w - 12.0, height: 14.0 },
        color: Color::new(0.60, 0.65, 0.75, 1.0),
        font_size: 11.0,
        is_fixed: true,
    });

    // About button
    cmds.push(PaintCommand::DrawRect {
        rect: Rect { x: about_x, y: 77.0, width: about_w, height: 20.0 },
        color: Color::new(0.10, 0.13, 0.18, 1.0),
        border_radius: 4.0,
        is_fixed: true,
    });
    cmds.push(PaintCommand::DrawText {
        text: "About".to_string(),
        rect: Rect { x: about_x + 6.0, y: 81.0, width: about_w - 12.0, height: 14.0 },
        color: Color::new(0.60, 0.65, 0.75, 1.0),
        font_size: 11.0,
        is_fixed: true,
    });

    cmds
}

/// Hit-tests a click event against the browser chrome components.
pub fn hit_test_chrome(
    x: f32,
    y: f32,
    viewport_width: f32,
    tab_manager: &TabManager,
    bookmark_mgr: &BookmarkManager,
) -> ChromeClick {
    // 1. Tab Bar (y: 0.0 .. 34.0)
    if y <= TAB_BAR_HEIGHT {
        let tab_stride = 170.0f32;
        let tab_w = 164.0f32;
        for i in 0..tab_manager.tabs.len() {
            let tab_x = 8.0 + i as f32 * tab_stride;
            if x >= tab_x && x <= tab_x + tab_w {
                // Check close button click (right 24px of tab)
                if x >= tab_x + tab_w - 24.0 {
                    return ChromeClick::CloseTab(i);
                }
                return ChromeClick::Tab(i);
            }
        }
        // Check new tab (+) button
        let new_tab_x = 8.0 + tab_manager.tabs.len() as f32 * tab_stride;
        if x >= new_tab_x && x <= new_tab_x + 30.0 {
            return ChromeClick::NewTab;
        }
        return ChromeClick::None;
    }

    // 2. Toolbar (y: 34.0 .. 74.0)
    if y > TAB_BAR_HEIGHT && y <= 74.0 {
        // Back
        if x >= 10.0 && x <= 42.0 {
            return ChromeClick::Back;
        }
        // Forward
        if x >= 46.0 && x <= 78.0 {
            return ChromeClick::Forward;
        }
        // Reload
        if x >= 82.0 && x <= 114.0 {
            return ChromeClick::Reload;
        }
        // DevTools
        let devtools_w = 76.0f32;
        let devtools_x = viewport_width - devtools_w - 10.0;
        if x >= devtools_x && x <= devtools_x + devtools_w {
            return ChromeClick::DevToolsToggle;
        }

        // Omnibox
        let omni_x = 124.0f32;
        let omni_w = (viewport_width - omni_x - devtools_w - 20.0).max(200.0);
        if x >= omni_x && x <= omni_x + omni_w {
            // Check Star button (right 55px .. 30px)
            let star_x = omni_x + omni_w - 55.0;
            if x >= star_x && x <= star_x + 24.0 {
                return ChromeClick::BookmarkToggle;
            }
            // Check Shield button (right 28px .. 6px)
            let shield_x = omni_x + omni_w - 28.0;
            if x >= shield_x {
                return ChromeClick::ShieldToggle;
            }
            return ChromeClick::Omnibox;
        }
        return ChromeClick::None;
    }

    // 3. Bookmarks Strip (y: 74.0 .. 100.0)
    if y > 74.0 && y <= TOTAL_CHROME_HEIGHT {
        let about_w = 48.0f32;
        let settings_w = 58.0f32;
        let history_w = 52.0f32;

        let about_x = viewport_width - about_w - 12.0;
        let settings_x = about_x - settings_w - 8.0;
        let history_x = settings_x - history_w - 8.0;

        if x >= about_x && x <= about_x + about_w {
            return ChromeClick::BookmarkUrl("kestrel://about".to_string());
        }
        if x >= settings_x && x <= settings_x + settings_w {
            return ChromeClick::BookmarkUrl("kestrel://settings".to_string());
        }
        if x >= history_x && x <= history_x + history_w {
            return ChromeClick::BookmarkUrl("kestrel://history".to_string());
        }

        let mut bm_x = 12.0f32;
        for bm in &bookmark_mgr.items {
            let chip_w = ((bm.title.len() * 7 + 16) as f32).max(50.0);
            if x >= bm_x && x <= bm_x + chip_w {
                return ChromeClick::BookmarkUrl(bm.url.clone());
            }
            bm_x += chip_w + 8.0;
            if bm_x > history_x - 10.0 {
                break;
            }
        }
        return ChromeClick::None;
    }

    ChromeClick::None
}
