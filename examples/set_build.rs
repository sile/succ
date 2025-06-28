extern crate clap;
extern crate splay_tree;
extern crate succ;

use clap::App;
use clap::Arg;
use splay_tree::SplaySet;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;
use succ::tree::traversal;
use succ::word;

fn main() {
    let matches = App::new("set_build")
        .arg(
            Arg::with_name("type")
                .long("type")
                .short("t")
                .takes_value(true)
                .required(true)
                .possible_values(&["null", "hashset", "splayset", "parentheses"]),
        )
        .arg(Arg::with_name("show_words").long("show_words").short("w"))
        .get_matches();
    let stdin = io::stdin();
    match matches.value_of("type") {
        Some("null") => {
            let lines = traversal::ByteLines::new(stdin.lock());
            for _ in lines.into_depth_first_traversal().iter() {}
        }
        Some("hashset") => {
            let _ = HashSet::<String>::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        Some("splayset") => {
            let _ = SplaySet::from_iter(stdin.lock().lines().map(|l| l.unwrap()));
        }
        Some("parentheses") => {
            let lines = traversal::ByteLines::new(stdin.lock()).into_depth_first_traversal();
            let tree =
                succ::BalancedParensTree::<_>::new_builder(lines, word::Letters::new()).build_all();
            println!("NODES: {}", tree.len());
            println!("BYTES: {}", tree.external_byte_size());
            if matches.is_present("show_words") {
                for word in word::Words::new(tree.root()) {
                    println!("{}", String::from_utf8(word).unwrap());
                }
            }

            // let mut buf = Vec::new();
            // let mut labels = Vec::new();
            // for v in traversal::PatriciaTreeTraversal::new(tree.root()).into_depth_first_iter() {
            //     let start = buf.len();
            //     let mut is_eow = false;
            //     for l in v.label {
            //         buf.push(l.value);
            //         is_eow = l.end_of_word;
            //     }
            //     buf.push(b'\n');
            //     labels.push((start, is_eow));
            // }
            // labels.sort_by_key(|&(start, _)| &buf[start..]);
            // println!("BUF: {}, LABELS: {}", buf.len(), labels.len());

            // let lines = succ::word::DepthFirstTraversal::new(Iter {
            //     buf: buf,
            //     labels: labels,
            //     i: 0,
            // });
            // let tree = succ::BalancedParensTree::<_>::new_builder(lines, word::Letters::new())
            //     .build_all();
            // println!("NODES: {}", tree.len());
            // println!("BYTES: {}", tree.external_byte_size());
        }
        _ => unreachable!(),
    }
}

// struct Iter {
//     buf: Vec<u8>,
//     labels: Vec<(usize, bool)>,
//     i: usize,
// }
// impl Iterator for Iter {
//     type Item = Vec<u8>;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.i < self.labels.len() {
//             let start = self.labels[self.i].0;
//             self.i += 1;
//             let w =
//                 self.buf[start..].iter().take_while(|b| **b != b'\n').cloned().collect::<Vec<_>>();
//             Some(w)
//         } else {
//             None
//         }
//     }
// }
