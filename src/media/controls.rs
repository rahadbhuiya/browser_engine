use super::pipeline::{MediaElementState, MediaType, PlaybackState};
use crate::layout::Rect;
use crate::paint::{Color, PaintCommand};

/// Hit test target on visual media controls overlay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaControlHit {
    PlayPauseButton,
    SeekBar(f32), // normalized 0.0 to 1.0 progress click
    MuteButton,
    VolumeSlider(f32), // normalized 0.0 to 1.0 volume click
    VideoSurface,
}

/// Visual control bar layout dimensions and paint command generator.
pub struct MediaControls {
    pub bounds: Rect,
    pub play_pause_rect: Rect,
    pub seek_bar_rect: Rect,
    pub timecode_rect: Rect,
    pub mute_rect: Rect,
    pub volume_rect: Rect,
}

impl MediaControls {
    pub fn compute_layout(media_rect: Rect, media_type: MediaType) -> Self {
        let bar_height: f32 = 36.0;
        let bounds = if media_type == MediaType::Audio {
            Rect {
                x: media_rect.x,
                y: media_rect.y,
                width: media_rect.width.max(300.0),
                height: bar_height.max(media_rect.height),
            }
        } else {
            // Video overlay controls at the bottom of the video frame
            Rect {
                x: media_rect.x,
                y: media_rect.y + media_rect.height - bar_height,
                width: media_rect.width,
                height: bar_height,
            }
        };

        let pad = 6.0;
        let btn_size = 24.0;

        let play_pause_rect = Rect {
            x: bounds.x + pad,
            y: bounds.y + (bounds.height - btn_size) / 2.0,
            width: btn_size,
            height: btn_size,
        };

        let mute_rect = Rect {
            x: bounds.x + bounds.width - btn_size - pad,
            y: bounds.y + (bounds.height - btn_size) / 2.0,
            width: btn_size,
            height: btn_size,
        };

        let timecode_width = 80.0;
        let timecode_rect = Rect {
            x: mute_rect.x - timecode_width - pad,
            y: bounds.y + (bounds.height - 16.0) / 2.0,
            width: timecode_width,
            height: 16.0,
        };

        let seek_x = play_pause_rect.x + play_pause_rect.width + pad * 2.0;
        let seek_width = (timecode_rect.x - seek_x - pad * 2.0).max(40.0);
        let seek_bar_rect = Rect {
            x: seek_x,
            y: bounds.y + (bounds.height - 8.0) / 2.0,
            width: seek_width,
            height: 8.0,
        };

        let volume_rect = Rect {
            x: mute_rect.x,
            y: mute_rect.y,
            width: mute_rect.width,
            height: mute_rect.height,
        };

        MediaControls {
            bounds,
            play_pause_rect,
            seek_bar_rect,
            timecode_rect,
            mute_rect,
            volume_rect,
        }
    }

    /// Performs hit testing against interactive control elements.
    pub fn hit_test(
        &self,
        media_rect: Rect,
        click_x: f32,
        click_y: f32,
    ) -> Option<MediaControlHit> {
        if self.play_pause_rect.contains(click_x, click_y) {
            return Some(MediaControlHit::PlayPauseButton);
        }

        if self.mute_rect.contains(click_x, click_y) {
            return Some(MediaControlHit::MuteButton);
        }

        if click_x >= self.seek_bar_rect.x - 4.0
            && click_x <= self.seek_bar_rect.x + self.seek_bar_rect.width + 4.0
            && click_y >= self.seek_bar_rect.y - 6.0
            && click_y <= self.seek_bar_rect.y + self.seek_bar_rect.height + 6.0
        {
            let normalized =
                ((click_x - self.seek_bar_rect.x) / self.seek_bar_rect.width).clamp(0.0, 1.0);
            return Some(MediaControlHit::SeekBar(normalized));
        }

        if media_rect.contains(click_x, click_y) {
            return Some(MediaControlHit::VideoSurface);
        }

        None
    }

