use std::{borrow::Cow, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    colors,
    segments::{Context, PromptSegment, RenderedSegment, ShrinkPriority},
};

const PATH_SEPARATOR: char = '\u{E0B1}';
const MIN_PATH_SIZE: usize = 6;

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

        Some(Self::new_from_path(path_type, path_buf))
    }

    fn new_from_path(path_type: PathType, path_buf: Cow<PathBuf>) -> Self {
        let components: Vec<String> = path_buf
            .iter()
            .map(|x| x.to_string_lossy().into_owned())
            .collect();

        let preferred_width = calculate_preferred_size(&components);

        PathSegment {
            path_segments: components,
            path_type,
            preferred_width,
        }
    }
}

impl PromptSegment for PathSegment {
    fn get_base_width(&self, shrink: ShrinkPriority) -> usize {
        match shrink {
            ShrinkPriority::Unconstrained => self.preferred_width,
            ShrinkPriority::ShrinkComfortable => MIN_PATH_SIZE,
            ShrinkPriority::ShrinkBeyondMin => 1,
        }
    }

    fn get_actual_width_when_under(&self, max_size: usize) -> usize {
        if max_size >= self.preferred_width {
            self.preferred_width
        } else if max_size >= MIN_PATH_SIZE {
            max_size
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
            if self.path_segments.is_empty() {
                format!(" {} ", prefix_char)
            } else {
                let full_text = self.path_segments.join(separator.as_str());
                format!(" {}{}{} ", prefix_char, separator, full_text)
            }
        } else if max_size >= MIN_PATH_SIZE {
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
    use std::{borrow::Cow, path::PathBuf};

    use crate::{
        path::{get_relative_path, PathType, MIN_PATH_SIZE, PATH_SEPARATOR},
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
    fn preferred_width_path() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/home/me/abc/de");
        let (path_type, path_buf) = get_relative_path(cwd, home);
        let segment = PathSegment::new_from_path(path_type, Cow::Owned(path_buf));
        assert_eq!(segment.preferred_width, " ~ > abc > de ".len());
    }

    #[test]
    fn preferred_width_home() {
        let home = PathBuf::from("/home/me");
        let cwd = PathBuf::from("/home/me/");
        let (path_type, path_buf) = get_relative_path(cwd, home);
        let segment = PathSegment::new_from_path(path_type, Cow::Owned(path_buf));
        assert_eq!(segment.preferred_width, " ~ ".len());
    }

    #[test]
    fn render_home() {
        let segment =
            PathSegment::new_from_path(PathType::RelativeToHome, Cow::Owned(PathBuf::new()));
        let rendered = segment.render_at_size(segment.preferred_width);
        assert_eq!(rendered.text, " ~ ");
    }

    #[test]
    fn render_single() {
        let segment = PathSegment::new_from_path(
            PathType::RelativeToHome,
            Cow::Owned(PathBuf::from("1234567890")),
        );
        let full_size = segment.render_at_size(segment.preferred_width);
        assert_eq!(full_size.text, format!(" ~ {} 1234567890 ", PATH_SEPARATOR));
        let constrained = segment.render_at_size(10);
        assert_eq!(constrained.text, " ...67890 ");
    }

    #[test]
    fn render_multiple() {
        let segment = PathSegment::new_from_path(
            PathType::RelativeToHome,
            Cow::Owned(PathBuf::from("1234567890/1234")),
        );
        let full_size = segment.render_at_size(segment.preferred_width);
        assert_eq!(
            full_size.text,
            format!(" ~ {0} 1234567890 {0} 1234 ", PATH_SEPARATOR)
        );
        let constrained = segment.render_at_size(16);
        assert_eq!(
            constrained.text,
            format!(" ...7890 {} 1234 ", PATH_SEPARATOR)
        );
    }

    #[test]
    fn render_absolute() {
        let segment = PathSegment::new_from_path(
            PathType::RelativeToRoot,
            Cow::Owned(PathBuf::from("1234567890/1234")),
        );
        let full_size = segment.render_at_size(segment.preferred_width);
        assert_eq!(
            full_size.text,
            format!(" / {0} 1234567890 {0} 1234 ", PATH_SEPARATOR)
        );
    }

    #[test]
    fn render_smallest() {
        let segment = PathSegment::new_from_path(
            PathType::RelativeToRoot,
            Cow::Owned(PathBuf::from("1234567890/1234")),
        );
        let allowed = segment.render_at_size(MIN_PATH_SIZE);
        assert_eq!(allowed.text, " ...4 ");
        let smallest = segment.render_at_size(MIN_PATH_SIZE - 1);
        assert_eq!(smallest.text, " ");
    }
}
