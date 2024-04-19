use crate::{
    colors,
    segments::{Context, PromptSegment, RenderedSegment, ShrinkPriority},
};

pub struct JobsSegment {
    jobs: usize,
}

impl JobsSegment {
    pub fn new(context: &Context) -> Option<Self> {
        if context.jobs == 0 {
            None
        } else {
            Some(JobsSegment { jobs: context.jobs })
        }
    }

    fn get_unconstrained_size(&self) -> usize {
        format!("{}", self.jobs).len() + 4
    }
}

impl PromptSegment for JobsSegment {
    fn get_base_width(&self, shrink: crate::segments::ShrinkPriority) -> usize {
        match shrink {
            ShrinkPriority::Unconstrained => match self.jobs {
                1 => 3,
                _ => self.get_unconstrained_size(),
            },
            ShrinkPriority::ShrinkComfortable => 3,
            ShrinkPriority::ShrinkBeyondMin => 0,
        }
    }

    fn get_actual_width_when_under(&self, max_size: usize) -> usize {
        if max_size >= self.get_unconstrained_size() + 2 {
            self.get_unconstrained_size() + 2
        } else if max_size >= 3 {
            3
        } else {
            0
        }
    }

    fn render_at_size(&self, max_size: usize) -> RenderedSegment {
        let text = if max_size >= self.get_unconstrained_size() {
            if self.jobs == 1 {
                String::from(" ⚙ ")
            } else {
                format!(" {} ⚙ ", self.jobs)
            }
        } else if max_size >= 3 {
            String::from(" ⚙ ")
        } else {
            String::new()
        };

        RenderedSegment {
            text,
            bg_color: colors::BLACK,
            fg_color: colors::YELLOW,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::segments::{Context, PromptSegment, ShrinkPriority};

    use super::JobsSegment;

    #[test]
    fn render_single_job() {
        let context = Context {
            path: None,
            pipestatus: None,
            jobs: 1,
        };
        let segment = JobsSegment::new(&context).unwrap();
        assert_eq!(segment.get_base_width(ShrinkPriority::Unconstrained), 3);
        assert_eq!(segment.render_at_size(3).text, " ⚙ ");
    }

    #[test]
    fn render_multiple_jobs() {
        let context = Context {
            path: None,
            pipestatus: None,
            jobs: 3,
        };
        let segment = JobsSegment::new(&context).unwrap();
        assert_eq!(segment.get_base_width(ShrinkPriority::Unconstrained), 5);
        assert_eq!(segment.render_at_size(5).text, " 3 ⚙ ");
    }

    #[test]
    fn render_constrained() {
        let context = Context {
            path: None,
            pipestatus: None,
            jobs: 3,
        };
        let segment = JobsSegment::new(&context).unwrap();
        assert_eq!(segment.get_base_width(ShrinkPriority::ShrinkComfortable), 3);
        assert_eq!(segment.render_at_size(3).text, " ⚙ ");
    }
}
