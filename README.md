# Site

[![build](https://github.com/hayatoito/site/workflows/build/badge.svg)](https://github.com/hayatoito/site/actions)
[![crates.io](https://img.shields.io/crates/v/site.svg)](https://crates.io/crates/site)

**Site** is a fast, simple, and opinionated static site generator written in
[Rust](https://www.rust-lang.org/). Its main features are:

- **Markdown-based**: Write your content in Markdown with extensions, powered by
  [pulldown_cmark](https://crates.io/crates/pulldown-cmark).
- **Jinja2 Templates**: Uses the [minijinja](https://crates.io/crates/minijinja)
  engine for templating.
- **Fast & Simple**: Blazingly fast, with parallel processing of articles.
- **Inspired by Pelican**: Borrows the concepts of _Articles_ and _Pages_ from
  the Python static site generator
  [Pelican](http://docs.getpelican.com/en/stable/).
- **Opinionated by Design**: Implements only what the author needs for a simple
  static site like [hayatoito.github.io](https://hayatoito.github.io/). For
  custom features, forking is recommended. The codebase is intentionally small,
  making it easy to understand and modify.

# Installation

```shell
cargo install site
```

# Usage

There is no documentation yet.

In the meantime, you can use the
[hayatoito/hayatoito.github.io](https://github.com/hayatoito/hayatoito.github.io)
repository as a starter template. The author's site,
[hayatoito.github.io](https://hayatoito.github.io/), is built from this
repository.

# Folder Structure

```text
root_dir/
 - src/
   - (Your markdown files go here)
 - template/
   - (Your template files go here)
```

- `src/`: This directory contains all your Markdown files. They are converted to
  HTML using Jinja2 templates and placed in the output directory. Any other
  files in this directory are also copied to the output directory.

- `template/`: This directory holds your Jinja2 template files.

# Markdown Format

`Site` uses Markdown with TOML front matter for metadata.

```markdown
# Article title

<!--
date = "2021-12-01"
-->

# Section

Hello Article!

- hello
- world
```

# Metadata

| Name          | Description                                       | Default Value                |
| ------------- | ------------------------------------------------- | ---------------------------- |
| `page`        | If `true`, the file is treated as a page.         | `false`                      |
| `date`        | The publication date of the article.              | (mandatory for articles)     |
| `update_date` | The date the article was last updated.            | (none)                       |
| `author`      | The author of the article.                        | (none)                       |
| `slug`        | The URL slug for the page.                        | (derived from the file path) |
| `math`        | If `true`, enables MathJax for the page.          | `false`                      |
| `draft`       | If `true`, the article will not be published.     | `false`                      |
| `template`    | The template file to use from the `template` dir. | `article` or `page`          |

# Pages

If a Markdown file's metadata contains `page = true`, **Site** treats it as a
_page_ instead of an _article_.

```markdown
# Page title

<!--
page = true
-->

# Section

Hello Page!

- hello
- world
```

The main differences between an article and a page are:

- A page is not included in the `articles` or `articles_by_year` template
  variables.
- A page does not require a `date` in its metadata.

# Template Variables

| Name               | Available On     | Description                                       |
| ------------------ | ---------------- | ------------------------------------------------- |
| `entry`            | Pages & Articles | The current article or page object.               |
| `site`             | Pages & Articles | Site configuration from the `--config` parameter. |
| `articles`         | Pages only       | A list of all articles.                           |
| `articles_by_year` | Pages only       | A list of articles grouped by year.               |

An article template does not have access to other articles.

## `entry`

The `entry` object contains all the metadata fields, plus the following:

| Name            | Description             |
| --------------- | ----------------------- |
| `entry.title`   | The title.              |
| `entry.content` | The rendered HTML body. |

# Generate Your Site

## From the CLI

```shell
site build --root . --out out --config=config.toml
```

The `root` directory must contain `src` and `template` directories. For more
examples, see the
[Make.zsh](https://github.com/hayatoito/hayatoito.github.io/blob/main/Make.zsh)
file in the starter template.

## With GitHub Actions

You can use GitHub Actions to automatically build and deploy your site to GitHub
Pages. See the example
[build.yml](https://github.com/hayatoito/hayatoito.github.io/blob/main/.github/workflows/build.yml)
workflow file.
