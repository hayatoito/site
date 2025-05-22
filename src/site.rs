use anyhow::Context as _;
pub use anyhow::Result;
use anyhow::{Error, anyhow};
use chrono::Datelike;
use minijinja::{Environment, Value, context, path_loader};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;

use crate::html;
use crate::text;
use orgize;

#[derive(Debug)]
enum SourceFile {
    Markdown(MarkdownFile),
    Org(OrgFile),
}

#[derive(PartialEq, Eq, Debug, Deserialize, Default, Clone)]
struct Metadata {
    page: Option<bool>,
    title: String,
    author: Option<String>,
    date: Option<chrono::NaiveDate>,
    update_date: Option<chrono::NaiveDate>,
    slug: Option<String>,
    math: Option<bool>,
    draft: Option<bool>,
    template: Option<String>,
}

impl FromStr for Metadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

#[derive(Debug)]
struct MarkdownFile {
    relative_path: PathBuf,
    markdown: Markdown,
}

#[derive(PartialEq, Eq, Debug)]
struct Markdown {
    metadata: Metadata,
    content: String,
}

impl Markdown {
    pub fn render(&self) -> String {
        let mut opts = pulldown_cmark::Options::empty();
        opts.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        opts.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
        opts.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        let mut html = String::with_capacity(self.content.len() * 3 / 2);
        let content = self.pre_process_content();
        let p = pulldown_cmark::Parser::new_ext(&content, opts);
        pulldown_cmark::html::push_html(&mut html, p);
        Self::post_process_markdown_html(&html)
    }

    fn pre_process_content(&self) -> String {
        let s = text::remove_newline_between_cjk(&self.content);
        let s = text::remove_prettier_ignore_preceeding_code_block(&s);
        text::remove_deno_fmt_ignore(&s)
    }

    fn post_process_markdown_html(html: &str) -> String {
        let html = html::build_header_links(html);
        html.to_string()
    }
}

#[derive(Debug)]
struct OrgFile {
    relative_path: PathBuf,
    org: Org,
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct Org {
    metadata: Metadata,
    content: String,
}

impl Org {
    pub fn render(&self) -> String {
        let s = text::remove_newline_between_cjk(&self.content);
        let s = text::remove_deno_fmt_ignore(&s);
        let html = orgize::Org::parse(&s).to_html();
        html::build_header_links(&html).to_string()
    }
}

impl FromStr for Org {
    type Err = Error;

    fn from_str(s: &str) -> Result<Org> {
        let mut metadata = Metadata::default();
        let mut content_lines = Vec::new();
        let mut in_metadata = true;

        for line in s.lines() {
            if in_metadata && line.starts_with("#+") {
                let parts: Vec<&str> = line[2..].splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_uppercase();
                    let value = parts[1].trim();
                    match key.as_str() {
                        "TITLE" => metadata.title = value.to_string(),
                        "AUTHOR" => metadata.author = Some(value.to_string()),
                        "DATE" => metadata.date = value.parse().ok(),
                        "SLUG" => metadata.slug = Some(value.to_string()),
                        "DRAFT" => metadata.draft = Some(value.parse().unwrap_or(false)),
                        "TEMPLATE" => metadata.template = Some(value.to_string()),
                        "PAGE" => metadata.page = Some(value.parse().unwrap_or(false)),
                        "MATH" => metadata.math = Some(value.parse().unwrap_or(false)),
                        // Add other common Org keywords if needed
                        _ => {} // Unknown keywords are ignored
                    }
                }
            } else {
                in_metadata = false;
                content_lines.push(line);
            }
        }

        Ok(Org {
            metadata,
            content: content_lines.join("\n"),
        })
    }
}

impl FromStr for Markdown {
    type Err = Error;

