use super::bookmarks::BookmarkManager;
use super::navbar::NavBarHistory;

/// Generates title and HTML content for internal `kestrel://` browser scheme URLs.
pub fn render_internal_page(
    url: &str,
    bookmark_mgr: &BookmarkManager,
    history: &NavBarHistory,
) -> Option<(String, String)> {
    let normalized = url.trim().to_ascii_lowercase();

    if normalized == "kestrel://newtab"
        || normalized == "kestrel://"
        || normalized == "kestrel://home"
        || normalized == "about:blank"
    {
        return Some((
            "New Tab".to_string(),
            render_newtab_html(bookmark_mgr),
        ));
    }

    if normalized == "kestrel://about" {
        return Some((
            "About Kestrel".to_string(),
            render_about_html(),
        ));
    }

    if normalized == "kestrel://bookmarks" {
        return Some((
            "Bookmarks".to_string(),
            render_bookmarks_html(bookmark_mgr),
        ));
    }

    if normalized == "kestrel://history" {
        return Some((
            "Browsing History".to_string(),
            render_history_html(history),
        ));
    }

    if normalized == "kestrel://settings" {
        return Some((
            "Settings".to_string(),
            render_settings_html(),
        ));
    }

    None
}

fn render_newtab_html(_bookmark_mgr: &BookmarkManager) -> String {
    format!(
        r#"
        <div style="background-color: #080c14; min-height: 100vh; padding: 180px 24px 80px 24px; font-family: sans-serif; color: #f1f5f9; text-align: center;">
            <div style="max-width: 620px; margin: 0 auto; text-align: center;">
                
                <!-- Brand Title -->
                <div style="margin-bottom: 36px; text-align: center;">
                    <h1 style="color: #38bdf8; font-size: 56px; font-weight: 800; letter-spacing: 5px; margin: 0 0 10px 0; text-align: center;">
                        KESTREL
                    </h1>
                    <p style="color: #64748b; font-size: 14px; margin: 0; text-align: center;">
                        Memory-Safe Pure Rust Web Engine (Exploidus)
                    </p>
                </div>

                <!-- Floating Omnibox Search Box (Chrome Style) -->
                <div style="background-color: #0f1724; border: 1px solid #1e293b; border-radius: 28px; padding: 16px 28px; max-width: 580px; margin: 0 auto; text-align: left; box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45);">
                    <span style="color: #64748b; font-size: 15px;">Search the web or type a URL...</span>
                </div>

            </div>
        </div>
        "#
    )
}

