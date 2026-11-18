//! Built-in Zero-Telemetry Privacy Shield & Tracker Blocker.
//!
//! Blocks tracking scripts, advertising beacons, and telemetry probes
//! before network socket dispatch, and provides an in-memory privacy dashboard.

/// Category of blocked tracker or ad request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerCategory {
    Analytics,
    Advertising,
    Fingerprinting,
    Telemetry,
}

impl TrackerCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrackerCategory::Analytics => "Analytics",
            TrackerCategory::Advertising => "Advertising",
            TrackerCategory::Fingerprinting => "Fingerprinting",
            TrackerCategory::Telemetry => "Telemetry",
        }
    }
}

/// Record of an intercepted tracking request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedEvent {
    pub url: String,
    pub domain: String,
    pub category: TrackerCategory,
    pub rule_matched: String,
}

/// Native zero-telemetry Tracker Blocker engine.
#[derive(Debug, Clone)]
pub struct TrackerBlocker {
    pub enabled: bool,
    pub blocked_events: Vec<BlockedEvent>,
    blocklist: Vec<(&'static str, TrackerCategory)>,
}

impl TrackerBlocker {
    pub fn new() -> Self {
        Self {
            enabled: true,
            blocked_events: Vec::new(),
            blocklist: vec![
                ("google-analytics.com", TrackerCategory::Analytics),
                ("googletagmanager.com", TrackerCategory::Analytics),
                ("analytics.twitter.com", TrackerCategory::Analytics),
                ("segment.io", TrackerCategory::Analytics),
                ("mixpanel.com", TrackerCategory::Analytics),
                ("amplitude.com", TrackerCategory::Analytics),
                ("hotjar.com", TrackerCategory::Analytics),
                ("scorecardresearch.com", TrackerCategory::Analytics),
                ("stats.wp.com", TrackerCategory::Analytics),
                ("pixel.wp.com", TrackerCategory::Analytics),
                ("doubleclick.net", TrackerCategory::Advertising),
                ("adservice.google.com", TrackerCategory::Advertising),
                ("pagead2.googlesyndication.com", TrackerCategory::Advertising),
                ("adnxs.com", TrackerCategory::Advertising),
                ("criteo.com", TrackerCategory::Advertising),
                ("facebook.com/tr", TrackerCategory::Fingerprinting),
                ("connect.facebook.net", TrackerCategory::Fingerprinting),
                ("pixel.facebook.com", TrackerCategory::Fingerprinting),
                ("fingerprintjs", TrackerCategory::Fingerprinting),
                ("telemetry.mozilla.org", TrackerCategory::Telemetry),
                ("vortex.data.microsoft.com", TrackerCategory::Telemetry),
                ("sentry.io/api", TrackerCategory::Telemetry),
            ],
        }
    }

    /// Evaluates whether an outgoing URL matches any tracker / telemetry rule.
    /// Returns `Some((category, rule))` if blocked, `None` if allowed.
    pub fn check_url(&self, url: &str) -> Option<(TrackerCategory, &'static str)> {
        if !self.enabled {
            return None;
        }
        let lower = url.to_lowercase();
        for &(pattern, category) in &self.blocklist {
            if lower.contains(pattern) {
                return Some((category, pattern));
            }
        }
        None
    }

    /// Intercepts and records a blocked URL if it matches tracker rules.
    /// Returns `true` if blocked, `false` if allowed.
    pub fn intercept(&mut self, url: &str) -> bool {
        if let Some((cat, rule)) = self.check_url(url) {
            let domain = extract_host_from_url(url);
            self.blocked_events.push(BlockedEvent {
                url: url.to_string(),
                domain,
                category: cat,
                rule_matched: rule.to_string(),
            });
            true
        } else {
            false
        }
    }

    pub fn total_blocked(&self) -> usize {
        self.blocked_events.len()
    }

    pub fn blocked_by_category(&self, cat: TrackerCategory) -> usize {
        self.blocked_events
            .iter()
            .filter(|e| e.category == cat)
            .count()
    }

    pub fn clear(&mut self) {
        self.blocked_events.clear();
    }
}

/// Helper to extract hostname from URL string without external crates.
fn extract_host_from_url(url: &str) -> String {
    let without_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };
    let host_end = without_scheme
        .find(|c| c == '/' || c == ':' || c == '?')
        .unwrap_or(without_scheme.len());
    without_scheme[..host_end].to_lowercase()
}

/// Privacy Inspection Dashboard modal / panel state.
#[derive(Debug, Clone)]
pub struct PrivacyDashboard {
    pub is_open: bool,
}