    fn from_str(s: &str) -> Result<Markdown> {
        // Skip the comment at the beginning. Emacs may use the first line for buffer-local variables.
        // e.g. <!-- -*- apheleia-formatters: prettier -*- -->
        static COMMENT_LINES: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^<!--.*-->\n+").unwrap());

        static TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^# +(.+?) *\n+").unwrap());

        let s = COMMENT_LINES.replace_all(s, "");

        let (metadata_yaml, content) = match TITLE.captures(&s) {
            Some(cap) => {
                // If the first line starts with "#", treat it as a title.
                let title = cap[1].to_string();
                let s = TITLE.replace(&s, "").to_string();

                let mut split = s.splitn(2, "\n\n");

                // Add "title: xxx" to metadata
                let metadata_yaml = split.next().ok_or_else(|| anyhow!("split error"))?;
                // TODO: Espace double quote?
                let metadata_yaml = format!("title = \"{title}\"\n{metadata_yaml}");

                let content = split.next().unwrap_or("");

                (metadata_yaml, content.to_string())
            }
            _ => {
                let mut split = s.splitn(2, "\n\n");
                let metadata_yaml = split.next().ok_or_else(|| anyhow!("split error"))?;
                let content = split.next().unwrap_or("");
                (metadata_yaml.to_string(), content.to_string())
            }
        };

        // Ignore comments, such as <!-- prettier-ignore -->, in metadata.
        static METADATA_COMMENT: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(<!--.*\n*)|(-->.*\n*)").unwrap());

        let metadata_yaml = METADATA_COMMENT.replace_all(&metadata_yaml, "");
        assert!(!metadata_yaml.contains("-->"));
        assert!(!metadata_yaml.contains("<!--"));

        Ok(Markdown {
            metadata: metadata_yaml
                .parse()
                .with_context(|| format!("can not parse metatada: {metadata_yaml}"))?,
            content,
        })
    }
}

fn slug_to_url(slug: &str) -> String {
    if slug.is_empty() || slug == "index" {
        "".to_string()
    } else if slug.ends_with('/') {
        slug.to_string()
    } else if Path::new(slug).extension().is_none() {
        format!("{slug}/")
    } else {
        slug.to_string()
    }
}

fn url_to_filename(url: &str) -> String {
    if url.is_empty() || url.ends_with('/') {
        format!("{}{}", url, "index.html")
    } else {
        url.to_string()
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Default)]
struct Article {
    title: String,
    slug: String,
    author: Option<String>,
    date: Option<chrono::NaiveDate>,
    update_date: Option<chrono::NaiveDate>,
    draft: bool,
    url: String,
    page: bool,
    math: bool,
    template: Option<String>,
    content: String,
}

impl Article {
    fn new(source_file: SourceFile) -> Article {
        let (relative_path, metadata, content) = match source_file {
            SourceFile::Markdown(MarkdownFile {
                relative_path,
                markdown,
            }) => {
                log::debug!("markdown article: {}", relative_path.display());
                let content = markdown.render();
                (relative_path, markdown.metadata, content)
            }
            SourceFile::Org(OrgFile { relative_path, org }) => {
                log::debug!("org article: {}", relative_path.display());
                let content = org.render();
                (relative_path, org.metadata, content)
            }
        };

        let slug = if let Some(slug) = metadata.slug.as_ref() {
            slug.to_string()
        } else {
            relative_path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        };
        let url = relative_path
            .parent()
            .unwrap()
            .join(slug_to_url(&slug))
            .display()
            .to_string();

        Article {
            title: metadata.title,
            slug,
            author: metadata.author,
            date: metadata.date,
            update_date: metadata.update_date,
            draft: metadata.draft.unwrap_or(false),
            url,
            page: metadata.page.unwrap_or(false),
            math: metadata.math.unwrap_or(false),
            template: metadata.template,
            content,
        }
    }

    fn context(&self, config: &Config, articles: Option<&[Article]>) -> Value {
        #[derive(PartialEq, Eq, Debug, Default, Serialize)]
        struct YearArticles<'a> {
            year: i32,
            articles: Vec<&'a Article>,
        }

