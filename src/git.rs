use std::cmp::min;

use crate::{colors, segments::*};
use git2::Repository;
use unicode_segmentation::UnicodeSegmentation;

struct FileChanges {
    staged: bool,
    unstaged: bool,
    conflicted: bool,
}

enum GitStatus {
    Clean,
    UntrackedFiles,
    Changes(FileChanges),
}

pub struct GitSegment {
    status: GitStatus,
    status_str_len: usize,
    branch_name: String,
    branch_name_len: usize,
}

const MIN_BRANCH_TEXT: usize = 4;

fn get_branch_name(repo: &Repository) -> Option<String> {
    if repo.head_detached().ok().unwrap_or(false) {
        let rev = repo.revparse_single("HEAD").ok()?;
        return String::from_utf8(rev.short_id().ok()?.to_vec()).ok();
    };

    let head = repo.head().ok()?;
    head.shorthand().map(|x| x.to_string())
}

impl GitSegment {
    pub fn new(context: &Context) -> Option<Self> {
        let repo = Repository::open(context.path.as_ref()?).ok()?;
        let statuses = repo.statuses(None).ok()?;
        let status = get_repo_status(&statuses);
        let status_str_len = calculate_status_size_len(&status);
        let branch_name = get_branch_name(&repo).unwrap_or(String::from("<NO HEAD>"));
        let branch_name_len = branch_name.graphemes(true).count();
        // repo.state() // TODO handle state
        Some(GitSegment {
            status,
            status_str_len,
            branch_name,
            branch_name_len,
        })
    }

    fn get_unconstrained_total_len(&self) -> usize {
        let mut size = self.branch_name_len + 4;
        if self.status_str_len != 0 {
            size += self.status_str_len + 1;
        }
        size
    }
    fn get_min_len_with_branch_name(&self) -> usize {
        let mut size = min(self.branch_name_len, MIN_BRANCH_TEXT + 3) + 4;
        if self.status_str_len != 0 {
            size += self.status_str_len + 1;
        }
        size
    }

    fn render_status_symbols(&self, string_builder: &mut String) {
        if let GitStatus::Changes(FileChanges {
            staged,
            unstaged,
            conflicted,
        }) = self.status
        {
            if staged || unstaged || conflicted {
                string_builder.push(' ');
            }
            if staged {
                string_builder.push('+');
            }
            if unstaged {
                string_builder.push('*');
            }
            if conflicted {
                string_builder.push('!');
            }
        }
    }
}

fn calculate_status_size_len(status: &GitStatus) -> usize {
    match status {
        GitStatus::Clean => 0,
        GitStatus::UntrackedFiles => 0,
        GitStatus::Changes(FileChanges {
            staged,
            unstaged,
            conflicted,
        }) => *staged as usize + *unstaged as usize + *conflicted as usize,
    }
}

fn get_repo_status(statuses: &git2::Statuses) -> GitStatus {
    let mut unstaged_changes = false;
    let mut staged_changes = false;
    let mut conflicted = false;
    let mut untracked = false;
    for e in statuses.iter() {
        let status = e.status();
        if status.is_empty() {
            continue;
        }
        if status.is_wt_new() {
            untracked = true;
            continue;
        }

        if status.is_wt_modified()
            || status.is_wt_deleted()
            || status.is_wt_typechange()
            || status.is_wt_renamed()
        {
            unstaged_changes = true;
            continue;
        }

        if status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_typechange()
            || status.is_index_renamed()
        {
            staged_changes = true;
            continue;
        }

        if status.is_conflicted() {
            conflicted = true;
            continue;
        }
    }

    if staged_changes || unstaged_changes || conflicted {
        GitStatus::Changes(FileChanges {
            staged: staged_changes,
            unstaged: unstaged_changes,
            conflicted,
        })
    } else if untracked {
        GitStatus::UntrackedFiles
    } else {
        GitStatus::Clean
    }
}

impl PromptSegment for GitSegment {
    fn get_base_width(&self, shrink: ShrinkPriority) -> usize {
        match shrink {
            ShrinkPriority::Unconstrained => self.get_unconstrained_total_len(),
            ShrinkPriority::ShrinkConfortable => self.get_min_len_with_branch_name(),
            ShrinkPriority::ShrinkBeyondMin => 0,
        }
    }