    /// Formats seconds into standard MM:SS display timecode.
    pub fn format_timecode(seconds: f64) -> String {
        let total_secs = seconds.max(0.0).round() as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Generates high-fidelity paint commands to render the controls bar.
    pub fn build_paint_commands(
        &self,
        state: &MediaElementState,
        media_rect: Rect,
    ) -> Vec<PaintCommand> {
        let mut cmds = Vec::new();

        // 1. If Video, render video surface frame
        if state.media_type == MediaType::Video {
            let bg_color = Color::new(0.05, 0.06, 0.08, 1.0);
            cmds.push(PaintCommand::DrawRect {
                rect: media_rect,
                color: bg_color,
                border_radius: 4.0,
                is_fixed: false,
            });

            // If not playing, render subtle center play glyph
            if state.state != PlaybackState::Playing {
                let center_x = media_rect.x + media_rect.width / 2.0 - 10.0;
                let center_y = media_rect.y + media_rect.height / 2.0 - 10.0;
                cmds.push(PaintCommand::DrawText {
                    text: "▶".to_string(),
                    rect: Rect {
                        x: center_x,
                        y: center_y,
                        width: 24.0,
                        height: 24.0,
                    },
                    color: Color::new(1.0, 1.0, 1.0, 0.8),
                    font_size: 24.0,
                    is_fixed: false,
                });
            }
        }

        if !state.controls {
            return cmds;
        }

        // 2. Control bar background
        let bar_bg = if state.media_type == MediaType::Audio {
            Color::new(0.12, 0.14, 0.18, 1.0)
        } else {
            Color::new(0.08, 0.09, 0.12, 0.88)
        };
        cmds.push(PaintCommand::DrawRect {
            rect: self.bounds,
            color: bar_bg,
            border_radius: 4.0,
            is_fixed: false,
        });

        // 3. Play / Pause button
        let play_glyph = if state.state == PlaybackState::Playing {
            "❚❚"
        } else {
            "▶"
        };
        cmds.push(PaintCommand::DrawText {
            text: play_glyph.to_string(),
            rect: self.play_pause_rect,
            color: Color::white(),
            font_size: 14.0,
            is_fixed: false,
        });

        // 4. Seek Bar Track (background)
        cmds.push(PaintCommand::DrawRect {
            rect: self.seek_bar_rect,
            color: Color::new(0.3, 0.33, 0.4, 0.6),
            border_radius: 3.0,
            is_fixed: false,
        });

        // 5. Seek Bar Buffered Track
        if state.duration > 0.0 && state.buffered_time > 0.0 {
            let buf_ratio = (state.buffered_time / state.duration).clamp(0.0, 1.0) as f32;
            let buf_rect = Rect {
                x: self.seek_bar_rect.x,
                y: self.seek_bar_rect.y,
                width: self.seek_bar_rect.width * buf_ratio,
                height: self.seek_bar_rect.height,
            };
            cmds.push(PaintCommand::DrawRect {
                rect: buf_rect,
                color: Color::new(0.5, 0.55, 0.65, 0.5),
                border_radius: 3.0,
                is_fixed: false,
            });
        }

        // 6. Seek Bar Progress Fill
        if state.duration > 0.0 {
            let progress_ratio = (state.current_time / state.duration).clamp(0.0, 1.0) as f32;
            let fill_rect = Rect {
                x: self.seek_bar_rect.x,
                y: self.seek_bar_rect.y,
                width: self.seek_bar_rect.width * progress_ratio,
                height: self.seek_bar_rect.height,
            };
            let accent_color = Color::new(0.2, 0.6, 1.0, 1.0); // Vibrant blue
            cmds.push(PaintCommand::DrawRect {
                rect: fill_rect,
                color: accent_color,
                border_radius: 3.0,
                is_fixed: false,
            });

            // Scrubber knob
            let knob_size = 10.0;
            let knob_rect = Rect {
                x: (fill_rect.x + fill_rect.width - knob_size / 2.0).clamp(
                    self.seek_bar_rect.x,
                    self.seek_bar_rect.x + self.seek_bar_rect.width - knob_size,
                ),
                y: self.seek_bar_rect.y - 1.0,
                width: knob_size,
                height: knob_size,
            };
            cmds.push(PaintCommand::DrawRect {
                rect: knob_rect,
                color: Color::white(),
                border_radius: 5.0,
                is_fixed: false,
            });
        }

        // 7. Timecode text
        let time_str = format!(
            "{}/{}",
            Self::format_timecode(state.current_time),
            Self::format_timecode(state.duration)
        );
        cmds.push(PaintCommand::DrawText {
            text: time_str,
            rect: self.timecode_rect,
            color: Color::new(0.85, 0.88, 0.92, 1.0),
            font_size: 11.0,
            is_fixed: false,
        });

        // 8. Mute / Volume icon
        let vol_glyph = if state.muted || state.volume == 0.0 {
            "[M]"
        } else {
            "[V]"
        };
        cmds.push(PaintCommand::DrawText {
            text: vol_glyph.to_string(),
            rect: self.mute_rect,
            color: Color::white(),
            font_size: 13.0,
            is_fixed: false,
        });

        cmds
    }
}