        let mut context = config.context();
        if let Some(articles) = articles {
            let mut articles_by_year = BTreeMap::<i32, Vec<&Article>>::new();
            for a in articles {
                articles_by_year
                    .entry(a.date.as_ref().unwrap().year())
                    .or_default()
                    .push(a);
            }
            let mut articles_by_year = articles_by_year
                .into_iter()
                .map(|(year, mut articles)| {
                    articles.sort_by_key(|a| a.date);
                    articles.reverse();
                    YearArticles { year, articles }
                })
                .collect::<Vec<_>>();
            articles_by_year.reverse();

            context = context! {
                articles,
                articles_by_year,
                ..context
            };
        };
        context = context! {
            entry => &self,
            ..context
        };
        context
    }

    fn template_name(&self) -> &str {
        match self.template.as_ref() {
            Some(a) => a,
            None => {
                if self.page {
                    "page"
                } else {
                    "article"
                }
            }
        }
    }

    fn render(
        &self,
        config: &Config,
        articles: Option<&[Article]>,
        env: &Environment,
    ) -> Result<String> {
        let context = self.context(config, articles);
        let template = env.get_template(&format!("{}.jinja", self.template_name()))?;
        template
            .render(&context)
            .map_err(|e| anyhow!("renderer err: {}", e))
    }

    fn render_and_write(
        &self,
        config: &Config,
        articles: Option<&[Article]>,
        env: &Environment,
        out_dir: &Path,
    ) -> Result<()> {
        let html = self.render(config, articles, env)?;
        let mut out_file = PathBuf::from(out_dir);
        out_file.push(url_to_filename(&self.url));
        log::debug!("{:32} => {}", self.url, out_file.display());
        std::fs::create_dir_all(out_file.parent().unwrap())?;
        std::fs::write(&out_file, html)?;
        Ok(())
    }
}

pub struct Config(std::collections::BTreeMap<String, String>);

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Config> {
        let s = std::fs::read_to_string(path.as_ref())?;
        Ok(Config(toml::from_str(&s)?))
    }

    fn context(&self) -> minijinja::Value {
        context! { site => &self.0}
    }

    pub fn extend(&mut self, config: &mut Config) {
        self.0.append(&mut config.0);
    }
}

pub struct Site {
    config: Config,
    root_dir: PathBuf,
    src_dir: PathBuf,
    out_dir: PathBuf,
    article_regex: Option<Regex>,
}

impl Site {
    pub fn new(
        config: Config,
        root_dir: PathBuf,
        out_dir: PathBuf,
        article_regex: Option<Regex>,
    ) -> Site {
        let src_dir = root_dir.join("src");
        Site {
            config,
            root_dir: root_dir.canonicalize().unwrap(),
            src_dir,
            out_dir,
            article_regex,
        }
    }

    pub fn build(&self) -> Result<()> {
        let src_dir = self.root_dir.join("src");
        let template_dir = self.root_dir.join("template");

        let mut env = Environment::new();
        env.set_loader(path_loader(template_dir));
        env.set_auto_escape_callback(|_name| minijinja::AutoEscape::None);
        env.set_keep_trailing_newline(true);

        self.render_source_files(&env, src_dir)?;
        if self.article_regex.is_none() {
            self.copy_files()?;
        }
        Ok(())
    }

    fn collect_source_files(&self, src_dir: impl AsRef<Path>) -> Result<Vec<SourceFile>> {
        let src_path = src_dir.as_ref();
        let md_glob = format!("{}/**/*.md", src_path.display());
        let org_glob = format!("{}/**/*.org", src_path.display());

        let markdown_files = glob::glob(&md_glob)?
            .filter_map(Result::ok)
            .filter(|f| match &self.article_regex {
                Some(regex) => regex.is_match(f.as_os_str().to_str().unwrap()),
                None => true,
            })
            .map(|f| -> Result<SourceFile> {
                let relative_path = f.strip_prefix(src_path).expect("prefix does not match");
                log::debug!("found markdown: {}", relative_path.display());
                Ok(SourceFile::Markdown(MarkdownFile {
                    relative_path: PathBuf::from(relative_path),
                    markdown: std::fs::read_to_string(&f)
                        .with_context(|| format!("can not read markdown: {}", f.display()))?
                        .parse()
                        .with_context(|| format!("can not parse markdown: {}", f.display()))?,
                }))
            });

        let org_files = glob::glob(&org_glob)?
            .filter_map(Result::ok)
            .filter(|f| match &self.article_regex {
                Some(regex) => regex.is_match(f.as_os_str().to_str().unwrap()),
                None => true,
            })
            .map(|f| -> Result<SourceFile> {
                let relative_path = f.strip_prefix(src_path).expect("prefix does not match");
                log::debug!("found org: {}", relative_path.display());
                Ok(SourceFile::Org(OrgFile {
                    relative_path: PathBuf::from(relative_path),
                    org: std::fs::read_to_string(&f)
                        .with_context(|| format!("can not read org: {}", f.display()))?
                        .parse()
                        .with_context(|| format!("can not parse org: {}", f.display()))?,
                }))
            });

        markdown_files.chain(org_files).collect()
    }

