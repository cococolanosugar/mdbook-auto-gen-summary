# mdbook-auto-gen-book-summary

A preprocessor and cli tool for mdbook to auto generate summary.

#### install

```bash
cargo install mdbook-auto-gen-summary
```

It can be use in two ways:

#### 1. Use as a cli tool.

```bash
mdbook-auto-gen-summary gen /path/to/your/mdbook/src
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



