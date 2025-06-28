use splay_tree::SplaySet;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use succ::tree::traversal;
use succ::word;

fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "set_build";

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    let ty = noargs::opt("type")
        .short('t')
        .ty("null | hashset | splayset | parentheses")
        .example("hashset")
        .take(&mut args)
        .then(|a| match a.value() {
            "null" | "hashset" | "splayset" | "parentheses" => Ok(a.value().to_owned()),
            _ => Err("type must be one of: null, hashset, splayset, parentheses"),
        })?;
    let show_words = noargs::flag("show_words")
        .short('w')
        .take(&mut args)
        .is_present();

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let stdin = io::stdin();
    match ty.as_str() {
        "null" => {
            let lines = traversal::ByteLines::new(stdin.lock());
            for _ in lines.into_depth_first_traversal().iter() {}
        }
        "hashset" => {
            let _ = HashSet::<String>::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        "splayset" => {
            let _ = SplaySet::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        "parentheses" => {
            let lines = traversal::ByteLines::new(stdin.lock()).into_depth_first_traversal();
            let tree =
                succ::BalancedParensTree::<_>::new_builder(lines, word::Letters::new()).build_all();
            println!("NODES: {}", tree.len());
            println!("BYTES: {}", tree.external_byte_size());
            if show_words {
                for word in word::Words::new(tree.root()) {
                    println!("{}", String::from_utf8(word).unwrap());
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
