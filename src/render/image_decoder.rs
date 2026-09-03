use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct ImageCache {
    cache: HashMap<String, DecodedImage>,
}

impl ImageCache {
    pub fn new() -> Self {
        ImageCache {
            cache: HashMap::new(),
        }
    }

    /// Decodes image bytes (PNG, JPEG, etc.) into an RGBA8 pixel buffer and caches it by URL.
    pub fn load_from_memory(&mut self, url: &str, bytes: &[u8]) -> Result<&DecodedImage, String> {
        if !self.cache.contains_key(url) {
            let img = image::load_from_memory(bytes)
                .map_err(|e| format!("Failed to decode image from memory: {:?}", e))?;
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();
            let decoded = DecodedImage {
                width,
                height,
                rgba: rgba_img.into_raw(),
            };
            self.cache.insert(url.to_string(), decoded);
        }
        Ok(self.cache.get(url).unwrap())
    }

    pub fn get(&self, url: &str) -> Option<&DecodedImage> {
        self.cache.get(url)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
