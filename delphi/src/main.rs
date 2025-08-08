use height_oracle::guess_height_prebip34block_unchecked;

fn main() {
    let rex = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: delphi <block-id-rex>");
        std::process::exit(1);
    });

    let block_hash = parse_block_hash(&rex).expect("Invalid block id");
    let height = guess_height_prebip34block_unchecked(&block_hash);

    println!("{}", height);
}

fn parse_block_hash(rex: &str) -> Result<[u8; 32], ()> {
    if rex.len() != 64 {
        panic!("Block id must be 64 characters");
    }

    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let start = i * 2;
        let end = start + 2;
        let byte_str = &rex[start..end];
        let byte = u8::from_str_radix(byte_str, 16).expect("Invalid hex byte");
        bytes[i] = byte;
    }

    // Bitcoin uses reverse hex, so reverse bytes to network byte order
    bytes.reverse();
    Ok(bytes)
}
