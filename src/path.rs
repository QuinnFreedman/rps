use std::{borrow::Cow, cmp::min, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    colors,
    segments::{Context, PromptSegment, RenderedSegment, ShrinkPriority},
};

const PATH_SEPARATOR: char = '\u{E0B1}';
const MIN_PATH_SIZE: usize = 5;

#[derive(Debug, PartialEq)]
enum PathType {
    RelativeToHome,
    RelativeToRoot,
}

fn get_relative_path(cwd: impl Into<PathBuf>, home: impl Into<PathBuf>) -> (PathType, PathBuf) {
    let cwd = cwd.into();
    if let Ok(relative) = cwd.strip_prefix(home.into()) {
        return (PathType::RelativeToHome, relative.to_path_buf());
    };

    if let Ok(relative) = cwd.strip_prefix("/") {
        return (PathType::RelativeToRoot, relative.to_path_buf());
    };

    (PathType::RelativeToRoot, cwd)
}

fn get_path_relative_to_home(cwd: &PathBuf) -> (PathType, Cow<PathBuf>) {
    #[allow(deprecated)]
    match std::env::home_dir() {
        Some(home) => {
            let relative_path = get_relative_path(cwd, &home);
            (relative_path.0, Cow::Owned(relative_path.1))
        }
        None => (PathType::RelativeToRoot, Cow::Borrowed(cwd)),
    }
}

pub struct PathSegment {
    path_segments: Vec<String>,
    path_type: PathType,
    preferred_width: usize,
}

fn calculate_preferred_size(components: &Vec<String>) -> usize {
    components
        .iter()
        .map(|x| x.graphemes(true).count() + 3)
        .sum::<usize>()
        + 3
}

impl PathSegment {
    pub fn new(context: &Context) -> Option<Self> {
        let (path_type, path_buf) = get_path_relative_to_home(context.path.as_ref()?);

        let components: Vec<String> = path_buf
            .iter()
            .map(|x| x.to_string_lossy().into_owned())
            .collect();

        let preferred_width = calculate_preferred_size(&components);

        Some(PathSegment {
            path_segments: components,
            path_type,
            preferred_width,
        })
    }
}

impl PromptSegment for PathSegment {
    fn get_base_width(&self, shrink: ShrinkPriority) -> usize {
        match shrink {
            ShrinkPriority::Unconstrained => self.preferred_width,
            ShrinkPriority::ShrinkConfortable => MIN_PATH_SIZE,
            ShrinkPriority::ShrinkBeyondMin => 1,
        }
    }

    fn get_actual_width_when_under(&self, max_size: usize) -> usize {
        if max_size >= MIN_PATH_SIZE {
            min(max_size, self.preferred_width)
        } else {
            1
        }
    }

    fn render_at_size(&self, max_size: usize) -> RenderedSegment {
        let separator = format!(" {} ", PATH_SEPARATOR);

        let prefix_char = match self.path_type {
            PathType::RelativeToHome => '~',
            PathType::RelativeToRoot => '/',
        };

        let text = if max_size >= self.preferred_width {
            let full_text = self.path_segments.join(separator.as_str());
            format!(" {}{}{} ", prefix_char, separator, full_text)
        } else if max_size > MIN_PATH_SIZE {
            let mut string_builder: Vec<&str> = vec![" "];
            let mut current_size = 1;
            'outer: for segment in self.path_segments.iter().rev() {
                for c in segment
                    .graphemes(true)
                    .rev()
                    .chain(separator.graphemes(true).rev())
                {
                    string_builder.push(c);
                    current_size += 1;
                    if current_size + 4 >= max_size {
                        break 'outer;
                    }
                }
            }
            string_builder.push(" ...");
            string_builder.into_iter().rev().collect()
        } else {
            " ".to_string()
        };
        debug_assert_eq!(
            text.graphemes(true).count(),
            self.get_actual_width_when_under(max_size)
        );
        RenderedSegment {
            text,
            bg_color: colors::BLUE,
            fg_color: colors::BLACK,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        path::{calculate_preferred_size, get_relative_path, PathType, PATH_SEPARATOR},
        segments::PromptSegment,
    };

    use super::PathSegment;

    #[test]
    fn format_relative_to_home() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/home/me/foo/bar");
        let (path_type, path) = get_relative_path(cwd, home);
        assert_eq!(path_type, PathType::RelativeToHome);
        assert_eq!(path, PathBuf::from("foo/bar"))
    }

    #[test]
    fn format_is_home() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/home/me");
        let (path_type, path) = get_relative_path(cwd, home);
        assert_eq!(path_type, PathType::RelativeToHome);
        assert_eq!(path, PathBuf::new())
    }

    #[test]
    fn format_relative_to_root() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/foo/bar/baz");
        let (path_type, path) = get_relative_path(cwd, home);
        assert_eq!(path_type, PathType::RelativeToRoot);
        assert_eq!(path, PathBuf::from("foo/bar/baz"))
    }

    #[test]
    fn format_is_root() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/");
        let (path_type, path) = get_relative_path(cwd, home);
        assert_eq!(path_type, PathType::RelativeToRoot);
        assert_eq!(path, PathBuf::new())
    }

    #[test]
    fn render_single() {
        let segment = PathSegment {
            path_segments: vec!["1234567890".to_string()],
            path_type: PathType::RelativeToHome,
            preferred_width: 16,
        };
        let rendered = segment.render_at_size(10);
        assert_eq!(rendered.text, " ...67890 ");
    }

    #[test]
    fn render_multiple() {
        let parts = vec!["1234567890".to_string(), "1234".to_string()];
        let preferred_width = calculate_preferred_size(&parts);
        let segment = PathSegment {
            path_segments: parts,
            path_type: PathType::RelativeToHome,
            preferred_width,
        };
        let rendered = segment.render_at_size(16);
        assert_eq!(rendered.text, format!(" ...7890 {} 1234 ", PATH_SEPARATOR));
    }

    #[test]
    fn render_unconstrained() {
        let parts = vec!["1234567890".to_string(), "1234".to_string()];
        let preferred_width = calculate_preferred_size(&parts);
        let segment = PathSegment {
            path_segments: parts,
            path_type: PathType::RelativeToHome,
            preferred_width,
        };
        let rendered = segment.render_at_size(99);
        assert_eq!(
            rendered.text,
            format!(" ~ {0} 1234567890 {0} 1234 ", PATH_SEPARATOR)
        );
    }
}
