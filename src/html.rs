use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

fn id_from_title(title: &str) -> String {
    let mut title = title.to_string();

    // Skip html-encoded stuff
    const REPL_SUB: &[&str] = &["&lt;", "&gt;", "&amp;", "&#39;", "&quot;"];
    for sub in REPL_SUB {
        title = title.replace(sub, " ");
    }

    // Convert the given string to a valid HTML element ID
    let ret = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>();

    static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" +").unwrap());

    let ret = SPACES.replace_all(ret.trim(), "-").to_string();
    if ret.is_empty() { "a".to_string() } else { ret }
}

pub fn build_header_links(html: &str) -> String {
    let header = Regex::new(r#"<h(?P<level>\d)( id="(?P<id>.*?)")?>(?P<title>.*?)</h\d>"#).unwrap();
    let mut id_counter = HashMap::new();

    header
        .replace_all(html, |caps: &regex::Captures<'_>| {
            let level = caps
                .name("level")
                .unwrap()
                .as_str()
                .parse()
                .expect("Regex should ensure we only ever get numbers here");
            let title = caps.name("title").unwrap().as_str();
            let id = caps.name("id").map(|id| id.as_str());

            wrap_header_with_link(level, title, id, &mut id_counter)
        })
        .into_owned()
}

fn wrap_header_with_link(
    level: usize,
    title: &str,
    id: Option<&str>,
    id_counter: &mut HashMap<String, usize>,
) -> String {
    if let Some(id) = id {
        format!(r##"<h{level} id="{id}"><a class="self-link" href="#{id}">{title}</a></h{level}>"##,)
    } else {
        let id = id_from_title(title);
        let id_count = id_counter.entry(id.to_owned()).or_insert(0);

        let id = if *id_count == 0 {
            id
        } else {
            format!("{id}-{}", *id_count)
        };
        *id_count += 1;
        format!(r##"<h{level} id="{id}"><a class="self-link" href="#{id}">{title}</a></h{level}>"##,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_headert() {
        let header =
            Regex::new(r#"<h(?P<level>\d)( id="(?P<id>.*?)")?>(?P<title>.*?)</h\d>"#).unwrap();

        let cap = header.captures(r#"<h1 id="id1">title1</h1>"#).unwrap();
        assert_eq!(cap.name("level").unwrap().as_str(), "1");
        assert_eq!(cap.name("id").unwrap().as_str(), "id1");
        assert_eq!(cap.name("title").unwrap().as_str(), "title1");

        let cap = header.captures(r#"<h1>title1</h1>"#).unwrap();
        assert_eq!(cap.name("level").unwrap().as_str(), "1");
        assert!(cap.name("id").is_none());
        assert_eq!(cap.name("title").unwrap().as_str(), "title1");
    }

    #[test]
    fn build_header_links_test() {
        let html = r#"
<h1 id="id1">title1</h1>
<h2>title2</h2>
<h3>title2</h3>
"#;
        let replaced = build_header_links(html);
        assert_eq!(
            replaced,
            r##"
<h1 id="id1"><a class="self-link" href="#id1">title1</a></h1>
<h2 id="title2"><a class="self-link" href="#title2">title2</a></h2>
<h3 id="title2-1"><a class="self-link" href="#title2-1">title2</a></h3>
"##
        );
    }

    #[test]
    fn id_from_content_test() {
        assert_eq!(id_from_title("abc"), "abc");
        assert_eq!(id_from_title("  abc  "), "abc");
        assert_eq!(id_from_title("abc def"), "abc-def");
        assert_eq!(id_from_title("あいう abc えお def"), "abc-def");
    }
}
