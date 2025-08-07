use height_oracle::{guess_height_prebip34block_unchecked, parse_block_hash};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let block_hash = parse_block_hash(&args[1]).unwrap_or([0u8; 32]);
    let height = guess_height_prebip34block_unchecked(&block_hash);

    print!("{}\n", height);
}
