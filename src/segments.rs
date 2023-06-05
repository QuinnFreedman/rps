use std::path::PathBuf;

use crate::colors;

pub struct Context {
    pub path: Option<PathBuf>,
}

// pub enum Side {
//     Left,
//     Right,
// }

// pub struct ShrinkStrategy {
//     pub min_width: usize,
//     pub shrink_from: Side,
// }

// pub struct Segment {
//     pub text: String,
//     pub bg_color: colors::Color,
//     pub fg_color: colors::Color,
//     pub shrink: Option<ShrinkStrategy>,
// }

// pub type SegmentBuilder = fn(&Context) -> Option<Segment>;

#[derive(Clone, Copy, Debug)]
pub enum ShrinkPriority {
    Unconstrained,
    ShrinkConfortable,
    ShrinkBeyondMin,
}

pub struct RenderedSegment {
    pub text: String,
    pub bg_color: colors::Color,
    pub fg_color: colors::Color,
}

pub trait PromptSegment {
    fn get_base_width(&self, shrink: ShrinkPriority) -> usize;
    fn get_actual_width_when_under(&self, max_size: usize) -> usize;
    fn render_at_size(&self, max_size: usize) -> RenderedSegment;
}
