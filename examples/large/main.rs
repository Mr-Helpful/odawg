use odawg::{from_word, FlatDawg, ReadDawg, WideNode};
use std::{fs, path::Path};

fn main() {
    let file_path = Path::new(file!()).parent().unwrap();
    let words_path = file_path.join("Large.txt");
    let dawg_path = file_path.join("Large.dawg");

    let content =
        fs::read_to_string(words_path).expect("Should be able to read words file as text");

    let mut dawg: FlatDawg<WideNode> = content.split("\n").map(from_word).collect();

    println!("Dawg size = {}", dawg.len());
    println!("Dawg has {} nodes", dawg.0.len());
    dawg.unlink();
    println!("Dawg size = {}", dawg.len());
    println!("Dawg has {} nodes", dawg.0.len());
    dawg.minimise();
    println!("Dawg size = {}", dawg.len());
    println!("Dawg has {} nodes", dawg.0.len());
    dawg.trim();
    println!("Dawg size = {}", dawg.len());
    println!("Dawg has {} nodes", dawg.0.len());

    let bytes = bincode::serialize(&dawg).expect("serialisation should succeed");
    fs::write(dawg_path, bytes).expect("Should be able to write file")
}