    fn get_actual_width_when_under(&self, max_size: usize) -> usize {
        if max_size >= self.get_min_len_with_branch_name() {
            min(max_size, self.get_unconstrained_total_len())
        } else if max_size >= self.get_min_len_with_branch_name() {
            max_size
        } else if self.status_str_len != 0 && max_size >= self.status_str_len + 4 {
            self.status_str_len + 4
        } else if max_size >= 3 {
            3
        } else {
            0
        }
    }

    fn render_at_size(&self, max_size: usize) -> RenderedSegment {
        let text = if max_size >= self.get_unconstrained_total_len() {
            // unconstrained
            let mut string_builder = String::from(" \u{e0a0} ");
            string_builder.push_str(self.branch_name.as_str());
            self.render_status_symbols(&mut string_builder);
            string_builder.push(' ');
            string_builder
        } else if max_size >= self.get_min_len_with_branch_name() {
            // elipsize branch name
            let mut string_builder = String::from(" \u{e0a0} ");
            self.branch_name
                .graphemes(true)
                .take(max_size.saturating_sub(
                    3 + 3
                        + 1
                        + if self.status_str_len == 0 {
                            0
                        } else {
                            self.status_str_len + 1
                        },
                ))
                .for_each(|x| string_builder.push_str(x));
            string_builder.push_str("...");
            self.render_status_symbols(&mut string_builder);
            string_builder.push(' ');
            string_builder
        } else if max_size >= self.status_str_len + 4 {
            // just git symbol and status symbols
            let mut string_builder = String::from(" \u{e0a0}");
            self.render_status_symbols(&mut string_builder);
            string_builder.push(' ');
            string_builder
        } else if max_size >= 3 {
            // just git symbol
            String::from(" \u{e0a0} ")
        } else {
            String::new()
        };

        debug_assert_eq!(
            text.graphemes(true).count(),
            self.get_actual_width_when_under(max_size)
        );
        RenderedSegment {
            text,
            bg_color: match self.status {
                GitStatus::Clean => colors::GREEN,
                _ => colors::YELLOW,
            },
            fg_color: colors::BLACK,
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::segments::{PromptSegment, ShrinkPriority};

    use super::{calculate_status_size_len, FileChanges, GitSegment, GitStatus};

    #[test]
    fn format_with_status() {
        let status = GitStatus::Changes(FileChanges {
            staged: true,
            unstaged: true,
            conflicted: false,
        });
        let status_str_len = calculate_status_size_len(&status);
        let segment = GitSegment {
            status,
            status_str_len,
            branch_name: "example123".to_string(),
            branch_name_len: 10,
        };
        assert_eq!(segment.get_base_width(ShrinkPriority::Unconstrained), 17);
        assert_eq!(
            segment.get_base_width(ShrinkPriority::ShrinkConfortable),
            14
        );
        assert_eq!(segment.get_base_width(ShrinkPriority::ShrinkBeyondMin), 0);

        assert_eq!(segment.render_at_size(40).text, " \u{e0a0} example123 +* ");
        assert_eq!(segment.render_at_size(14).text, " \u{e0a0} exam... +* ");
        assert_eq!(segment.render_at_size(13).text, " \u{e0a0} +* ");
        assert_eq!(segment.render_at_size(5).text, " \u{e0a0} ");
        assert_eq!(segment.render_at_size(2).text, "");
    }

    #[test]
    fn format_no_status() {
        let status = GitStatus::Changes(FileChanges {
            staged: false,
            unstaged: false,
            conflicted: false,
        });
        let status_str_len = calculate_status_size_len(&status);
        let segment = GitSegment {
            status,
            status_str_len,
            branch_name: "example123".to_string(),
            branch_name_len: 10,
        };
        assert_eq!(segment.get_base_width(ShrinkPriority::Unconstrained), 14);
        assert_eq!(
            segment.get_base_width(ShrinkPriority::ShrinkConfortable),
            11
        );
        assert_eq!(segment.get_base_width(ShrinkPriority::ShrinkBeyondMin), 0);

        assert_eq!(segment.render_at_size(40).text, " \u{e0a0} example123 ");
        assert_eq!(segment.render_at_size(13).text, " \u{e0a0} exampl... ");
        assert_eq!(segment.render_at_size(10).text, " \u{e0a0} ");
        assert_eq!(segment.render_at_size(2).text, "");
    }
}