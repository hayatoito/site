use lazy_static::*;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;

// Convert the given string to a valid HTML element ID
fn normalize_id(content: &str) -> String {
    let ret = content
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>();

    lazy_static! {
        static ref SPACES: Regex = Regex::new(r" +").unwrap();
    }

    let ret = SPACES.replace_all(ret.trim(), "-").to_string();
    if ret.is_empty() {
        "a".to_string()
    } else {
        ret
    }
}

fn id_from_content(content: &str) -> String {
    let mut content = content.to_string();

    // Skip html-encoded stuff
    const REPL_SUB: &[&str] = &["&lt;", "&gt;", "&amp;", "&#39;", "&quot;"];
    for sub in REPL_SUB {
        content = content.replace(sub, " ");
    }

    // Remove tag. e.g. <a href=xxx>hello</a>  => hello
    lazy_static! {
        static ref TAG: Regex = Regex::new(r"</?\w+.*?>").unwrap();
    }
    let content = TAG.replace_all(&content, " ");
    normalize_id(content.as_ref())
}

pub fn build_header_links(html: &str) -> String {
    let regex = Regex::new(r"<h(\d)>(.*?)</h\d>").unwrap();
    let mut id_counter = HashMap::new();

    regex
        .replace_all(html, |caps: &regex::Captures<'_>| {
            let level = caps[1]
                .parse()
                .expect("Regex should ensure we only ever get numbers here");

            wrap_header_with_link(level, &caps[2], &mut id_counter)
        })
        .into_owned()
}

fn wrap_header_with_link(
    level: usize,
    content: &str,
    id_counter: &mut HashMap<String, usize>,
) -> String {
    lazy_static! {
        static ref ANCHOR_REGEX: Regex = Regex::new(r#"<a name="(?P<id>.*?)"></a>"#).unwrap();
    }

    let (raw_id, text) = if let Some(caps) = ANCHOR_REGEX.captures(content) {
        (caps["id"].to_string(), ANCHOR_REGEX.replace(content, ""))
    } else {
        (
            id_from_content(content),
            std::borrow::Cow::Borrowed(content),
        )
    };

    let id_count = id_counter.entry(raw_id.to_owned()).or_insert(0);

    let id = match *id_count {
        0 => raw_id,
        other => format!("{raw_id}-{other}"),
    };

    *id_count += 1;

    format!(r##"<h{level} id="{id}"><a class="self-link" href="#{id}">{text}</a></h{level}>"##,)
}

pub fn build_toc(html: &str, toc_level: Option<u8>) -> String {
    let header_level: String = match toc_level {
        Some(level) if level <= 9 => (1..=level).map(|i| i.to_string()).collect(),
        Some(level) => {
            log::warn!("Invalid toc_level is found: {}. Using 1-9...", level);
            "1-9".to_string()
        }
        None => "1-9".to_string(),
    };
    let regex = Regex::new(&format!(
        r#"<h(?P<level>[{header_level}]) id="(?P<id>.*?)">(<a.*?>)?(?P<text>.*?)(</a>)?</h\d>"#,
    ))
    .unwrap();
    let mut toc = String::new();
    let list_start = r#"<ul>
"#;
    let list_end = r#"</ul>
"#;

    let mut prev_level = 0;
    for cap in regex.captures_iter(html) {
        let level: usize = cap["level"].parse().unwrap();
        let anchor = format!(
            r##"<li><a href="#{id}">{text}</a></li>
"##,
            id = &cap["id"],
            text = &cap["text"]
        );
        match prev_level.cmp(&level) {
            Ordering::Less => {
                for _ in 0..(level - prev_level) {
                    toc.push_str(list_start);
                }
            }
            Ordering::Greater => {
                for _ in 0..(prev_level - level) {
                    toc.push_str(list_end);
                }
            }
            Ordering::Equal => {
                // do nothing
            }
        }
        toc.push_str(&anchor);
        prev_level = level;
    }
    for _ in 0..prev_level {
        toc.push_str(list_end);
    }
    toc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_from_content_test() {
        assert_eq!(id_from_content("abc"), "abc");
        assert_eq!(id_from_content("  abc  "), "abc");
        assert_eq!(id_from_content("abc def"), "abc-def");
        assert_eq!(id_from_content("あいう abc えお def"), "abc-def");
        assert_eq!(id_from_content("a<a href=xxx>hello</a>b"), "a-hello-b");
    }

    #[test]
    fn build_toc_test() {
        assert_eq!(
            build_toc(r#"<h1 id="hello1">hello</h1>"#, None),
            r##"<ul>
<li><a href="#hello1">hello</a></li>
</ul>
"##
        );

        assert_eq!(
            build_toc(
                r#"<h1 id="hello1">Hello 1</h1>
<h1 id="hello2">Hello 2</h1>
"#,
                None
            ),
            r##"<ul>
<li><a href="#hello1">Hello 1</a></li>
<li><a href="#hello2">Hello 2</a></li>
</ul>
"##
        );

        assert_eq!(
            build_toc(
                r#"<h1 id="hello1">Hello 1</h1>
<h1 id="hello2">Hello 2</h1>
<h2 id="hello3">Hello 3</h2>
"#,
                None
            ),
            r##"<ul>
<li><a href="#hello1">Hello 1</a></li>
<li><a href="#hello2">Hello 2</a></li>
<ul>
<li><a href="#hello3">Hello 3</a></li>
</ul>
</ul>
"##
        );

        assert_eq!(
            build_toc(
                r#"<h1 id="hello1">Hello 1</h1>
<h1 id="hello2">Hello 2</h1>
<h2 id="hello3">Hello 3</h2>
"#,
                Some(1)
            ),
            r##"<ul>
<li><a href="#hello1">Hello 1</a></li>
<li><a href="#hello2">Hello 2</a></li>
</ul>
"##
        );
    }
}
