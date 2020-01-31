/// For pretieer: wrapping: "proseWrap": "always"
/// e.g. "あいう\nえお" -> "あいうえお"
/// See the test.
pub fn remove_newline_between_cjk(s: &str) -> String {
    use unicode_width::UnicodeWidthChar;

    enum State {
        Char,
        WideChar,
        WideCharNewlineSpaces,
    }

    enum CharacterType {
        Newline,
        Space,
        WideChar,
        Char,
    }

    impl CharacterType {
        fn from(c: char) -> CharacterType {
            match c {
                '\n' => CharacterType::Newline,
                ' ' => CharacterType::Space,
                _ => match c.width() {
                    Some(w) if w >= 2 => CharacterType::WideChar,
                    _ => CharacterType::Char,
                },
            }
        }
    }

    let mut out = String::new();
    let mut buffer = String::new();

    let mut state = State::Char;
    for c in s.chars() {
        let ctype = CharacterType::from(c);
        match state {
            State::Char => match ctype {
                CharacterType::Newline | CharacterType::Space | CharacterType::Char => {
                    out.push(c);
                    state = State::Char;
                }
                CharacterType::WideChar => {
                    out.push(c);
                    state = State::WideChar;
                }
            },
            State::WideChar => match ctype {
                CharacterType::Newline => {
                    buffer.push(c);
                    state = State::WideCharNewlineSpaces;
                }
                CharacterType::Space | CharacterType::Char => {
                    out.push(c);
                    state = State::Char;
                }
                CharacterType::WideChar => {
                    out.push(c);
                    state = State::WideChar;
                }
            },
            State::WideCharNewlineSpaces => match ctype {
                CharacterType::Newline | CharacterType::Char => {
                    out.push_str(&buffer);
                    out.push(c);
                    buffer.clear();
                    state = State::Char;
                }
                CharacterType::Space => {
                    buffer.push(c);
                    state = State::WideCharNewlineSpaces;
                }
                CharacterType::WideChar => {
                    // Ignore buffer
                    buffer.clear();
                    out.push(c);
                    state = State::WideChar;
                }
            },
        }
    }
    out
}

pub fn remove_prettier_ignore_preceeding_code_block(s: &str) -> String {
    s.replace("\n<!-- prettier-ignore -->\n```", "\n```")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_prettier_ignore_preceeding_code_block_test() {
        let s = r"foo
<!-- prettier-ignore -->
```html";
        assert_eq!(
            remove_prettier_ignore_preceeding_code_block(s),
            "foo\n```html"
        );

        let s = r"foo

<!-- prettier-ignore -->
```html";
        assert_eq!(
            remove_prettier_ignore_preceeding_code_block(s),
            "foo\n\n```html"
        );
    }

    #[test]
    fn remove_newline_between_cjk_test() {
        let s = r"abc
de";
        assert_eq!(remove_newline_between_cjk(s), "abc\nde");

        let s = r"ä
ä";
        assert_eq!(remove_newline_between_cjk(s), "ä\nä");

        let s = r"あいう
えお";
        assert_eq!(remove_newline_between_cjk(s), "あいうえお");

        let s = r"あいう
ab";
        assert_eq!(remove_newline_between_cjk(s), "あいう\nab");

        let s = r"あいう
ä";
        assert_eq!(remove_newline_between_cjk(s), "あいう\nä");

        // For itemized list. Remove newline + spaces
        let s = r"- あいう
  えお";
        assert_eq!(remove_newline_between_cjk(s), "- あいうえお");

        let s = r"- あいう
  ab";
        assert_eq!(remove_newline_between_cjk(s), "- あいう\n  ab");

        // Don't remove. newline + newline
        let s = r"あいう

えお";
        assert_eq!(remove_newline_between_cjk(s), "あいう\n\nえお");
    }
}
