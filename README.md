# Site

[![Build Status](https://travis-ci.com/hayatoito/site.svg?branch=master)](https://travis-ci.com/hayatoito/site)
[![crates.io](https://img.shields.io/crates/v/site.svg)](https://crates.io/crates/site)

**Site** is a fast, simple, and opinioned static site generator, written in
[Rust](https://www.rust-lang.org/). Highlights includes:

- Write your content in Markdown format. Site is using
  [pulldown_cmark](https://crates.io/crates/pulldown-cmark).
- Uses [Jinja2](http://jinja.pocoo.org/) template system. Site is using
  [tera](https://crates.io/crates/tera) template engine.
- Very simple. Blazingly fast.
- Inspired by [Pelican](http://docs.getpelican.com/en/stable/), which is a
  static site generator written in Python. Site has a similar concept of
  _Articles_ and _Pages_.
- Inflexible and opinioned _by design_. Site implements only what
  [hayato.io](https://hayato.io/) needs. If you want to customize, I'd recommend
  to fork Site itself and update the forked one. Site is intentionally kept very
  small so you can understand codebase and customize it easily.

# Quick start

## Install

```shell
cargo install site
```

## Usages

No documentations yet.

Use this GitHub [repository](https://github.com/hayatoito/hayatoito.github.io)
as your starter boilerplate, which is the source of
[hayato.io](https://hayato.io/).
[Makefile](https://github.com/hayatoito/hayatoito.github.io/blob/source/Makefile)
there includes the example usages of `Site`.
