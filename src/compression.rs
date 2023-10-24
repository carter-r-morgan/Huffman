use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, BinaryHeap};
use bitvec::prelude::*;

fn _print_dot(node: &Box<HuffmanNode>) {
    if let (Some(left), Some(right)) = (&node.left, &node.right) {
        println!("{}{} -> {}{};", node.value, node.weight, left.value, left.weight);
        _print_dot(&left);
        println!("{}{} -> {}{};", node.value, node.weight, right.value, right.weight);
        _print_dot(&right);
    }
}

pub struct HuffmanNode {
    value: char,
    weight: u64,
    left: Option<Box<HuffmanNode>>, 
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    pub fn leaf(value: char, weight: u64) -> HuffmanNode {
        HuffmanNode {
            value: value,
            weight: weight,
            left: None,
            right: None
        }
    }

    pub fn internal(left: HuffmanNode, right: HuffmanNode) -> HuffmanNode {
        HuffmanNode {
            value: left.value,
            weight: left.weight + right.weight,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        }
    }

    pub fn build_tree(table: &HashMap<char, u64>) -> Option<HuffmanNode> {
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

    pub fn get_char_codes(&self) -> HashMap<char, BitVec<u8, Msb0>> {
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

    pub fn encode(&self, text: &str) -> BitVec<u8, Msb0> {
        let codes = self.get_char_codes();
        let mut output = BitVec::new();

        for c in text.chars() {
            output.extend_from_bitslice(&codes[&c]);
        }

        output
    }

    pub fn decode(&self, data: &BitSlice<u8, Msb0>) -> String {
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

pub fn count_chars(text: &str) -> HashMap<char, u64> {
    let mut char_counts = HashMap::new();

    for c in text.chars() {
        char_counts.entry(c)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    char_counts
}