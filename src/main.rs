extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};

use std::{error::Error};
use std::fs;
use csv;
use serde::Deserialize;


// use std::process::exit;
// A structure for hosting the junk data from the csv file.
#[derive(Debug, Deserialize)]
struct UnprocessedText {
    text: String,
}
impl UnprocessedText {

    // This functional statement removes the non alphanumeric characters and transforms
    // the junk email into a vector of strings. (extracted words.)
    fn processing_text(&self) -> ProcessedText {
        let text: Vec<String> = self.text.rsplit(|c: char| !c.is_ascii_alphabetic())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .rev()
            .collect::<Vec<String>>();
        ProcessedText {
            text: stemming(remove_stop_words(text)),
        }
    }
}

// A structure for hosting the processed (stemming and stop word removal) junk data 
// for further analysis
#[derive(Debug)]
struct ProcessedText {
    text: Vec<String>,
}


// A function that'll load stop-words from a particular file.
fn load_stop_words(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let unformatted_stop_words = fs::read_to_string(path)?;
    let stop_words = unformatted_stop_words.lines()
        .map(|s| s.trim_end().to_string())
        .collect();
    
    Ok(stop_words)
}

fn find(text: &Vec<String>, word: &String) -> usize {
    let mut index: usize = usize::MAX;
    for i in 0..text.len() {
        if text[i] == *word {
            index = i;
            break;
        }
    }
    index
}

// A function that will be required to remove all of the stop words from the unprocessed data.
fn remove_stop_words(text: Vec<String>) -> Vec<String>{
    let stop_words = load_stop_words("../stopwords2.txt").unwrap();
    let mut processed_text: Vec<String> = text.clone();
    
    for word in stop_words {
        //println!("{}", word);
        let mut index = find(&processed_text, &word);
        while index != usize::MAX {
            processed_text.remove(index);
            index = find(&processed_text, &word);
        }
    }

    processed_text
}


// A function to stem the processed words.
fn stemming(text: Vec<String>) -> Vec<String> {
    
    let mut stemmed_words: Vec<String> = Vec::new();
    let stemmer = Stemmer::create(Algorithm::English);
    for word in text {
        stemmed_words.push(stemmer.stem(word.to_lowercase().as_str())
            .to_string());
    }
    stemmed_words

}

fn read_in_emails(path: &str) -> Result<(), Box<dyn Error>> {

    let mut rdr = csv::Reader::from_path(path)?;
    
    let _headers = rdr.headers()?;

    for result in rdr.deserialize() {
        let record: UnprocessedText = result?;
        let processed_record: ProcessedText = record.processing_text();
        println!("\n{:?}\n", processed_record);
    }
    Ok(())
}



fn main() {
    if let Err(e) = read_in_emails("../emails.csv") {
        eprintln!("{}", e);
    }
}
