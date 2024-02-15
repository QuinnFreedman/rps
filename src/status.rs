use crate::{
    colors,
    segments::{Context, PromptSegment, RenderedSegment, ShrinkPriority},
};
use const_format::formatcp;

#[derive(Debug, PartialEq, Eq)]
enum ExitStatus {
    Ok,
    Failed,
}

pub struct StatusSegment {
    status: Vec<ExitStatus>,
}

impl StatusSegment {
    pub fn new(context: &Context) -> Option<Self> {
        let statuses = context.pipestatus.as_ref()?;
        let all_ok = statuses.split_ascii_whitespace().all(|x| x == "0");
        if all_ok {
            None
        } else {
            Some(StatusSegment {
                status: statuses
                    .split_ascii_whitespace()
                    .map(|x| {
                        if x == "0" {
                            ExitStatus::Ok
                        } else {
                            ExitStatus::Failed
                        }
                    })
                    .collect(),
            })
        }
    }

    fn get_unconstrained_size(&self) -> usize {
        self.status.len() * 2 + 1
    }
}

const SUCCESS_SYMBOL: &str = formatcp!("\x1b[{}m\u{2713}", colors::GREEN.fg);
const FAILURE_SYMBOL: &str = formatcp!("\x1b[{}m\u{2718}", colors::RED.fg);

fn render_status(status: &ExitStatus) -> &'static str {
    match status {
        ExitStatus::Ok => SUCCESS_SYMBOL,
        ExitStatus::Failed => FAILURE_SYMBOL,
    }
}

impl PromptSegment for StatusSegment {
    fn get_base_width(&self, shrink: crate::segments::ShrinkPriority) -> usize {
        match shrink {
            ShrinkPriority::Unconstrained => self.get_unconstrained_size(),
            ShrinkPriority::ShrinkConfortable => 3,
            ShrinkPriority::ShrinkBeyondMin => 0,
        }
    }

    fn get_actual_width_when_under(&self, max_size: usize) -> usize {
        if max_size >= self.get_unconstrained_size() {
            self.get_unconstrained_size()
        } else if max_size >= 3 {
            3
        } else {
            0
        }
    }

    fn render_at_size(&self, max_size: usize) -> RenderedSegment {
        let text = if max_size >= self.get_unconstrained_size() {
            self.status
                .iter()
                .map(render_status)
                .intersperse(" ")
                .collect::<String>()
        } else if max_size >= 3 {
            render_status(&self.status[0]).to_string()
        } else {
            String::new()
        };

        RenderedSegment {
            text: format!(" {} ", text),
            bg_color: colors::BLACK,
            fg_color: colors::BLACK,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{segments::Context, status::ExitStatus};

    use super::StatusSegment;

    #[test]
    fn create_segment() {
        let context = Context {
            path: None,
            pipestatus: Some(String::from("0 127 0")),
        };
        let segment = StatusSegment::new(&context).unwrap();
        assert_eq!(segment.status.len(), 3);
        assert_eq!(segment.status[0], ExitStatus::Ok);
        assert_eq!(segment.status[1], ExitStatus::Failed);
        assert_eq!(segment.status[2], ExitStatus::Ok);
    }
}
