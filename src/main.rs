use tree_sitter::{Parser, Node};
use std::env;
use std::fs;

fn main() {
    let filename = env::args()
        .nth(1)
        .expect("Usage: pytext-extract <file.py>");

    let source = fs::read_to_string(&filename)
        .expect("Failed to read file");

    let mut parser = Parser::new();


    parser
        .set_language(tree_sitter_python::language())
        .expect("Error loading Python grammar");
    let tree = parser.parse(&source, None)
        .expect("Failed to parse");

    // Collect "keep" byte ranges as (start, end).
    let mut keep_ranges: Vec<(usize, usize)> = Vec::new();
    walk(tree.root_node(), &mut keep_ranges);

    // Sort ranges so we can scan the file efficiently.
    keep_ranges.sort_by_key(|r| r.0);

    let mut output = source.clone().into_bytes();

    let mut idx = 0;
    let mut r = 0;

    while idx < output.len() {
        let keep = r < keep_ranges.len()
            && idx >= keep_ranges[r].0
            && idx < keep_ranges[r].1;

        if keep {
            idx += 1;
        } else {
            if output[idx] != b'\n' {
                output[idx] = b' ';
            }
            idx += 1;

            // Move to next keep-range if we passed this one
            while r < keep_ranges.len() && idx >= keep_ranges[r].1 {
                r += 1;
            }
        }
    }

    print!("{}", String::from_utf8_lossy(&output));
}

// Walk the AST and record ranges we want to keep.
fn walk(node: Node, keep: &mut Vec<(usize, usize)>) {
    let kind = node.kind();

    match kind {
        // Comments: keep the whole node.
        "comment" => {
            keep.push((node.start_byte(), node.end_byte()));
        }

        // String content only (no quotes/prefixes):
        "string_content" => {
            keep.push((node.start_byte(), node.end_byte()));
        }

        // Skip f-string expressions and everything else.
        _ => {}
    }

    // Recurse
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, keep);
    }
}
