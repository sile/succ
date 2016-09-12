extern crate succ;
extern crate clap;
extern crate splay_tree;

use std::io;
use std::io::BufRead;
use std::collections::HashSet;
use std::iter::FromIterator;
use splay_tree::SplaySet;
use clap::App;
use clap::Arg;
use succ::tree::traversal;
use succ::word;

fn main() {
    let matches = App::new("set_build")
        .arg(Arg::with_name("type")
            .long("type")
            .short("t")
            .takes_value(true)
            .required(true)
            .possible_values(&["null", "hashset", "splayset", "parentheses"]))
        .arg(Arg::with_name("show_words")
            .long("show_words")
            .short("w"))
        .get_matches();
    let stdin = io::stdin();
    match matches.value_of("type") {
        Some("null") => {
            let lines = traversal::ByteLines::new(stdin.lock());
            for _ in lines.into_depth_first_traversal().iter() {
            }
        }
        Some("hashset") => {
            let _ = HashSet::<String>::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        Some("splayset") => {
            let _ = SplaySet::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        Some("parentheses") => {
            let lines = traversal::ByteLines::new(stdin.lock()).into_depth_first_traversal();
            let tree = succ::BalancedParensTree::<_>::new_builder(lines, word::Letters::new())
                .build_all();
            println!("NODES: {}", tree.len());
            println!("BYTES: {}", tree.external_byte_size());
            if matches.is_present("show_words") {
                for word in word::Words::new(tree.root()) {
                    println!("{}", String::from_utf8(word).unwrap());
                }
            }
        }
        _ => unreachable!(),
    }
}