    fn render_source_files(&self, env: &Environment, src_dir: impl AsRef<Path>) -> Result<()> {
        let src_dir_path = src_dir.as_ref().canonicalize().unwrap();
        log::info!("Collecting source files: {}", src_dir_path.display());
        let (pages, articles) = self
            .collect_source_files(&src_dir_path)?
            .into_iter()
            .partition::<Vec<SourceFile>, _>(|src| match src {
                SourceFile::Markdown(md) => md.markdown.metadata.page.unwrap_or(false),
                SourceFile::Org(org) => org.org.metadata.page.unwrap_or(false),
            });
        log::info!(
            "Found {} articles and {} pages",
            articles.len(),
            pages.len()
        );

        for article_source in &articles {
            let (path_for_log, metadata_for_log) = match article_source {
                SourceFile::Markdown(md) => (md.relative_path.clone(), md.markdown.metadata.clone()),
                SourceFile::Org(org) => (org.relative_path.clone(), org.org.metadata.clone()),
            };
            anyhow::ensure!(
                metadata_for_log.date.is_some(),
                "{} doesn't have date",
                path_for_log.display()
            )
        }

        log::info!("Build articles");
        let mut articles = articles
            .into_par_iter()
            .map(|src_file| -> Result<Article> {
                let article = Article::new(src_file);
                article.render_and_write(&self.config, None, env, &self.out_dir)?;
                Ok(article)
            })
            .collect::<Vec<Result<Article>>>()
            .into_iter()
            .collect::<Result<Vec<Article>>>()?;

        // Remove draft articles.
        articles.retain(|a| !a.draft);

        articles.sort_by_key(|a| a.date);
        articles.reverse();

        log::info!("Build pages");
        for m in pages {
            let page = Article::new(m);
            page.render_and_write(&self.config, Some(&articles), env, &self.out_dir)?;
        }
        Ok(())
    }

