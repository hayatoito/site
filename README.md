# Site

[![build](https://github.com/hayatoito/site/workflows/build/badge.svg)](https://github.com/hayatoito/site/actions)
[![crates.io](https://img.shields.io/crates/v/site.svg)](https://crates.io/crates/site)

**Site** is a fast, simple, and opinioned static site generator, written in
[Rust](https://www.rust-lang.org/). Highlights include:

- Write your content in Markdown format. Site is using
  [pulldown_cmark](https://crates.io/crates/pulldown-cmark) as a markdown parser
  with extensions.
- Uses [Jinja2](http://jinja.pocoo.org/) as a template language. **Site** is
  using [tera](https://crates.io/crates/tera) template engine.
- Very simple. Blazingly fast. Build articles in parallel.
- Inspired by [Pelican](http://docs.getpelican.com/en/stable/), which is a
  static site generator written in Python. **Site** has a similar concept of
  _Articles_ and _Pages_.
- Inflexible and opinioned _by design_. **Site** implements only what the author
  needs to build a simple static site, like
  [https://hayatoito.github.io/](https://hayatoito.github.io/). If you want to
  customize, I'd recommend to fork **Site** itself. **Site** is intentionally
  kept very small so you can understand codebase and customize it easily.

# Install

```shell
cargo install site
```

# Usages

No documentations yet.

Meanwhile, as a living document, use
[hayatoito/hayatoito.github.io](https://github.com/hayatoito/hayatoito.github.io)
as your starter boilerplate.
[https://hayatoito.github.io/](https://hayatoito.github.io/) is built from that
repository.

# Folder structure

```text
root_dir/
 - src/
   - (Put your markdown files here)
 - template/
   - (Put your template files here)
```

- [`src/`](https://github.com/hayatoito/hayatoito.github.io/tree/main/src) is a
  folder where your all markdown files live. They are converted into HTML files,
  using Jinja2 template, and are copied into the output directory.

  Any other resources in `src` directory are also copied to the output
  directory.

- [`template/`](https://github.com/hayatoito/hayatoito.github.io/tree/main/template)
  is a folder where jinja2's template files live.

# Markdown format

`Site` uses markdown.

```markdown
# Article title

<!--
date: 2021-12-01
-->

# Section

Hello Article!

- hello
- world
```

Nothing special except for:

- The first section is considered as a title of the article.
- _Metadata_, such as `date`, follows.

# Metadata

TODO: Explain

| Name          | Description                               | Default value                             |
| ------------- | ----------------------------------------- | ----------------------------------------- |
| `page`        |                                           | false                                     |
| `date`        |                                           | (`date` is mandatory unless `page: true`) |
| `update_date` |                                           | NA                                        |
| `author`      |                                           | NA                                        |
| `slug`        | The page's URL                            | Calculated by a relative PATH to `src`    |
| `toc`         | Whether to generate Table of Contents     | false                                     |
| `toc_level`   |                                           | NA (arbitrary depth)                      |
| `draft`       | Skip this markdown                        | false                                     |
| `template`    | Template file to use in `template` folder | `article` or `page`                       |

# Pages

If a markdown's metadata contains `page: true`, **Site** consider that the
markdown represents a _page_, instead of an _article_.

```markdown
# Page title

<!--
page: true
-->

# Section

Hello Page!

- hello
- world
```

The differences between _article_ and _page_ are:

- A _page_ will not be included in `articles` template variable. Neither in
  `articles_by_year`.
- A _page_ doesn't have to contain `date` metadata.

# Template variables

TODO: Explain

| Name               | page | article | Description                                                |
| ------------------ | ---- | ------- | ---------------------------------------------------------- |
| `entry`            | x    | x       | Represents an article or a page (its metadata and content) |
| `site`             | x    | x       | Site configuration given by `--config` parameter           |
| `articles`         | x    |         | The list of the articles                                   |
| `articles_by_year` | x    |         | The list of { year, articles}                              |

- `articles` and `articles_by_year` are only available in a page. In other
  words, an article can't know other articles.

## `entry`

In addition to its metadata, `entry` contains the following fields:

| Name             | Description                    |
| ---------------- | ------------------------------ |
| `entry.title`    | Title                          |
| `entry.content`  | Generated HTML                 |
| `entry.toc_html` | Generated TOC (if `toc: true`) |

# Build

## CLI

```shell
site build --root-dir . --config=config.toml --out-dir out
```

`root-dir` should contain `src` and `template` folders.

See
[Make.zsh](https://github.com/hayatoito/hayatoito.github.io/blob/main/Make.zsh)
for the example CLI usages for various tasks.

## GitHub Action

You can also use GitHub Action to build and deploy automatically if you are
using GitHub Pages. See
[build.yml](https://github.com/hayatoito/hayatoito.github.io/blob/main/.github/workflows/build.yml)
as an example.
