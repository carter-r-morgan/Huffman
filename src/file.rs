use std::iter;
use std::io::{self, Read, Write};
use std::collections::HashMap;
use bitvec::prelude::*;
use crate::compression::{self, HuffmanNode};

pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) {
    let mut text = String::new();
    input.read_to_string(&mut text).expect("File must be utf8-encoded.");
    let table = compression::count_chars(&text);
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

pub fn decode<R: Read, W: Write>(input: &mut R, output: &mut W) {
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