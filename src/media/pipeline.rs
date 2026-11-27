/// Audio and Video stream demuxing, decoding pipeline, and playback state machine.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Audio,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Empty,
    Loading,
    Ready,
    Playing,
    Paused,
    Ended,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaContainerFormat {
    Wav,
    Ogg,
    WebM,
    Mp4,
    Mp3,
    Unknown,
}

impl MediaContainerFormat {
    pub fn from_magic_bytes(header: &[u8]) -> Self {
        if header.len() >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WAVE" {
            MediaContainerFormat::Wav
        } else if header.len() >= 4 && &header[0..4] == b"OggS" {
            MediaContainerFormat::Ogg
        } else if header.len() >= 4 && header[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
            MediaContainerFormat::WebM
        } else if header.len() >= 8 && (&header[4..8] == b"ftyp" || &header[4..8] == b"isom") {
            MediaContainerFormat::Mp4
        } else if header.len() >= 3 && &header[0..3] == b"ID3" {
            MediaContainerFormat::Mp3
        } else {
            MediaContainerFormat::Unknown
        }
    }

    pub fn from_url_or_src(src: &str) -> Self {
        let clean = src.split('?').next().unwrap_or(src).to_ascii_lowercase();
        if clean.ends_with(".wav") {
            MediaContainerFormat::Wav
        } else if clean.ends_with(".ogg") || clean.ends_with(".ogv") || clean.ends_with(".oga") {
            MediaContainerFormat::Ogg
        } else if clean.ends_with(".webm") {
            MediaContainerFormat::WebM
        } else if clean.ends_with(".mp4") || clean.ends_with(".m4v") {
            MediaContainerFormat::Mp4
        } else if clean.ends_with(".mp3") {
            MediaContainerFormat::Mp3
        } else {
            MediaContainerFormat::Unknown
        }
    }
}

/// A decoded video frame ready for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
    pub rgba_data: Vec<u8>,
}

impl VideoFrame {
    pub fn new_colored(width: u32, height: u32, timestamp_ms: u64, r: u8, g: u8, b: u8) -> Self {
        let len = (width * height * 4) as usize;
        let mut rgba = Vec::with_capacity(len);
        for _ in 0..(width * height) {
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(255);
        }
        VideoFrame {
            width,
            height,
            timestamp_ms,
            rgba_data: rgba,
        }
    }
}

/// Full interactive state of an `<audio>` or `<video>` DOM element.
#[derive(Debug, Clone)]
pub struct MediaElementState {
    pub node_idx: usize,
    pub media_type: MediaType,
    pub src: String,
    pub format: MediaContainerFormat,
    pub autoplay: bool,
    pub controls: bool,
    pub loop_playback: bool,
    pub muted: bool,
    pub volume: f32,       // 0.0 to 1.0
    pub current_time: f64, // seconds
    pub duration: f64,     // seconds
    pub playback_rate: f32,
    pub state: PlaybackState,
    pub buffered_time: f64,
    pub current_frame: Option<VideoFrame>,
}

impl MediaElementState {
    pub fn new(node_idx: usize, media_type: MediaType, src: String) -> Self {
        let format = MediaContainerFormat::from_url_or_src(&src);
        MediaElementState {
            node_idx,
            media_type,
            src,
            format,
            autoplay: false,
            controls: true,
            loop_playback: false,
            muted: false,
            volume: 1.0,
            current_time: 0.0,
            duration: 0.0,
            playback_rate: 1.0,
            state: PlaybackState::Empty,
            buffered_time: 0.0,
            current_frame: None,
        }
    }

    /// Loads media source, determines container header, and sets default duration.
    pub fn load_source(&mut self, data: Option<&[u8]>, estimated_duration: f64) {
        self.state = PlaybackState::Loading;
        if let Some(bytes) = data {
            let detected = MediaContainerFormat::from_magic_bytes(bytes);
            if detected != MediaContainerFormat::Unknown {
                self.format = detected;
            }
        }
        self.duration = if estimated_duration > 0.0 {
            estimated_duration
        } else {
            180.0
        }; // default 3 min
        self.buffered_time = self.duration.min(30.0); // initial 30s buffered
        self.state = PlaybackState::Ready;

        if self.media_type == MediaType::Video {
            // Generate initial poster frame
            self.current_frame = Some(VideoFrame::new_colored(640, 360, 0, 30, 35, 45));
        }

        if self.autoplay {
            self.play();
        }
    }

    pub fn play(&mut self) {
        if self.state == PlaybackState::Ended {
            self.current_time = 0.0;
        }
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    pub fn toggle_playback(&mut self) {
        if self.state == PlaybackState::Playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn seek(&mut self, target_seconds: f64) {
        let clamped = target_seconds.clamp(0.0, self.duration);
        self.current_time = clamped;
        if self.current_time >= self.duration {
            if self.loop_playback {
                self.current_time = 0.0;
                self.state = PlaybackState::Playing;
            } else {
                self.state = PlaybackState::Ended;
            }
        } else if self.state == PlaybackState::Ended {
            self.state = PlaybackState::Paused;
        }

        // Update frame timestamp
        if self.media_type == MediaType::Video {
            let ts = (self.current_time * 1000.0) as u64;
            let shade = ((self.current_time * 15.0) as u8) % 200 + 40;
            self.current_frame = Some(VideoFrame::new_colored(
                640,
                360,
                ts,
                shade,
                shade / 2,
                shade + 10,
            ));
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if self.volume > 0.0 && self.muted {
            self.muted = false;
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn advance_playback(&mut self, delta_seconds: f64) -> bool {
        if self.state != PlaybackState::Playing {
            return false;
        }

        let step = delta_seconds * (self.playback_rate as f64);
        self.current_time += step;

        // Auto-buffer simulation
        if self.current_time + 10.0 > self.buffered_time && self.buffered_time < self.duration {
            self.buffered_time = (self.buffered_time + 15.0).min(self.duration);
        }

        if self.current_time >= self.duration {
            if self.loop_playback {
                self.current_time = 0.0;
            } else {
                self.current_time = self.duration;
                self.state = PlaybackState::Ended;
            }
        }

        // Update frame
        if self.media_type == MediaType::Video {
            let ts = (self.current_time * 1000.0) as u64;
            let shade = ((self.current_time * 20.0) as u8) % 180 + 50;
            self.current_frame = Some(VideoFrame::new_colored(640, 360, ts, shade, 70, 120));
        }

        true
    }
}
