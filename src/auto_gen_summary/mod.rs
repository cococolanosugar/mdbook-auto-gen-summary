use hex;
use md5::{Digest, Md5};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::MDBook;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::fs::DirEntry;

const SUMMARY_FILE: &str = "SUMMARY.md";
const README_FILE: &str = "README.md";

const FIRST_LINE_AS_LINK_TEXT: &str = "first-line-as-link-text";

#[derive(Debug)]
pub struct MdFile {
    pub name: String,
    pub title: String,
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
        "auto-gen-summary"
    }

    fn run(&self, ctx: &PreprocessorContext, _book: Book) -> Result<Book, Error> {
        let mut use_first_line_as_link_text = false;

        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name()) {
            if nop_cfg.contains_key("blow-up") {
                anyhow::bail!("Boom!!1!");
            }
            if nop_cfg.contains_key(FIRST_LINE_AS_LINK_TEXT) {
                let v = nop_cfg.get(FIRST_LINE_AS_LINK_TEXT).unwrap();
                use_first_line_as_link_text = v.as_bool().unwrap_or(false);
            }
        }

        let source_dir = ctx
            .root
            .join(&ctx.config.book.src)
            .to_str()
            .unwrap()
            .to_string();

        gen_summary(&source_dir, use_first_line_as_link_text);

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

fn md5(buf: &String) -> String {
    let mut hasher = Md5::new();
    hasher.update(buf.as_bytes());
    let f = hasher.finalize();
    let md5_vec = f.as_slice();
    let md5_string = hex::encode_upper(md5_vec);

    return md5_string;
}

pub fn gen_summary(source_dir: &String, use_first_line_as_link_text: bool) {
    let mut source_dir = source_dir.clone();
    if !source_dir.ends_with("/") {
        source_dir.push_str("/")
    }
    let group = walk_dir(source_dir.clone().as_str());
    let lines = gen_summary_lines(source_dir.clone().as_str(), &group, use_first_line_as_link_text);
    let buff: String = lines.join("\n");

    let new_md5_string = md5(&buff);

    let summary_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(source_dir.clone() + "/" + SUMMARY_FILE)
        .unwrap();

    let mut old_summary_file_content = String::new();
    let mut summary_file_reader = BufReader::new(summary_file);
    summary_file_reader.read_to_string(&mut old_summary_file_content).unwrap();

    let old_md5_string = md5(&old_summary_file_content);

    if new_md5_string == old_md5_string {
        return;
    }

    let summary_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(source_dir.clone() + "/" + SUMMARY_FILE)
        .unwrap();
    let mut summary_file_writer = BufWriter::new(summary_file);
    summary_file_writer.write_all(buff.as_bytes()).unwrap();
}

fn count(s: &String) -> usize {
    let v: Vec<&str> = s.split("/").collect();
    let cnt = v.len();
    cnt
}

fn gen_summary_lines(root_dir: &str, group: &MdGroup, use_first_line_as_link_text: bool) -> Vec<String> {
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

        buff_link = format!("{}* [{}]({})", buff_spaces, name, README_FILE);
    } else {
        buff_link = format!("{}* [{}]({}/{})", buff_spaces, name, path, README_FILE);
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
        if path == SUMMARY_FILE {
            continue;
        }
        if path.ends_with(README_FILE) {
            continue;
        }

        let cnt = count(&path);
        let buff_spaces = String::from(" ".repeat(4 * (cnt - 1)));

        let buff_link: String;
        if use_first_line_as_link_text && md.title.len() > 0 {
            buff_link = format!("{}* [{}]({})", buff_spaces, md.title, path);
        } else {
            buff_link = format!("{}* [{}]({})", buff_spaces, md.name, path);
        }

        lines.push(buff_link);
    }

    for group in &group.group_list {
        let mut line = gen_summary_lines(root_dir, group, use_first_line_as_link_text);
        lines.append(&mut line);
    }

    lines
}

fn get_title(entry: &DirEntry) -> String {
    let md_file = std::fs::File::open(entry.path().to_str().unwrap()).unwrap();
    let mut md_file_content = String::new();
    let mut md_file_reader = BufReader::new(md_file);
    md_file_reader.read_to_string(&mut md_file_content).unwrap();
    let lines = md_file_content.split("\n");

    let mut title: String = "".to_string();
    let mut first_h1_line = "";
    for line in lines {
        if line.starts_with("# ") {
            first_h1_line = line.trim_matches('#').trim();
            break;
        }
    }

    if first_h1_line.len() > 0 {
        title = first_h1_line.to_string();
    }

    return title;
}

fn walk_dir(dir: &str) -> MdGroup {
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
        if file_name == README_FILE {
            group.has_readme = true;
        }
        let arr: Vec<&str> = file_name.split(".").collect();
        if arr.len() < 2 {
            continue;
        }
        let file_name = arr[0];
        let file_ext = arr[1];
        if file_ext.to_lowercase() != "md" {
            continue;
        }

        let title = get_title(&entry);

        let md = MdFile {
            name: file_name.to_string(),
            title,
            path: entry.path().to_str().unwrap().to_string(),
        };

        group.md_list.push(md);
    }

    return group;
}
