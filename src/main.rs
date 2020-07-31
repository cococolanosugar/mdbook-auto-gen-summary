mod auto_gen_summary;

use auto_gen_summary::AutoGenSummary;
use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use std::io;
// use std::io::prelude::*;
use std::process;

pub fn make_app() -> App<'static, 'static> {
    App::new("auto-gen-summary-preprocessor")
        .about("A mdbook preprocessor to auto generate book summary")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            SubCommand::with_name("gen")
                .arg(Arg::with_name("dir").required(true))
                .about("the dir of markdown book src"),
        )

}

fn main() {
    let matches = make_app().get_matches();

    let preprocessor = AutoGenSummary::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Some(sub_args) = matches.subcommand_matches("gen") {
        let source_dir = sub_args
            .value_of("dir")
            .expect("Required argument")
            .to_string();
        // println!("{:?}", source_dir);
        // let g = auto_gen_summary::walk_dir((source_dir.clone() + "/").as_str());
        // let list = auto_gen_summary::gen_summary((source_dir.clone() + "/").as_str(), &g);
        // let buf: String = list.join("\n");
        // let mut f = std::fs::File::create(source_dir.clone() + "/SUMMARY.md").unwrap();
        // let mut writer = BufWriter::new(f);
        // writer.write(buf.as_bytes());
        auto_gen_summary::gen_summary(&source_dir);
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
