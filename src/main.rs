#![feature(iter_intersperse)]

mod colors;
mod git;
mod init;
mod jobs;
mod path;
mod segments;
mod status;

use std::{
    cmp::min,
    io::{self, Write},
};

use clap::{Parser, ValueEnum};
use git::GitSegment;
use init::echo_init_script;
use jobs::JobsSegment;
use path::PathSegment;
use segments::*;
use status::StatusSegment;

const SEGMENT_SEPARATOR: char = '\u{E0B0}';
const MIN_WHITESPACE: usize = 40;

#[derive(PartialEq, Eq, Debug)]
enum Line {
    SingleLine,
    SplitLine,
    OverflowLine,
}

struct SegmentLayout<'a> {
    segment: &'a Box<dyn PromptSegment>,
    current_size: usize,
}

type Layout<'a> = Vec<SegmentLayout<'a>>;

impl std::fmt::Debug for SegmentLayout<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.current_size).as_str())
    }
}

fn get_size(layout: &Layout) -> usize {
    layout.iter().map(|x| x.current_size + 1).sum::<usize>() + 1
}

fn amount_can_shrink(segment_layout: &SegmentLayout, shrink_level: ShrinkPriority) -> usize {
    let base_width = segment_layout.segment.get_base_width(shrink_level);
    segment_layout.current_size.saturating_sub(base_width)
}

fn layout_segments(
    segments: &Vec<Box<dyn PromptSegment>>,
    term_width: usize,
    min_whitespace: usize,
) -> (Line, Layout) {
    let mut layout: Layout = segments
        .iter()
        .map(|x| SegmentLayout {
            segment: x,
            current_size: x.get_base_width(ShrinkPriority::Unconstrained),
        })
        .collect();
    let mut prompt_width = get_size(&layout);

    if term_width.saturating_sub(prompt_width) > min_whitespace {
        return (Line::SingleLine, layout);
    }

    for shrink_priority in [
        ShrinkPriority::ShrinkConfortable,
        ShrinkPriority::ShrinkBeyondMin,
    ] {
        while prompt_width > term_width {
            let amount_to_shrink = prompt_width - term_width;
            let to_shrink = layout
                .iter_mut()
                .max_by_key(|segment| amount_can_shrink(&segment, shrink_priority))
                .unwrap();
            let amount_can_shrink = amount_can_shrink(&to_shrink, shrink_priority);
            if amount_can_shrink == 0 {
                break;
            }
            let new_requested_size = to_shrink
                .current_size
                .saturating_sub(min(amount_to_shrink, amount_can_shrink));
            let new_actual_size = to_shrink
                .segment
                .get_actual_width_when_under(new_requested_size);
            to_shrink.current_size = new_actual_size;
            prompt_width = get_size(&layout);
        }
    }

    if prompt_width > term_width {
        return (Line::OverflowLine, layout);
    }

    (Line::SplitLine, layout)
}

fn set_stdout_color(fg: &colors::Color, bg: &colors::Color) {
    // if *fg == colors::DEFAULT {
    //     print!("%f");
    // } else {
    //     print!("%F{{{}}}", fg.name);
    // }
    // if *bg == colors::DEFAULT {
    //     print!("%k");
    // } else {
    //     print!("%K{{{}}}", bg.name);
    // }
    print!("\x1b[{}m\x1b[{}m", fg.fg, bg.bg);
}

fn reset_stdout_color() {
    // print!("%{{%f%k%}}");
    print!("\x1b[0m");
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

/// Terminal prompt in rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Echo init script for given shell
    #[arg(long, value_enum, value_name = "SHELL")]
    init: Option<Shell>,

    /// Status or pipestatus from the last run process
    #[arg(short, long, value_name = "PIPESTATUS")]
    status: Option<String>,

    /// The width of the terminal window, in characters
    #[arg(short, long, value_name = "COLS")]
    columns: Option<usize>,

    /// The number of background jobs, from jobs -l | wc -l
    #[arg(short, long, value_name = "JOBS")]
    jobs: Option<usize>,
}

