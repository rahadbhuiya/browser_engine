pub mod controls;
pub mod pipeline;

pub use controls::{MediaControlHit, MediaControls};
pub use pipeline::{MediaContainerFormat, MediaElementState, MediaType, PlaybackState, VideoFrame};

use crate::dom::{Dom, NodeType};
use std::collections::HashMap;

/// Central manager orchestrating all `<audio>` and `<video>` playback across loaded tabs.
#[derive(Default)]
pub struct MediaManager {
    elements: HashMap<usize, MediaElementState>, // node_idx -> MediaElementState
}

impl MediaManager {
    pub fn new() -> Self {
        MediaManager {
            elements: HashMap::new(),
        }
    }

    /// Scans the DOM for `<audio>` and `<video>` elements and initializes their state.
    pub fn scan_dom(&mut self, dom: &Dom) {
        for (idx, node) in dom.nodes.iter().enumerate() {
            if let NodeType::Element(ref elem) = node.node_type {
                let tag = elem.tag.to_ascii_lowercase();
                if tag == "audio" || tag == "video" {
                    let mtype = if tag == "audio" {
                        MediaType::Audio
                    } else {
                        MediaType::Video
                    };

                    let src = elem.get_attribute("src").unwrap_or("").to_string();

                    let controls =
                        elem.has_attribute("controls") || !elem.has_attribute("nocontrols");
                    let autoplay = elem.has_attribute("autoplay");
                    let loop_pb = elem.has_attribute("loop");
                    let muted = elem.has_attribute("muted");

                    let state = self
                        .elements
                        .entry(idx)
                        .or_insert_with(|| MediaElementState::new(idx, mtype, src.clone()));

                    state.controls = controls;
                    state.autoplay = autoplay;
                    state.loop_playback = loop_pb;
                    state.muted = muted;

                    if state.state == PlaybackState::Empty && !state.src.is_empty() {
                        state.load_source(None, 120.0);
                    }
                }
            }
        }
    }

    pub fn get(&self, node_idx: usize) -> Option<&MediaElementState> {
        self.elements.get(&node_idx)
    }

    pub fn get_mut(&mut self, node_idx: usize) -> Option<&mut MediaElementState> {
        self.elements.get_mut(&node_idx)
    }

    /// Advances playback for all currently playing media elements by `delta_seconds`.
    pub fn tick(&mut self, delta_seconds: f64) {
        for state in self.elements.values_mut() {
            state.advance_playback(delta_seconds);
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::build_dom;
    use crate::html::Tokenizer;
    use crate::layout::Rect;

    #[test]
    fn test_media_container_format_detection() {
        assert_eq!(
            MediaContainerFormat::from_url_or_src("https://example.com/audio.mp3"),
            MediaContainerFormat::Mp3
        );
        assert_eq!(
            MediaContainerFormat::from_url_or_src("video.mp4"),
            MediaContainerFormat::Mp4
        );
        assert_eq!(
            MediaContainerFormat::from_url_or_src("stream.webm"),
            MediaContainerFormat::WebM
        );
        assert_eq!(
            MediaContainerFormat::from_url_or_src("sound.wav"),
            MediaContainerFormat::Wav
        );
        assert_eq!(
            MediaContainerFormat::from_url_or_src("track.ogg"),
            MediaContainerFormat::Ogg
        );

        let wav_header = b"RIFF\x24\x00\x00\x00WAVEfmt ";
        assert_eq!(
            MediaContainerFormat::from_magic_bytes(wav_header),
            MediaContainerFormat::Wav
        );
        let ogg_header = b"OggS\x00\x02\x00\x00";
        assert_eq!(
            MediaContainerFormat::from_magic_bytes(ogg_header),
            MediaContainerFormat::Ogg
        );
    }

    #[test]
    fn test_media_playback_state_machine() {
        let mut state = MediaElementState::new(1, MediaType::Video, "test.mp4".to_string());
        assert_eq!(state.state, PlaybackState::Empty);

        state.load_source(None, 60.0);
        assert_eq!(state.state, PlaybackState::Ready);
        assert_eq!(state.duration, 60.0);

        state.play();
        assert_eq!(state.state, PlaybackState::Playing);

        // Advance 5 seconds
        state.advance_playback(5.0);
        assert_eq!(state.current_time, 5.0);
        assert!(state.current_frame.is_some());

        // Seek
        state.seek(30.0);
        assert_eq!(state.current_time, 30.0);

        // Pause
        state.pause();
        assert_eq!(state.state, PlaybackState::Paused);

        // Seek to end
        state.seek(60.0);
        assert_eq!(state.state, PlaybackState::Ended);
    }

    #[test]
    fn test_media_controls_layout_and_hit_testing() {
        let media_rect = Rect {
            x: 100.0,
            y: 100.0,
            width: 640.0,
            height: 360.0,
        };
        let controls = MediaControls::compute_layout(media_rect, MediaType::Video);

        // Hit play button
        let hit_play = controls.hit_test(
            media_rect,
            controls.play_pause_rect.x + 2.0,
            controls.play_pause_rect.y + 2.0,
        );
        assert_eq!(hit_play, Some(MediaControlHit::PlayPauseButton));

        // Hit seek bar at 50%
        let seek_mid_x = controls.seek_bar_rect.x + controls.seek_bar_rect.width * 0.5;
        let seek_mid_y = controls.seek_bar_rect.y + controls.seek_bar_rect.height * 0.5;
        let hit_seek = controls.hit_test(media_rect, seek_mid_x, seek_mid_y);
        if let Some(MediaControlHit::SeekBar(ratio)) = hit_seek {
            assert!((ratio - 0.5).abs() < 0.05);
        } else {
            panic!("Expected SeekBar hit");
        }

        // Format timecode
        assert_eq!(MediaControls::format_timecode(75.0), "01:15");
        assert_eq!(MediaControls::format_timecode(0.0), "00:00");
    }

    #[test]
    fn test_media_manager_scan_dom() {
        let html = r#"<main><video src="video.mp4" controls autoplay></video><audio src="sound.wav" controls></audio></main>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let mut manager = MediaManager::new();
        manager.scan_dom(&dom);
        assert_eq!(manager.len(), 2);

        // One video, one audio
        let mut has_audio = false;
        let mut has_video = false;
        for (_, elem) in manager.elements.iter() {
            if elem.media_type == MediaType::Audio {
                has_audio = true;
            }
            if elem.media_type == MediaType::Video {
                has_video = true;
            }
        }
        assert!(has_audio);
        assert!(has_video);
    }
}
