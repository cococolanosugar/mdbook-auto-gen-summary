# mdbook-auto-gen-summary

A preprocessor and cli tool for mdbook to auto generate summary.

#### install

```bash
cargo install mdbook-auto-gen-summary
```

It can be use in two ways:

#### 1. Use as a cli tool.

```bash
mdbook-auto-gen-summary gen /path/to/your/mdbook/src

or

mdbook-auto-gen-summary gen -t /path/to/your/mdbook/src # -t indicate mdbook to make the first line(default the file name) of markdown file as the link text in SUMMARY.md 
```

This will walk your mdbook src dir and generate the book summary in /path/to/your/mdbook/src/SUMMARY.md

#### 2. Use as mdbook preprocessor.

```bash
#cat /path/to/your/mdbook/book.toml

[book]
authors = []
language = "en"
multilingual = false
src = "src"

[build]
create-missing = false

#use as mdbook preprocessor
[preprocessor.auto-gen-summary]
first-line-as-link-text = true # indicate mdbook to make the first line(default the file name) of markdown file as the link text in SUMMARY.md 

[output.html.fold]
enable = true
level = 0

```

When you run 
```bash
mdbook serve
```
Or
```bash
mdbook build
```
this will also walk your mdbook src dir and generate the book summary in /path/to/your/mdbook/src/SUMMARY.md



