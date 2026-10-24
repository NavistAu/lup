//! Hand-authored shell completion scripts, embedded at compile time.

const BASH: &str = include_str!("bash.txt");
const ZSH: &str = include_str!("zsh.txt");
const FISH: &str = include_str!("fish.txt");

pub fn completion_text(shell: &[u8]) -> Option<&'static str> {
    match shell {
        b"bash" => Some(BASH),
        b"zsh" => Some(ZSH),
        b"fish" => Some(FISH),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shells_return_text() {
        assert!(completion_text(b"bash").is_some());
        assert!(completion_text(b"zsh").is_some());
        assert!(completion_text(b"fish").is_some());
    }

    #[test]
    fn unknown_shell_returns_none() {
        assert!(completion_text(b"powershell").is_none());
    }

    #[test]
    fn bash_completion_invokes_complete_builtin() {
        assert!(BASH.contains("complete -F _lup lup"));
    }
}
