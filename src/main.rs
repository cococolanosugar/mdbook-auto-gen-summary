use crate::nop_lib::AutoGenSummary;
use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::MDBook;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use std::process;

pub fn make_app() -> App<'static, 'static> {
    App::new("nop-preprocessor")
        .about("A mdbook preprocessor which does precisely nothing")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            SubCommand::with_name("gen")
                .arg(Arg::with_name("dir").required(true))
                .about("the dir of md book src"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    // Users will want to construct their own preprocessor here
    let preprocessor = AutoGenSummary::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Some(sub_args) = matches.subcommand_matches("gen") {
        let source_dir = sub_args
            .value_of("dir")
            .expect("Required argument")
            .to_string();
        println!("{:?}", source_dir);
        let g = nop_lib::walk_dir((source_dir.clone() + "/").as_str());
        let list = nop_lib::gen_summary((source_dir.clone() + "/").as_str(), &g);
        let buf: String = list.join("\n");
        let mut f = std::fs::File::create(source_dir.clone() + "/SUMMARY.md").unwrap();
        let mut writer = BufWriter::new(f);
        writer.write(buf.as_bytes());
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

/// The actual implementation of the `Nop` preprocessor. This would usually go
/// in your main `lib.rs` file.
mod nop_lib {
    use super::*;
    use std::fs;

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

    /// A no-op preprocessor.
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

        fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
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
            let g = walk_dir((source_dir.clone() + "/").as_str());
            let list = gen_summary((source_dir.clone() + "/").as_str(), &g);
            let buf: String = list.join("\n");
            let mut f = std::fs::File::create(source_dir.clone() + "/SUMMARY.md").unwrap();
            let mut writer = BufWriter::new(f);
            writer.write(buf.as_bytes());

            match MDBook::load(&ctx.root) {
                Ok(mdbook) => {
                    return Ok(mdbook.book);
                }
                Err(e) => {
                    panic!(e);
                }
            };

            Ok(book)
        }

        fn supports_renderer(&self, renderer: &str) -> bool {
            renderer != "not-supported"
        }
    }

    pub fn gen_summary(root_dir: &str, group: &MdGroup) -> Vec<String> {
        let mut list: Vec<String> = vec![];

        let mut path = group.path.replace(root_dir, "");
        let sl: Vec<&str> = path.split("/").collect();
        let cnt = sl.len();
        let buf = String::from(" ".repeat(4 * (cnt - 1)));
        let mut name = group.name.clone();

        let mut buf2 = String::new();
        if name == "src" {
            name = String::from("Welcome");
        }
        if path == "" {
            list.push(String::from("# Summary"));

            buf2 = format!("{}* [{}](README.md)", buf, name);
        } else {
            buf2 = format!("{}* [{}]({}/README.md)", buf, name, path);
        }

        if buf.len() == 0 {
            list.push(String::from("\n"));
            list.push(String::from("----"));
        }

        list.push(buf2);

        for md in &group.md_list {
            let path = md.path.replace(root_dir, "");
            if path == "SUMMARY.md" {
                continue;
            }
            if path.ends_with("README.md") {
                continue;
            }
            let sl: Vec<&str> = path.split("/").collect();
            let cnt = sl.len();
            let buf = String::from(" ".repeat(4 * (cnt - 1)));
            let buf2 = format!("{}* [{}]({})", buf, md.name, path);
            list.push(buf2);
        }

        for g in &group.group_list {
            let mut l = gen_summary(root_dir, g);
            list.append(&mut l);
        }

        list
    }

    pub fn walk_dir(dir: &str) -> MdGroup {
        let readDir = fs::read_dir(dir).unwrap();
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

        for e in readDir {
            let entry = e.unwrap();
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
}
