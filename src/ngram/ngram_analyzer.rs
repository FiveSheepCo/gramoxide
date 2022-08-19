use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::time::Instant;
use tokio_stream::StreamExt;

use crate::{parser::parse_corpus_entry, CorpusList};

const THRESHOLD: u32 = 0;
const YEAR_THRESHOLD: u32 = 2000;

pub struct NgramAnalyzer {
    corpus_list: CorpusList,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NgramFrequencies {
    pub frequencies: HashMap<String, u32>,
}

impl NgramAnalyzer {
    pub fn new(corpus_list: CorpusList) -> Self {
        Self { corpus_list }
    }

    pub async fn analyze(&self) -> Result<NgramFrequencies, Box<dyn std::error::Error>> {
        let mut stream = tokio_stream::iter(&self.corpus_list.list);
        let mut frequencies = HashMap::<String, u32>::new();
        // Collect frequency data
        while let Some(corpus) = stream.next().await {
            let start_time = Instant::now();
            println!("[Analyzer] Parsing {}", corpus.filename);
            let file = std::fs::File::open(Path::new(&corpus.filename))?;
            let reader = std::io::BufReader::new(file);
            let lines = std::io::BufRead::lines(reader);
            // Calculate local frequencies
            let local_frequencies: HashMap<String, u32> = lines
                .into_iter()
                .filter_map(|line| line.ok())
                .par_bridge()
                .filter_map(|line| {
                    if let Ok(corpus) = parse_corpus_entry(line) {
                        if corpus.frequency() < THRESHOLD || corpus.year() < YEAR_THRESHOLD {
                            None
                        } else {
                            Some((corpus.word().to_string(), corpus.frequency()))
                        }
                    } else {
                        None
                    }
                })
                .fold(
                    || HashMap::new(),
                    |mut acc: HashMap<String, u32>, (word, freq)| {
                        *acc.entry(word).or_default() += freq;
                        acc
                    },
                )
                .reduce_with(|mut m1, m2| {
                    for (k, v) in m2 {
                        *m1.entry(k).or_default() += v;
                    }
                    m1
                })
                .unwrap();
            println!(
                "[Analyzer] Parsing {} done (took {}s)",
                corpus.filename,
                Instant::now().duration_since(start_time).as_secs()
            );
            let local_frequencies: HashMap<String, u32> = local_frequencies
                .into_par_iter()
                // Remove any keys containing uppercase characters after the first char
                .filter(|(k, _)| !k.chars().skip(1).any(|c| c.is_uppercase()))
                // Remove two-letter keys ending in ascii punctuation
                .filter(|(k, _)| {
                    !(k.len() == 2
                        && k.chars().last().map(|c| c.is_ascii_punctuation()) == Some(true))
                })
                .collect();
            println!(
                "[Analyzer] Found {} word-frequency pairs",
                local_frequencies.len()
            );
            // Merge local frequencies into total frequencies
            println!("[Analyzer] Merging frequencies");
            let start_time = Instant::now();
            for (k, v) in local_frequencies {
                *frequencies.entry(k).or_default() += v;
            }
            println!(
                "[Analyzer] Merging frequencies done (took {}s)",
                Instant::now().duration_since(start_time).as_secs()
            );
            println!("[Analyzer] Current count: {}", frequencies.len());
        }
        // Transform frequencies
        println!(
            "[Analyzer] Total word-frequency pairs: {}",
            frequencies.len()
        );
        // Choose best candidates
        println!("[Analyzer] Choosing best candidates");
        let start_time = Instant::now();
        let mut final_frequencies = HashMap::<String, u32>::new();
        #[derive(Debug, Hash, PartialEq, Eq)]
        struct Entry {
            k: String,
            v: u32,
        }
        println!("[Analyzer] | Finding duplicates with different casing");
        let mut dups = HashMap::<Entry, Entry>::new();
        for (k, v) in frequencies.iter() {
            if k.starts_with(|c: char| c.is_uppercase()) {
                let lowercase = k.to_lowercase();
                if let Some(lowercase_v) = frequencies.get(&lowercase) {
                    dups.entry(Entry {
                        k: lowercase,
                        v: *lowercase_v,
                    })
                    .or_insert(Entry {
                        k: k.to_owned(),
                        v: *v,
                    });
                } else {
                    final_frequencies.entry(k.to_owned()).or_insert(*v);
                }
            } else if k.starts_with(|c: char| c.is_lowercase()) {
                let mut chars = k.chars();
                let uppercase = chars
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .chain(chars)
                    .collect::<String>();
                if let Some(uppercase_v) = frequencies.get(&uppercase) {
                    dups.entry(Entry {
                        k: k.to_owned(),
                        v: *v,
                    })
                    .or_insert(Entry {
                        k: uppercase,
                        v: *uppercase_v,
                    });
                } else {
                    final_frequencies.entry(k.to_owned()).or_insert(*v);
                }
            }
        }
        println!("[Analyzer] | Found {} duplicates", dups.len());
        println!("[Analyzer] | Choosing most popular casing for duplicates");
        for (lower, upper) in dups {
            let upper_frac = upper.v as f64 / (lower.v + upper.v) as f64;
            let entry = if upper_frac > 0.75 { upper } else { lower };
            println!(
                "[{}] {{ word: {}, upper_frac: {:.2}% }}",
                self.corpus_list.lang.to_ngram_lang_str(),
                entry.k,
                upper_frac * 100.0
            );
            final_frequencies.entry(entry.k).or_insert(entry.v);
        }
        println!(
            "[Analyzer] Choosing best candidates done (took {}s)",
            Instant::now().duration_since(start_time).as_secs()
        );
        println!(
            "[Analyzer] Total word-frequency pairs: {}",
            final_frequencies.len()
        );
        Ok(NgramFrequencies {
            frequencies: final_frequencies,
        })
    }
}
