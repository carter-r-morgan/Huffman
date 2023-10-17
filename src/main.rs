use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::io::{self, Read, Write};
use std::fs::File;
use std::env;
use std::iter;
use bitvec::prelude::*;


fn main() {
    let args = env::args();
    let config = match Config::build(args) {
        Ok(cfg) => cfg,
        Err(msg) => { println!("{msg}"); return; },
    };

    let mut input = File::open(config.input_path).unwrap();
    let mut output = File::create(config.output_path).unwrap();

    match config.mode {
        Mode::Encode => encode_file(&mut input, &mut output),
        Mode::Decode => decode_file(&mut input, &mut output),
    }

    // test();

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

fn encode_file<R: Read, W: Write>(input: &mut R, output: &mut W) {
    let mut text = String::new();
    input.read_to_string(&mut text).expect("File must be utf8-encoded.");
    let table = count_chars(&text);
    let (chars, counts): (String, Vec<u64>) = table.iter().unzip();
    let encoded_text = match HuffmanNode::build_tree(&table) {
        Some(tree) => tree.encode(&text),
        None => BitVec::<u8, Msb0>::new(),
    };

    let mut data_out: Vec<u8> = Vec::new();

    let chars_start: u64 = 40;
    let counts_start: u64 = chars_start + chars.len() as u64;
    let data_start: u64 = counts_start + (counts.len() << 3) as u64;

    data_out.extend((text.len() as u64).to_be_bytes());
    data_out.extend((table.len() as u64).to_be_bytes());
    data_out.extend(chars_start.to_be_bytes());
    data_out.extend(counts_start.to_be_bytes());
    data_out.extend(data_start.to_be_bytes());

    data_out.extend(chars.bytes());
    data_out.extend(counts.iter().flat_map(|count| count.to_be_bytes()));
    data_out.extend(encoded_text.as_raw_slice());

    output.write_all(&data_out).expect("I dunno, how could this go wrong?");
}

fn decode_file<R: Read, W: Write>(input: &mut R, output: &mut W) {
    let text_len = read_u64(input).unwrap();
    let table_len = read_u64(input).unwrap();
    let chars_start = read_u64(input).unwrap();
    let counts_start = read_u64(input).unwrap();
    let data_start = read_u64(input).unwrap();
    
    let chars_len = counts_start - chars_start;
    let mut chars = String::new();
    input.by_ref().take(chars_len).read_to_string(&mut chars);

    let mut counts = Vec::new();
    for _ in 0..table_len {
        counts.push(read_u64(input).unwrap());
    }

    let mut data = Vec::new();
    input.read_to_end(&mut data).unwrap();

    let table = chars.chars().zip(counts).collect::<HashMap<_,_>>();
    let decoded_text = match HuffmanNode::build_tree(&table) {
        Some(tree) => tree.decode(data.view_bits::<Msb0>()),
        None => match table.iter().next() {
            Some((&character, &count)) => iter::repeat(character).take(count as usize).collect::<String>(),
            None => String::new(),
        },
    };

    output.write_all(decoded_text.as_bytes());
}

fn read_u64<R: Read>(input: &mut R) -> io::Result<u64> {
    let mut buffer = [0; 8];
    input.read_exact(&mut buffer)?;
    return Ok(u64::from_be_bytes(buffer));
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

    let root = HuffmanNode::build_tree(&count_chars(input)).unwrap();
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
    weight: u64,
    left: Option<Box<HuffmanNode>>, 
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    fn leaf(value: char, weight: u64) -> HuffmanNode {
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

    fn build_tree(table: &HashMap<char, u64>) -> Option<HuffmanNode> {
        if table.len() <= 1 { return None; }

        let mut heap = BinaryHeap::from_iter(
            table.iter().map(|(k, v)| Reverse(HuffmanNode::leaf(*k, *v)))
        );

        while heap.len() >= 2 {
            let Reverse(left) = heap.pop().unwrap();
            let Reverse(right) = heap.pop().unwrap();
            
            heap.push(Reverse(HuffmanNode::internal(left, right)));
        }
        
        return Some(heap.pop().unwrap().0);
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
        let mut char_count = 0;

        for bit in data.iter().by_vals() {
            if bit {
                node = node.right.as_ref().unwrap();
            }
            else {
                node = node.left.as_ref().unwrap();
            }

            if let (None, None) = (&node.left, &node.right) {
                output.push(node.value);
                char_count += 1;
                if char_count == self.weight { break; }

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

fn count_chars(text: &str) -> HashMap<char, u64> {
    let mut char_counts = HashMap::new();

    for c in text.chars() {
        char_counts.entry(c)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    char_counts
}

