use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::path::Path;
use std::{env, fs};
use bitvec::prelude::*;


fn main() {
    let args = env::args();
    let config = match Config::build(args) {
        Ok(cfg) => cfg,
        Err(msg) => { println!("{msg}"); return; },
    };

    match config.mode {
        Mode::Encode => encode_file(config.input_path, config.output_path),
        Mode::Decode => decode_file(config.input_path, config.output_path),
    }

    test();

    /* hzip [encode|decode] input-name [-o output-name]
     * Parse arguments
     * 
     */
}

#[derive(Debug)]
enum Mode {
    Encode,
    Decode
}

#[derive(Debug)]
struct Config {
    mode: Mode,
    input_path: String,
    output_path: String
}

impl Config {
    fn build(mut args: impl Iterator<Item=String>) -> Result<Config, &'static str> {
        let (mode, input_path) = match (args.nth(1), args.next()) {
            (Some(m), Some(ip)) => (m, ip),
            _ => return Err("Not enough arguments"),
        };

        let mode = if mode.eq_ignore_ascii_case("encode") {
            Mode::Encode
        }
        else if mode.eq_ignore_ascii_case("decode") {
            Mode::Decode
        }
        else {
            return Err("Unknown mode");
        };

        let output_path = match (args.next(), args.next()) {
            (Some(flag), Some(value)) if flag == "-o" => value,
            _ => String::from("out.txt"),
        };

        Ok( Config {mode, input_path, output_path} )
    }
}

fn encode_file<P: AsRef<Path>>(input: P, output: P) {
    let text = fs::read_to_string(input);
    let table = count_chars(text);
    let tree = HuffmanNode::build_tree(table);
    let encoded_text = tree.encode(text);


    fs::write(output, output_data);
}

fn decode_file<P: AsRef<Path>>(input: P, output: P) {
    
}

// -----------------------------------------------------------

fn test() {
    let _unbalanced = "abcdddeeeefffffffggggggggggg";
    let _balanced = "abcdefg";

    let _real = "It was the best of times, it was the worst of times,
    it was the age of wisdom, it was the age of foolishness,
    it was the epoch of belief, it was the epoch of incredulity,
    it was the season of light, it was the season of darkness,
    it was the spring of hope, it was the winter of despair.";

    let input = _real;

    let root = HuffmanNode::build_tree(count_chars(input));
    let root = Box::new(root);
    print_dot(&root);

    for (c, code) in root.get_char_codes().iter() {
        println!("{c}: {code:b}");
    }

    let code = root.encode(input);
    let output = root.decode(code.as_bitslice());

    println!("Input: {input}\n\nData: {code:b}\n\nOutput: {output}");
}

fn print_dot(node: &Box<HuffmanNode>) {
    if let (Some(left), Some(right)) = (&node.left, &node.right) {
        println!("{}{} -> {}{};", node.value, node.weight, left.value, left.weight);
        print_dot(&left);
        println!("{}{} -> {}{};", node.value, node.weight, right.value, right.weight);
        print_dot(&right);
    }
}

struct HuffmanNode {
    value: char,
    weight: u32,
    left: Option<Box<HuffmanNode>>, 
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    fn leaf(value: char, weight: u32) -> HuffmanNode {
        HuffmanNode {
            value: value,
            weight: weight,
            left: None,
            right: None
        }
    }

    fn internal(left: HuffmanNode, right: HuffmanNode) -> HuffmanNode {
        HuffmanNode {
            value: left.value,
            weight: left.weight + right.weight,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        }
    }

    fn build_tree(table: HashMap<char, u32>) -> HuffmanNode {
        let mut heap = BinaryHeap::from_iter(
            table.iter().map(|(k, v)| Reverse(HuffmanNode::leaf(*k, *v)))
        );

        while heap.len() >= 2 {
            let Reverse(left) = heap.pop().unwrap();
            let Reverse(right) = heap.pop().unwrap();
            
            heap.push(Reverse(HuffmanNode::internal(left, right)));
        }
        
        heap.pop().expect("character set should not be empty").0
    }

    fn get_char_codes(&self) -> HashMap<char, BitVec<u8, Msb0>> {
        let mut stack = vec![(self, BitVec::new())];
        let mut char_codes = HashMap::new();

        while let Some((node, mut bitcode)) = stack.pop() {
            if let (Some(left), Some(right)) = (&node.left, &node.right) {
                let mut rightcode = bitcode.clone();
                bitcode.push(false);
                rightcode.push(true);

                stack.push((left, bitcode));
                stack.push((right, rightcode));
            }
            else {
                char_codes.insert(node.value, bitcode);
            }
        }

        char_codes
    }

    fn encode(&self, text: &str) -> BitVec<u8, Msb0> {
        let codes = self.get_char_codes();
        let mut output = BitVec::new();

        for c in text.chars() {
            output.extend_from_bitslice(&codes[&c]);
        }

        output
    }

    fn decode(&self, data: &BitSlice<u8, Msb0>) -> String {
        let mut node = self;
        let mut output = String::new();

        for bit in data.iter().by_vals() {
            if bit {
                node = node.right.as_ref().unwrap();
            }
            else {
                node = node.left.as_ref().unwrap();
            }

            if let (None, None) = (&node.left, &node.right) {
                output.push(node.value);
                node = self;
            }
        }

        output
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.weight == other.weight
    }
}

impl Eq for HuffmanNode {}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.weight != other.weight {
            self.weight.cmp(&other.weight)
        }
        else {
            self.value.cmp(&other.value)
        }
    }
}

fn count_chars(text: &str) -> HashMap<char, u32> {
    let mut char_counts = HashMap::new();

    for c in text.chars() {
        char_counts.entry(c)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    char_counts
}