fn main() {
    let args = Args::parse();

    if let Some(shell) = args.init {
        echo_init_script(shell);
        return;
    }

    // print!("{} ", args.columns.unwrap_or(0));
    // set_stdout_color(&colors::DEFAULT, &colors::BLUE);
    // print!(">>>>>>>>>>>");
    // println!("%{{%b%f%}}");
    // println!("> ");
    // return;

    let context = Context {
        path: std::env::current_dir().ok(),
        pipestatus: args.status,
        jobs: args.jobs.unwrap_or(0),
    };

    let segments: Vec<Box<dyn PromptSegment>> = vec![
        StatusSegment::new(&context).map(|x| Box::new(x) as Box<dyn PromptSegment>),
        JobsSegment::new(&context).map(|x| Box::new(x) as Box<dyn PromptSegment>),
        PathSegment::new(&context).map(|x| Box::new(x) as Box<dyn PromptSegment>),
        GitSegment::new(&context).map(|x| Box::new(x) as Box<dyn PromptSegment>),
    ]
    .iter_mut()
    .filter_map(|x| x.take())
    .collect();

    let (line_type, layout) = layout_segments(
        &segments,
        args.columns.map(|x| x - 3).unwrap_or(usize::MAX),
        MIN_WHITESPACE,
    );

    if line_type == Line::OverflowLine {
        set_stdout_color(&colors::DEFAULT, &colors::BLUE);
        print!("{}", SEGMENT_SEPARATOR);
        reset_stdout_color();
        return;
    }

    let rendered: Vec<RenderedSegment> = layout
        .iter()
        .map(|x| x.segment.render_at_size(x.current_size))
        .collect();

    for (i, segment) in rendered.iter().enumerate() {
        set_stdout_color(&segment.fg_color, &segment.bg_color);
        print!("{}", segment.text);
        let next_bg_color = rendered
            .get(i + 1)
            .map_or(colors::DEFAULT, |x| x.bg_color.clone());
        set_stdout_color(&segment.bg_color, &next_bg_color);
        print!("{}", SEGMENT_SEPARATOR);
    }

    reset_stdout_color();
    match line_type {
        Line::SingleLine => {
            print!(" ");
        }
        _ => {
            print!("\n");
            set_stdout_color(&colors::BLACK, &colors::BLUE);
            print!(" ↳ ");
            set_stdout_color(&colors::BLUE, &colors::DEFAULT);
            print!("{}", SEGMENT_SEPARATOR);
            reset_stdout_color();
            print!(" ");
        }
    }

    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use crate::{
        layout_segments,
        segments::{PromptSegment, ShrinkPriority},
        Line,
    };

    const MIN_TEST_SEGMENT_SIZE: usize = 5;
    struct TestSegment {
        width: usize,
    }
    impl PromptSegment for TestSegment {
        fn get_base_width(&self, shrink: crate::segments::ShrinkPriority) -> usize {
            match shrink {
                ShrinkPriority::Unconstrained => self.width,
                ShrinkPriority::ShrinkConfortable => MIN_TEST_SEGMENT_SIZE,
                ShrinkPriority::ShrinkBeyondMin => 1,
            }
        }

        fn get_actual_width_when_under(&self, max_size: usize) -> usize {
            if max_size >= MIN_TEST_SEGMENT_SIZE {
                max_size
            } else {
                1
            }
        }

        fn render_at_size(&self, _max_size: usize) -> crate::segments::RenderedSegment {
            todo!()
        }
    }

    #[test]
    fn layout_segments_one_line() {
        let segments = vec![Box::new(TestSegment { width: 10 }) as Box<dyn PromptSegment>];

        let (line_type, layout) = layout_segments(&segments, 20, 5);
        assert_eq!(line_type, Line::SingleLine);
        assert_eq!(layout[0].current_size, 10);
    }

    #[test]
    fn layout_segments_split_line() {
        let segments = vec![Box::new(TestSegment { width: 10 }) as Box<dyn PromptSegment>];

        let (line_type, layout) = layout_segments(&segments, 20, 10);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, 10);
    }

    #[test]
    fn layout_segments_shrink_comfortable() {
        let segments = vec![Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>];

        let (line_type, layout) = layout_segments(&segments, 20, 10);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, 18);
    }

    #[test]
    fn layout_segments_shrink_small() {
        let segments = vec![Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>];

        let (line_type, layout) = layout_segments(&segments, 6, 10);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, 1);
    }

    #[test]
    fn layout_multiple_segments_shrink_one() {
        let segments = vec![
            Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>,
            Box::new(TestSegment { width: 30 }) as Box<dyn PromptSegment>,
        ];

        let (line_type, layout) = layout_segments(&segments, 50, 40);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, 25);
        assert_eq!(layout[1].current_size, 22);
    }

    #[test]
    fn layout_multiple_segments_shrink_both() {
        let segments = vec![
            Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>,
            Box::new(TestSegment { width: 30 }) as Box<dyn PromptSegment>,
        ];

        let (line_type, layout) = layout_segments(&segments, 25, 40);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, 25 - MIN_TEST_SEGMENT_SIZE - 3);
        assert_eq!(layout[1].current_size, MIN_TEST_SEGMENT_SIZE);
    }

    #[test]
    fn layout_multiple_segments_shrink_one_small() {
        let segments = vec![
            Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>,
            Box::new(TestSegment { width: 30 }) as Box<dyn PromptSegment>,
        ];

        let (line_type, layout) = layout_segments(&segments, 10, 40);
        assert_eq!(line_type, Line::SplitLine);
        assert_eq!(layout[0].current_size, MIN_TEST_SEGMENT_SIZE);
        assert_eq!(layout[1].current_size, 1);
    }

    #[test]
    fn layout_multiple_segments_overflow() {
        let segments = vec![
            Box::new(TestSegment { width: 25 }) as Box<dyn PromptSegment>,
            Box::new(TestSegment { width: 30 }) as Box<dyn PromptSegment>,
        ];

        let (line_type, layout) = layout_segments(&segments, 3, 40);
        assert_eq!(line_type, Line::OverflowLine);
        assert_eq!(layout[0].current_size, 1);
        assert_eq!(layout[1].current_size, 1);
    }
}
