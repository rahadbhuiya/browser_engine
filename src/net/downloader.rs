use std::fs::File;
use std::io::Write;
use super::fetch;

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadItem {
    pub url: String,
    pub filename: String,
    pub bytes_downloaded: usize,
    pub completed: bool,
}

pub struct DownloadManager {
    pub downloads: Vec<DownloadItem>,
}

impl DownloadManager {
    pub fn new() -> Self {
        DownloadManager { downloads: Vec::new() }
    }

    pub fn download_file(&mut self, url: &str, dest_path: &str) -> Result<usize, String> {
        let resp = fetch(url)?;
        let bytes = resp.body;
        let mut file = File::create(dest_path).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;

        let filename = dest_path.split('\\').last().unwrap_or(dest_path).to_string();
        self.downloads.push(DownloadItem {
            url: url.to_string(),
            filename,
            bytes_downloaded: bytes.len(),
            completed: true,
        });

        Ok(bytes.len())
    }

    pub fn render_downloads_bar_html(&self) -> String {
        if self.downloads.is_empty() {
            return String::new();
        }
        let mut html = String::from(r#"<div class="downloads-bar" style="background-color: #0f172a; padding: 4px 12px; border-top: 1px solid #1e293b;">"#);
        html.push_str(r#"<span style="font-weight: bold; color: #38bdf8; margin-right: 10px; font-size: 12px;">📥 DOWNLOADS:</span>"#);
        for item in &self.downloads {
            html.push_str(&format!(
                r#"<span style="background-color: #1e293b; color: #4ade80; border: 1px solid #334155; padding: 2px 8px; margin-right: 6px; font-size: 12px;">✓ {} ({} B)</span>"#,
                item.filename, item.bytes_downloaded
            ));
        }
        html.push_str(r#"</div>"#);
        html
    }
}