fn render_about_html() -> String {
    format!(
        r#"
        <div style="background-color: #0b0f19; min-height: 100vh; padding: 40px 24px; font-family: sans-serif; color: #e2e8f0; text-align: center;">
            <div style="max-width: 800px; margin: 0 auto; text-align: left;">
                <h1 style="color: #38bdf8; font-size: 32px; font-weight: 800; margin-bottom: 8px;">
                    About Kestrel Browser
                </h1>
                <p style="color: #94a3b8; font-size: 15px; margin-bottom: 24px;">
                    Version 0.4.0 (Milestone #3) &bull; Built from scratch in pure Rust by Rahad Bhuiya
                </p>

                <div style="background-color: #111827; border-radius: 12px; padding: 24px; border: 1px solid #1f2937; margin-bottom: 20px;">
                    <h3 style="color: #60a5fa; margin-top: 0;">Philosophy &amp; Mission</h3>
                    <p style="color: #cbd5e1; font-size: 14px; line-height: 1.6;">
                        Modern web browsers have become multi-gigabyte monolithic operating systems dominated by single-vendor monopolies.
                        <strong>Kestrel</strong> is an exploration into first-principles systems engineering: proving that a fast, secure,
                        and memory-safe Zero-Chromium web browser engine can be built without a single line of Chromium or WebKit C++ code.
                    </p>
                </div>

                <div style="background-color: #111827; border-radius: 12px; padding: 24px; border: 1px solid #1f2937; margin-bottom: 20px;">
                    <h3 style="color: #60a5fa; margin-top: 0;">Milestone Roadmap</h3>
                    <ul style="color: #cbd5e1; font-size: 14px; line-height: 1.8;">
                        <li><strong>v0.1.0:</strong> HTML5 Tokenizer, Tree Builder &amp; Arena DOM</li>
                        <li><strong>v0.2.0:</strong> CSSOM Parser, Flexbox Layout &amp; GPU Paint Display List</li>
                        <li><strong>v0.3.0:</strong> TLS 1.3 Networking, Bytecode JS VM &amp; Multi-Process Sandbox</li>
                        <li><strong>v0.4.0:</strong> W3C DOM Event Bubbling, Interactive Forms &amp; Audio/Video Streaming</li>
                    </ul>
                </div>

                <p style="text-align: center; margin-top: 30px;">
                    <a href="kestrel://newtab" style="color: #38bdf8; text-decoration: none; font-weight: bold;">&larr; Back to New Tab</a>
                </p>
            </div>
        </div>
        "#
    )
}

fn render_bookmarks_html(bookmark_mgr: &BookmarkManager) -> String {
    let mut items_html = String::new();
    if bookmark_mgr.items.is_empty() {
        items_html.push_str(r#"<p style="color: #64748b;">No bookmarks saved yet. Press Ctrl+D on any page to bookmark it!</p>"#);
    } else {
        for bm in &bookmark_mgr.items {
            items_html.push_str(&format!(
                r#"
                <div style="background-color: #1e293b; padding: 12px 16px; border-radius: 8px; margin-bottom: 10px; display: flex; justify-content: space-between;">
                    <div>
                        <a href="{}" style="color: #38bdf8; font-weight: bold; text-decoration: none; font-size: 15px;">{}</a>
                        <p style="color: #94a3b8; font-size: 12px; margin: 4px 0 0 0;">{}</p>
                    </div>
                </div>
                "#,
                bm.url, bm.title, bm.url
            ));
        }
    }

    format!(
        r#"
        <div style="background-color: #0b0f19; min-height: 100vh; padding: 40px 24px; font-family: sans-serif; color: #e2e8f0;">
            <div style="max-width: 800px; margin: 0 auto;">
                <h1 style="color: #38bdf8; font-size: 28px; font-weight: 800; margin-bottom: 8px;">Saved Bookmarks</h1>
                <p style="color: #94a3b8; font-size: 14px; margin-bottom: 24px;">Manage your saved web destinations.</p>
                <div style="background-color: #111827; border-radius: 12px; padding: 24px; border: 1px solid #1f2937;">
                    {}
                </div>
                <p style="margin-top: 24px;">
                    <a href="kestrel://newtab" style="color: #38bdf8; text-decoration: none; font-weight: bold;">&larr; Back to New Tab</a>
                </p>
            </div>
        </div>
        "#,
        items_html
    )
}

fn render_history_html(history: &NavBarHistory) -> String {
    let mut items_html = String::new();
    let mut all_history: Vec<&String> = history.back_stack.iter().collect();
    if let Some(ref curr) = history.current_url {
        all_history.push(curr);
    }

    if all_history.is_empty() {
        items_html.push_str(r#"<p style="color: #64748b;">No browsing history recorded yet.</p>"#);
    } else {
        for url in all_history.iter().rev() {
            items_html.push_str(&format!(
                r#"
                <div style="padding: 10px 14px; border-bottom: 1px solid #1f2937;">
                    <a href="{}" style="color: #60a5fa; text-decoration: none; font-size: 14px;">{}</a>
                </div>
                "#,
                url, url
            ));
        }
    }

    format!(
        r#"
        <div style="background-color: #0b0f19; min-height: 100vh; padding: 40px 24px; font-family: sans-serif; color: #e2e8f0;">
            <div style="max-width: 800px; margin: 0 auto;">
                <h1 style="color: #38bdf8; font-size: 28px; font-weight: 800; margin-bottom: 8px;">Browsing History</h1>
                <p style="color: #94a3b8; font-size: 14px; margin-bottom: 24px;">Previously visited web pages.</p>
                <div style="background-color: #111827; border-radius: 12px; padding: 20px; border: 1px solid #1f2937;">
                    {}
                </div>
                <p style="margin-top: 24px;">
                    <a href="kestrel://newtab" style="color: #38bdf8; text-decoration: none; font-weight: bold;">&larr; Back to New Tab</a>
                </p>
            </div>
        </div>
        "#,
        items_html
    )
}

fn render_settings_html() -> String {
    format!(
        r#"
        <div style="background-color: #0b0f19; min-height: 100vh; padding: 40px 24px; font-family: sans-serif; color: #e2e8f0;">
            <div style="max-width: 800px; margin: 0 auto;">
                <h1 style="color: #38bdf8; font-size: 28px; font-weight: 800; margin-bottom: 8px;">Kestrel Settings</h1>
                <p style="color: #94a3b8; font-size: 14px; margin-bottom: 24px;">Engine and privacy configurations.</p>
                
                <div style="background-color: #111827; border-radius: 12px; padding: 24px; border: 1px solid #1f2937; margin-bottom: 20px;">
                    <h3 style="color: #60a5fa; margin-top: 0;">Privacy &amp; Security</h3>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>Tracker Protection:</strong> Enabled (Strict)</p>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>Content Security Policy:</strong> Enforced</p>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>TLS Protocol:</strong> TLS 1.3 / Ring Cryptography</p>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>Cookie Isolation:</strong> First-Party Domain Isolated</p>
                </div>

                <div style="background-color: #111827; border-radius: 12px; padding: 24px; border: 1px solid #1f2937;">
                    <h3 style="color: #60a5fa; margin-top: 0;">Graphics &amp; Compositor</h3>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>Backend:</strong> wgpu (Vulkan / DX12)</p>
                    <p style="color: #cbd5e1; font-size: 14px;"><strong>Glyph Rasterizer:</strong> wgpu_glyph GPU Atlas Cache</p>
                </div>

                <p style="margin-top: 24px;">
                    <a href="kestrel://newtab" style="color: #38bdf8; text-decoration: none; font-weight: bold;">&larr; Back to New Tab</a>
                </p>
            </div>
        </div>
        "#
    )
}
