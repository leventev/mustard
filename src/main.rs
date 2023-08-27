use std::{env, fs::File, io::Read};

mod tar;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).expect("Filename not specified");

    let mut file = File::open(filename).expect("Failed to open file");
    let mut buff = String::new();

    let _size = file.read_to_string(&mut buff).expect("Failed to read file");
    
    let reader = tar::TarReader::from_buff(buff.as_bytes());
    let mut headers = reader.headers();
    let header = headers.next();
    println!("{:?}", header);
}