impl PrivacyDashboard {
    pub fn new() -> Self {
        Self { is_open: false }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn render_html(&self, blocker: &TrackerBlocker, active_page_url: &str) -> String {
        if !self.is_open {
            return String::new();
        }

        let total = blocker.total_blocked();
        let ads = blocker.blocked_by_category(TrackerCategory::Advertising);
        let analytics = blocker.blocked_by_category(TrackerCategory::Analytics);
        let fingerprinting = blocker.blocked_by_category(TrackerCategory::Fingerprinting);
        let telemetry = blocker.blocked_by_category(TrackerCategory::Telemetry);

        let mut events_html = String::new();
        if blocker.blocked_events.is_empty() {
            events_html.push_str(r#"<p style="color: #4ade80; font-size: 12px; margin: 4px 0;">Clean page! Zero trackers or advertising beacons detected.</p>"#);
        } else {
            for ev in blocker.blocked_events.iter().rev().take(8) {
                events_html.push_str(&format!(
                    r#"<div style="background-color: #0f172a; padding: 4px 8px; margin: 3px 0; border-left: 3px solid #f43f5e; font-size: 11px;">
                        <span style="color: #f43f5e; font-weight: bold;">[BLOCKED]</span>
                        <span style="color: #38bdf8; font-weight: bold; margin: 0 4px;">{}</span>
                        <span style="color: #94a3b8;">({})</span>
                    </div>"#,
                    ev.domain,
                    ev.category.as_str()
                ));
            }
        }

        format!(
            r#"
            <div class="privacy-dashboard" style="background-color: #020617; border: 2px solid #10b981; border-radius: 8px; padding: 12px; margin-top: 10px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <span style="color: #10b981; font-weight: bold; font-size: 14px;">PRIVACY SHIELD &amp; TRACKER INSPECTION DASHBOARD</span>
                    <span style="background-color: #064e3b; color: #34d399; font-size: 11px; padding: 2px 8px; border-radius: 4px; font-weight: bold;">ZERO-TELEMETRY MODE</span>
                </div>
                <p style="color: #cbd5e1; font-size: 12px; margin: 2px 0;">Active Page: <span style="color: #38bdf8;">{}</span></p>
                <div style="display: flex; gap: 12px; margin: 8px 0; padding: 8px; background-color: #0f172a; border-radius: 6px;">
                    <div style="color: #f43f5e; font-weight: bold; font-size: 12px;">Total Blocked: {}</div>
                    <div style="color: #fb923c; font-size: 12px;">Ads: {}</div>
                    <div style="color: #60a5fa; font-size: 12px;">Analytics: {}</div>
                    <div style="color: #c084fc; font-size: 12px;">Fingerprinting: {}</div>
                    <div style="color: #a3e635; font-size: 12px;">Telemetry: {}</div>
                </div>
                <div style="margin-top: 6px;">
                    <p style="color: #94a3b8; font-weight: bold; font-size: 12px; margin-bottom: 4px;">Recent Intercepted Trackers:</p>
                    {}
                </div>
                <p style="color: #64748b; font-size: 10px; margin-top: 6px;">All tracking queries are severed at the client layer before socket dispatch. No data is stored or uploaded externally.</p>
            </div>
            "#,
            active_page_url, total, ads, analytics, fingerprinting, telemetry, events_html
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_blocking_rules() {
        let mut blocker = TrackerBlocker::new();

        assert!(blocker.intercept("https://www.google-analytics.com/analytics.js"));
        assert!(blocker.intercept("https://adservice.google.com/adsid/google/ui"));
        assert!(blocker.intercept("https://connect.facebook.net/en_US/fbevents.js"));
        assert!(blocker.intercept("https://telemetry.mozilla.org/submit/telemetry"));

        // Normal site should not be blocked
        assert!(!blocker.intercept("https://example.com/index.html"));
        assert!(!blocker.intercept("https://github.com/rahadbhuiya/browser_engine"));

        assert_eq!(blocker.total_blocked(), 4);
        assert_eq!(
            blocker.blocked_by_category(TrackerCategory::Analytics),
            1
        );
        assert_eq!(
            blocker.blocked_by_category(TrackerCategory::Advertising),
            1
        );
        assert_eq!(
            blocker.blocked_by_category(TrackerCategory::Fingerprinting),
            1
        );
        assert_eq!(
            blocker.blocked_by_category(TrackerCategory::Telemetry),
            1
        );
    }

    #[test]
    fn test_privacy_dashboard_toggle_and_render() {
        let mut blocker = TrackerBlocker::new();
        blocker.intercept("https://google-analytics.com/collect");

        let mut dash = PrivacyDashboard::new();
        assert!(!dash.is_open);
        assert!(dash.render_html(&blocker, "https://example.com").is_empty());

        dash.toggle();
        assert!(dash.is_open);
        let html = dash.render_html(&blocker, "https://example.com");
        assert!(html.contains("PRIVACY SHIELD"));
        assert!(html.contains("google-analytics.com"));
        assert!(html.contains("Total Blocked: 1"));
    }
}