    fn copy_files(&self) -> Result<()> {
        log::info!(
            "Copy files: {} => {}",
            self.src_dir.display(),
            self.out_dir.display()
        );
        for entry in walkdir::WalkDir::new(&self.src_dir) {
            let entry = entry?;
            let src_path = entry.path();
            if let Some(ext_str) = src_path.extension().and_then(|ext| ext.to_str()) {
                if ext_str == "md" || ext_str == "org" {
                    continue;
                }
            }

            let relative_path = src_path.strip_prefix(&self.src_dir).expect("");
            let out_path = self.out_dir.join(relative_path);
            log::debug!("{:32} => {}", relative_path.display(), out_path.display());

            if src_path.is_dir() {
                std::fs::create_dir_all(&out_path).expect("create_dir_all failed")
            } else {
                std::fs::copy(src_path, out_path).expect("copy failed");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_to_url_test() {
        assert_eq!(slug_to_url("foo"), "foo/");
        assert_eq!(slug_to_url("foo/"), "foo/");
        assert_eq!(slug_to_url("feed.xml"), "feed.xml");
        assert_eq!(slug_to_url("feed.xml/"), "feed.xml/");
        assert_eq!(slug_to_url("index"), "");
        assert_eq!(slug_to_url(""), "");
        assert_eq!(slug_to_url("a/b"), "a/b/");
        assert_eq!(slug_to_url("a/b/"), "a/b/");
        assert_eq!(slug_to_url("a/b.html"), "a/b.html");
        assert_eq!(slug_to_url("a/b.html/"), "a/b.html/");
    }

    #[test]
    fn url_to_filename_test() {
        assert_eq!(url_to_filename(""), "index.html");
        assert_eq!(url_to_filename("a"), "a");
        assert_eq!(url_to_filename("a/"), "a/index.html");
        assert_eq!(url_to_filename("a.html"), "a.html");
        assert_eq!(url_to_filename("a.html/"), "a.html/index.html");
        assert_eq!(url_to_filename("a/b"), "a/b");
        assert_eq!(url_to_filename("a/b/"), "a/b/index.html");
        assert_eq!(url_to_filename("a/b.html"), "a/b.html");
        assert_eq!(url_to_filename("a/b.html/"), "a/b.html/index.html");
    }

    #[test]
    fn parse_markdowne_metadata_test() {
        let s = r#"title = "Hello"
slug = "10th-anniversary"
date = "2018-01-11"
"#;
        assert_eq!(
            s.parse::<Metadata>().unwrap(),
            Metadata {
                title: "Hello".to_string(),
                slug: Some("10th-anniversary".to_string()),
                date: Some("2018-01-11".parse().unwrap()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn parse_markdown_test() {
        let s = r#"title = "Hello"
slug = "10th-anniversary"
date = "2018-01-11"

hello world
"#;

        assert_eq!(
            s.parse::<Markdown>().unwrap(),
            Markdown {
                metadata: Metadata {
                    title: "Hello".to_string(),
                    slug: Some("10th-anniversary".to_string()),
                    date: Some("2018-01-11".parse().unwrap()),
                    ..Default::default()
                },
                content: "hello world\n".to_string(),
            }
        );

        let s = r#"<!--
title = "Hello"
-->

hello world
"#;
        assert_eq!(
            s.parse::<Markdown>().unwrap(),
            Markdown {
                metadata: Metadata {
                    title: "Hello".to_string(),
                    ..Default::default()
                },
                content: "hello world\n".to_string(),
            }
        );

        let s = r#"<!-- prettier-ignore -->
title = "Hello"

hello world
"#;
        assert_eq!(
            s.parse::<Markdown>().unwrap(),
            Markdown {
                metadata: Metadata {
                    title: "Hello".to_string(),
                    ..Default::default()
                },
                content: "hello world\n".to_string(),
            }
        );

        // If the first line starts with "#", treat that as a title.
        let s = r#"# title

<!-- prettier-ignore -->
date = "2018-01-11"

hello world
"#;
        assert_eq!(
            s.parse::<Markdown>().unwrap(),
            Markdown {
                metadata: Metadata {
                    title: "title".to_string(),
                    date: Some("2018-01-11".parse().unwrap()),
                    ..Default::default()
                },
                content: "hello world\n".to_string(),
            }
        );

        // If the first line starts with "<!--", Ignore that
        let s = r#"<!-- -*- apheleia-formatters: prettier -*-  -->

# title

<!-- prettier-ignore -->
date = "2018-01-11"

hello world
"#;
        assert_eq!(
            s.parse::<Markdown>().unwrap(),
            Markdown {
                metadata: Metadata {
                    title: "title".to_string(),
                    date: Some("2018-01-11".parse().unwrap()),
                    ..Default::default()
                },
                content: "hello world\n".to_string(),
            }
        );
    }

    #[test]
    fn parse_org_metadata_test() {
        let s = r#"#+TITLE: Org Title
#+AUTHOR: Test Author
#+DATE: 2024-01-01
#+SLUG: org-slug
#+DRAFT: true
#+TEMPLATE: custom_template.jinja
#+PAGE: true
#+MATH: true
"#;
        let org_struct: Org = s.parse().unwrap();
        let metadata = org_struct.metadata;
        assert_eq!(metadata.title, "Org Title");
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.date, Some("2024-01-01".parse().unwrap()));
        assert_eq!(metadata.slug, Some("org-slug".to_string()));
        assert_eq!(metadata.draft, Some(true));
        assert_eq!(metadata.template, Some("custom_template.jinja".to_string()));
        assert_eq!(metadata.page, Some(true));
        assert_eq!(metadata.math, Some(true));

        // Test partial metadata
        let s_partial = r#"#+TITLE: Partial Title
#+DATE: 2023-12-31
"#;
        let org_partial: Org = s_partial.parse().unwrap();
        assert_eq!(org_partial.metadata.title, "Partial Title");
        assert_eq!(org_partial.metadata.date, Some("2023-12-31".parse().unwrap()));
        assert_eq!(org_partial.metadata.author, None);
    }

    #[test]
    fn parse_org_content_test() {
        let s = r#"#+TITLE: Content Test
#+DATE: 2024-01-02

* Heading 1
Some paragraph text.
- list item 1
- list item 2
"#;
        let org_struct: Org = s.parse().unwrap();
        assert_eq!(org_struct.metadata.title, "Content Test");
        assert_eq!(
            org_struct.content,
            "* Heading 1\nSome paragraph text.\n- list item 1\n- list item 2"
        );

        // Test without metadata
        let s_content_only = r#"* Just Content
No metadata here.
"#;
        let org_content_only: Org = s_content_only.parse().unwrap();
        assert_eq!(org_content_only.metadata.title, ""); // Default title
        assert_eq!(org_content_only.content, "* Just Content\nNo metadata here.");

        // Test with empty lines between metadata and content
        let s_empty_lines = r#"#+TITLE: Empty Lines Test


* Content Starts Here
"#;
        let org_empty_lines: Org = s_empty_lines.parse().unwrap();
        assert_eq!(org_empty_lines.metadata.title, "Empty Lines Test");
        assert_eq!(org_empty_lines.content, "\n* Content Starts Here");
    }

    #[test]
    fn render_org_html_test() {
        let org_document = Org {
            metadata: Metadata {
                title: "Render Test".to_string(),
                ..Default::default()
            },
            content: "* Hello Org\nThis is org content with a [[https://example.com][link]].".to_string(),
        };
        let html = org_document.render();
        assert!(html.contains("<h1 id=\"hello-org\">Hello Org</h1>"));
        assert!(html.contains("<p>\nThis is org content with a <a href=\"https://example.com\">link</a>.\n</p>")); // orgize adds newline

        // Test with a list
        let org_list = Org {
            metadata: Metadata::default(),
            content: "- item 1\n- item 2".to_string(),
        };
        let html_list = org_list.render();
        assert!(html_list.contains("<ul>"));
        assert!(html_list.contains("<li>item 1</li>"));
        assert!(html_list.contains("<li>item 2</li>"));
        assert!(html_list.contains("</ul>"));
    }

    #[test]
    fn article_from_org_test() {
        let org_content_str = r#"#+TITLE: Org Article Title
#+AUTHOR: Org Author
#+DATE: 2024-03-15
#+SLUG: my-org-article
#+DRAFT: false
#+PAGE: false
#+MATH: true

* Introduction
This is an article written in Org mode.
"#;
        let org_file = OrgFile {
            relative_path: PathBuf::from("test_articles/my-org-article.org"),
            org: org_content_str.parse().unwrap(),
        };

        let article = Article::new(SourceFile::Org(org_file));

        assert_eq!(article.title, "Org Article Title");
        assert_eq!(article.author, Some("Org Author".to_string()));
        assert_eq!(article.date, Some("2024-03-15".parse().unwrap()));
        assert_eq!(article.slug, "my-org-article");
        assert_eq!(article.draft, false);
        assert_eq!(article.page, false);
        assert_eq!(article.math, true);
        assert_eq!(article.url, "test_articles/my-org-article/");
        assert!(article.content.contains("<h1 id=\"introduction\">Introduction</h1>"));
        assert!(article.content.contains("<p>\nThis is an article written in Org mode.\n</p>"));

        // Test with minimal metadata (relying on slug generation from filename)
         let org_minimal_str = r#"#+TITLE: Minimal Org
#+DATE: 2024-03-16

Minimal content.
"#;
        let org_file_minimal = OrgFile {
            relative_path: PathBuf::from("another/minimal.org"),
            org: org_minimal_str.parse().unwrap(),
        };
        let article_minimal = Article::new(SourceFile::Org(org_file_minimal));
        assert_eq!(article_minimal.title, "Minimal Org");
        assert_eq!(article_minimal.date, Some("2024-03-16".parse().unwrap()));
        assert_eq!(article_minimal.slug, "minimal"); // auto-generated from filename
        assert_eq!(article_minimal.url, "another/minimal/");
        assert!(article_minimal.content.contains("<p>\nMinimal content.\n</p>"));
    }
}
