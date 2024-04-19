use crate::Shell;

fn get_exe_path() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    let canonical = std::fs::canonicalize(path).ok()?;
    let string = canonical.into_os_string().into_string().ok()?;
    Some(string)
}

fn init_script_zsh(exe_path: String) -> String {
    format!(
        "\
            unsetopt promptsubst\n\
            precmd() {{ PS1=$({} --columns=\"$COLUMNS\" --status=\"$pipestatus\" --jobs=\"$(jobs -l | wc -l)\") }}\n\
        ",
        exe_path
    )
}

fn init_script_fish(exe_path: String) -> String {
    format!(
        "\
            function fish_prompt\n\
                {} --columns=\"$COLUMNS\" --status=\"$pipestatus\" --jobs=(jobs | wc -l)\n\
            end\n\
        ",
        exe_path
    )
}

fn init_script_bash(exe_path: String) -> String {
    format!(
        "PROMPT_COMMAND=\"PS1=\\$({} --columns=\\\"$COLUMNS\\\" --status=\\\"${{pipestatus:-0}}\\\" --jobs=${{jobs -l | wc -l}})\"",
        exe_path
    )
}

pub fn echo_init_script(shell: Shell) {
    let path = get_exe_path();
    let string = match path {
        None => String::from("echo 'Error getting executable path for prompt'; (exit 1)"),
        Some(path) => match shell {
            Shell::Zsh => init_script_zsh(path),
            Shell::Fish => init_script_fish(path),
            Shell::Bash => init_script_bash(path),
        },
    };
    println!("{}", string);
}
