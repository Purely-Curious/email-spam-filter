#![allow(dead_code)]

extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};
use std::{error::Error};
use std::fs;
use std::collections::HashMap;
use csv;
use serde::Deserialize;

// p(non_spam) =  #(non_spam messages) /  (total messages)
// p(messages being non_spam) = p(non_spam) * p(words of the messages given that they're not spam) [training data.]

// calculate the probabilites of the message being spam and non_spam and which ever is greater is the label. [if n_s is greater, then label it as non_spam].

// if a word appears several times in the email or a word wasn't present in the training data. [ i.e. p(word) = 0 ]

// if p(word) = 0, then p(word) = #(no of word in category) / #(words in category)

// A structure for hosting the junk data from the csv file.
#[derive(Debug, Deserialize)]
struct UnprocessedText {
    text: String,
    spam: u8,
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
            _text: stemming(remove_stop_words(text)),
            spam_or_not: self.spam,
        }
    }
}

// A structure for hosting the processed (stemming and stop word removal) junk data 
// for further analysis
#[derive(Debug, Clone)]
struct ProcessedText {
    _text: Vec<String>,
    spam_or_not: u8,
}

impl ProcessedText {
    // fn _word_specifier(&self, non_spam_wordset: &mut Vec<String>, spam_wordset: &mut Vec<String>) {}
}

#[derive(Debug)]
struct WordClassifier {
    spam_word_count: HashMap<String, usize>,
    non_spam_word_count: HashMap<String, usize>,
    spam_words: HashMap<String, f32>,
    non_spam_words: HashMap<String, f32>,
    total_spam_words: usize,
    total_non_spam_words: usize,
}

impl WordClassifier {
    fn new(processed_texts: Vec<ProcessedText>) -> Self {
        let mut spam_word_count: HashMap<String, usize> = HashMap::new();
        let mut non_spam_word_count: HashMap<String, usize> = HashMap::new();
        
        for processed_text in processed_texts {
            for word in processed_text._text {
                if processed_text.spam_or_not == 1 {
                    spam_word_count.insert(word, 0);
                }
                else {
                    non_spam_word_count.insert(word, 0);
                }
            }
        }
        WordClassifier {
            spam_word_count,
            non_spam_word_count,
            spam_words: HashMap::default(),
            non_spam_words: HashMap::default(),
            total_spam_words: 0,
            total_non_spam_words: 0,
        }
    }
    // This creates a hashmap so that the words are correctly classified into their proper categories.
    fn word_specifier(&mut self, processed_text: ProcessedText) {
        if processed_text.spam_or_not == 1 {
            for word in processed_text._text {
                *self.spam_word_count.get_mut(&word).unwrap() +=1;
                self.total_spam_words += 1;
            }
        }
        else {
            for word in processed_text._text {
                *self.non_spam_word_count.get_mut(&word).unwrap() +=1;
                self.total_non_spam_words += 1;
            }
        }
    }

    fn update_classification(&mut self) {
        let spam_words = self.spam_word_count.keys();
        for word in spam_words {
            // need to added one to each word in the word classifier to ensure that no emails are accidentally misattributed.
            //*self.spam_word_count.get_mut(word).unwrap() +=1;
            let count: usize = *self.spam_word_count.get(word).unwrap(); 
            self.spam_words.insert(word.to_string(), count as f32 / self.total_spam_words as f32);
        }
        for word in self.non_spam_word_count.keys() {
            let count: usize = *self.non_spam_word_count.get(word).unwrap(); 
            self.non_spam_words.insert(word.to_string(), count as f32 / self.total_non_spam_words as f32);
        }
    }
}

struct EmailClassifier {
    total_emails: Vec<ProcessedText>,
    email_is_spam: f32,
    email_is_not_spam: f32,
    word_classifier: WordClassifier,
}
impl EmailClassifier {
    fn new(training_data: Vec<ProcessedText>) -> Self {
        Self {
            total_emails: training_data.clone(),
            email_is_spam: 0.0,
            email_is_not_spam: 0.0,
            word_classifier: WordClassifier::new(training_data)
        }
    }

    fn update_training_classification(&mut self) {

        println!("Starting configuration.");

        // update the records in the word classifier.
        for record in self.total_emails.clone() {
            self.word_classifier.word_specifier(record);
        }
        let amount_of_emails = self.total_emails.len() as f32;
        for email in &self.total_emails {
            if email.spam_or_not == 0 {
                self.email_is_spam += 1.0;
            }
            else {
                self.email_is_not_spam += 1.0;
            }
        }
        // gathering the probablities of emails being spam or ham.
        self.email_is_not_spam /= amount_of_emails;
        self.email_is_spam /= amount_of_emails;

        println!("Ending configuration.");
    }

    fn email_spam_probablity(&self, email: &ProcessedText) -> f32 {
        let mut result= self.email_is_spam;
        for word in email.clone()._text {
            result *= self.word_classifier.spam_words.get(&word).unwrap();
        }
        result
    }

    fn email_non_spam_probablity(&self, email: &ProcessedText) -> f32 {
        let mut result = self.email_is_not_spam;
        for word in email.clone()._text {
            result *= self.word_classifier.non_spam_words.get(&word).unwrap();
        }
        result
    }
    // the function that classifies the emails.
    fn classify_email(&self, email: &mut ProcessedText) {
        let email_spam_chance = self.email_spam_probablity(&email);
        let email_non_spam_chance = self.email_non_spam_probablity(&email);

        if email_non_spam_chance > email_spam_chance {
            email.spam_or_not = 1;
        }
        else {
            email.spam_or_not = 0;
        }
    }
}


// A function that'll load stop-words from a particular file.
fn load_stop_words(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let unformatted_stop_words = fs::read_to_string(path)?;
    let stop_words = unformatted_stop_words.lines()
        .map(|s| s.trim_end().to_string())
        .collect();
    
    Ok(stop_words)
}

// required to see if a particular word is present in the text.
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
    let stop_words = load_stop_words("stopwords2.txt").unwrap();
    let mut processed_text: Vec<String> = text.clone();
    
    for word in stop_words {
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


fn _correlation_based_feature() {}

fn read_in_emails(path: &str) -> Result<Vec<ProcessedText>, Box<dyn Error>> {

    let mut rdr = csv::Reader::from_path(path)?;

    let mut processed_records: Vec<ProcessedText> = vec![];
    let _headers = rdr.headers()?;

    for result in rdr.deserialize() {
        let record: UnprocessedText = result?;
        let processed_record: ProcessedText = record.processing_text();
        processed_records.push(processed_record);
        }
    
    Ok(processed_records)
}



fn main() {
    let training_records: Vec<ProcessedText> = read_in_emails("emails.csv").unwrap();
    // this is the email classifier made with the training data used for the classifier.
    let mut email_classifier: EmailClassifier = EmailClassifier::new(training_records);
    email_classifier.update_training_classification();
}
