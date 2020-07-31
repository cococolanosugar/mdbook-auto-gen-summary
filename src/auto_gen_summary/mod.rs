use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::MDBook;
use std::fs;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

#[derive(Debug)]
pub struct MdFile {
    pub name: String,
    pub path: String,
}

#[derive(Debug)]
pub struct MdGroup {
    pub name: String,
    pub path: String,
    pub has_readme: bool,
    pub group_list: Vec<MdGroup>,
    pub md_list: Vec<MdFile>,
}

pub struct AutoGenSummary;

impl AutoGenSummary {
    pub fn new() -> AutoGenSummary {
        AutoGenSummary
    }
}

impl Preprocessor for AutoGenSummary {
    fn name(&self) -> &str {
        "auto-gen-summary-preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, _book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name()) {
            if nop_cfg.contains_key("blow-up") {
                anyhow::bail!("Boom!!1!");
            }
        }

        let source_dir = ctx
            .root
            .join(&ctx.config.book.src)
            .to_str()
            .unwrap()
            .to_string();

        gen_summary(&source_dir);

        match MDBook::load(&ctx.root) {
            Ok(mdbook) => {
                return Ok(mdbook.book);
            }
            Err(e) => {
                panic!(e);
            }
        };
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

pub fn gen_summary(source_dir: &String) {
    let mut source_dir = source_dir.clone();
    if !source_dir.ends_with("/") {
        source_dir.push_str("/")
    }
    let group = walk_dir(source_dir.clone().as_str());
    let lines = gen_summary_lines(source_dir.clone().as_str(), &group);
    let buff: String = lines.join("\n");
    let file = std::fs::File::create(source_dir.clone() + "/SUMMARY.md").unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(buff.as_bytes()).unwrap();
}

pub fn count(s: &String) -> usize {
    let v: Vec<&str> = s.split("/").collect();
    let cnt = v.len();
    cnt
}

pub fn gen_summary_lines(root_dir: &str, group: &MdGroup) -> Vec<String> {
    let mut lines: Vec<String> = vec![];

    let path = group.path.replace(root_dir, "");
    let cnt = count(&path);

    let buff_spaces = String::from(" ".repeat(4 * (cnt - 1)));
    let mut name = group.name.clone();

    let buff_link: String;
    if name == "src" {
        name = String::from("Welcome");
    }
    if path == "" {
        lines.push(String::from("# Summary"));

        buff_link = format!("{}* [{}](README.md)", buff_spaces, name);
    } else {
        buff_link = format!("{}* [{}]({}/README.md)", buff_spaces, name, path);
    }

    if buff_spaces.len() == 0 {
        lines.push(String::from("\n"));
        if name != "Welcome" {
            lines.push(String::from("----"));
        }
    }

    lines.push(buff_link);

    for md in &group.md_list {
        let path = md.path.replace(root_dir, "");
        if path == "SUMMARY.md" {
            continue;
        }
        if path.ends_with("README.md") {
            continue;
        }

        let cnt = count(&path);
        let buff_spaces = String::from(" ".repeat(4 * (cnt - 1)));
        let buff_link = format!("{}* [{}]({})", buff_spaces, md.name, path);
        lines.push(buff_link);
    }

    for group in &group.group_list {
        let mut line = gen_summary_lines(root_dir, group);
        lines.append(&mut line);
    }

    lines
}

pub fn walk_dir(dir: &str) -> MdGroup {
    let read_dir = fs::read_dir(dir).unwrap();
    let name = Path::new(dir)
        .file_name()
        .unwrap()
        .to_owned()
        .to_str()
        .unwrap()
        .to_string();
    let mut group = MdGroup {
        name: name,
        path: dir.to_string(),
        has_readme: false,
        group_list: vec![],
        md_list: vec![],
    };

    for entry in read_dir {
        let entry = entry.unwrap();
        // println!("{:?}", entry);
        if entry.file_type().unwrap().is_dir() {
            let g = walk_dir(entry.path().to_str().unwrap());
            if g.has_readme {
                group.group_list.push(g);
            }
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap().to_string();
        if file_name == "README.md" {
            group.has_readme = true;
        }
        let arr: Vec<&str> = file_name.split(".").collect();
        let file_name = arr[0];
        let file_ext = arr[1];
        if file_ext.to_lowercase() != "md" {
            continue;
        }

        let md = MdFile {
            name: file_name.to_string(),
            path: entry.path().to_str().unwrap().to_string(),
        };

        group.md_list.push(md);
    }

    return group;
}
