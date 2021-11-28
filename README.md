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

# Directory structure

```text
root_dir/
 - src/
   - (Put your markdown files here)
 - template/
   - (Put your template files here)
```

- [`src/`](https://github.com/hayatoito/hayatoito.github.io/tree/main/src) is a
  directory where your all markdown files live. They are converted into HTML
  files, using Jinja2 template, and are copied into the output directory.

  Any other resources in `src` directory are also copied to the output
  directory.

- [`template/`](https://github.com/hayatoito/hayatoito.github.io/tree/main/template)
  is a directory where jinja2's template files live.

# Markdown

`site` supports markdown:

```text
# Article title

<!--
date: 2021-12-01
-->

# Section

Hello world!
```

Nothing special except for:

- The first section is considered as a title of the article.
- _Metadata_, such as `date` follows.

# Metadata

TODO: Explain

# Template variables

TODO: Explain

# Build

## CLI

```shell
site build --root-dir . --config=config.toml --out-dir out
```

`root-dir` should contain `src` and `template` directories.

See
[Make.zsh](https://github.com/hayatoito/hayatoito.github.io/blob/main/Make.zsh)
for the example CLI for various tasks.

## GitHub Action

You can also use GitHub Action to build and deploy automatically if you are
using GitHub pages. See
[build.yml](https://github.com/hayatoito/hayatoito.github.io/blob/main/.github/workflows/build.yml)
as an example.
