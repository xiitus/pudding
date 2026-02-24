use ratatui::layout::Rect;

use crate::model::Orientation;

pub const MIN_RATIO: f32 = 0.1;
pub const MAX_RATIO: f32 = 0.9;

pub fn clamp_ratio(ratio: f32) -> f32 {
    ratio.clamp(MIN_RATIO, MAX_RATIO)
}

pub fn split_rect(rect: Rect, orientation: Orientation, ratio: f32) -> (Rect, Rect) {
    let ratio = clamp_ratio(ratio);
    match orientation {
        Orientation::Vertical => {
            if rect.width <= 1 {
                return (
                    Rect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    Rect {
                        x: rect.x + rect.width,
                        y: rect.y,
                        width: 0,
                        height: rect.height,
                    },
                );
            }
            let mut w1 = (rect.width as f32 * ratio).round() as u16;
            if w1 < 1 {
                w1 = 1;
            }
            if w1 >= rect.width {
                w1 = rect.width - 1;
            }
            let r1 = Rect {
                x: rect.x,
                y: rect.y,
                width: w1,
                height: rect.height,
            };
            let r2 = Rect {
                x: rect.x + w1,
                y: rect.y,
                width: rect.width - w1,
                height: rect.height,
            };
            (r1, r2)
        }
        Orientation::Horizontal => {
            if rect.height <= 1 {
                return (
                    Rect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    Rect {
                        x: rect.x,
                        y: rect.y + rect.height,
                        width: rect.width,
                        height: 0,
                    },
                );
            }
            let mut h1 = (rect.height as f32 * ratio).round() as u16;
            if h1 < 1 {
                h1 = 1;
            }
            if h1 >= rect.height {
                h1 = rect.height - 1;
            }
            let r1 = Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: h1,
            };
            let r2 = Rect {
                x: rect.x,
                y: rect.y + h1,
                width: rect.width,
                height: rect.height - h1,
            };
            (r1, r2)
        }
    }
}

pub fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use crate::layout::{clamp_ratio, split_rect};
    use crate::model::Orientation;

    #[test]
    fn split_rect_small_width_no_underflow() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 1,
            height: 10,
        };
        let (left, right) = split_rect(rect, Orientation::Vertical, 0.5);
        assert_eq!(left.width, 1);
        assert_eq!(right.width, 0);
    }

    #[test]
    fn split_rect_small_height_no_underflow() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
        };
        let (top, bottom) = split_rect(rect, Orientation::Horizontal, 0.5);
        assert_eq!(top.height, 1);
        assert_eq!(bottom.height, 0);
    }

    #[test]
    fn clamp_ratio_bounds() {
        assert_eq!(clamp_ratio(0.0), 0.1);
        assert_eq!(clamp_ratio(0.1), 0.1);
        assert_eq!(clamp_ratio(0.5), 0.5);
        assert_eq!(clamp_ratio(0.9), 0.9);
        assert_eq!(clamp_ratio(1.0), 0.9);
    }

    #[test]
    fn split_rect_uses_clamped_ratio_bounds() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };

        let (low_left, low_right) = split_rect(rect, Orientation::Vertical, 0.0);
        assert_eq!(low_left.width, 1);
        assert_eq!(low_right.width, 9);

        let (high_left, high_right) = split_rect(rect, Orientation::Vertical, 1.0);
        assert_eq!(high_left.width, 9);
        assert_eq!(high_right.width, 1);
    }

    #[test]
    fn clamp_ratio_handles_far_out_of_range_values() {
        assert_eq!(clamp_ratio(-10.0), 0.1);
        assert_eq!(clamp_ratio(10.0), 0.9);
    }
}
