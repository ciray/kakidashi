use flate2::read::MultiGzDecoder;
use rand::seq::IndexedRandom;
use std::io::Read;

fn main() {
    let bytes = include_bytes!("resources/data.csv.gz");

    let mut decoder = MultiGzDecoder::new(&bytes[..]);
    let mut csv_text = Vec::new();
    decoder
        .read_to_end(&mut csv_text)
        .expect("Failed to decompress data");

    let mut reader = csv::Reader::from_reader(csv_text.as_slice());
    let records: Vec<csv::StringRecord> = reader.records().filter_map(Result::ok).collect();
    let mut rng = rand::rng();
    if let Some(random_record) = records.choose(&mut rng) {
        if let Some(text) = random_record.get(2) {
            println!("{text}");
        } else {
            println!("No 'text' column found in the selected record.");
        }
    } else {
        println!("No records found in the CSV.");
    }
}
