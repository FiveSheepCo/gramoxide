use std::{io::Read, path::Path};

use bytes::Buf;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::error::Error;

use super::NgramLang;

const NGRAM_BASE_URL: &str = "https://storage.googleapis.com/books/ngrams/books";

#[derive(Debug, Serialize, Deserialize)]
pub struct Corpus {
    url: String,
    pub filename: String,
}

pub struct NgramDownloader {
    lang: NgramLang,
}

impl NgramDownloader {
    pub fn new(lang: NgramLang) -> Self {
        Self { lang }
    }

    pub async fn download_all(&self) -> Result<Vec<Corpus>, Error> {
        let include = {
            let mut include = ('a'..='z').map(String::from).collect::<Vec<_>>();
            include.push("other".into());
            include
        };

        let corpus_list = include
            .into_iter()
            .map(|corpus_id| Corpus {
                url: format!(
                    "{base_url}/googlebooks-{lang}-all-1gram-{date}-{id}.gz",
                    base_url = NGRAM_BASE_URL,
                    lang = self.lang.to_ngram_lang_str(),
                    date = "20120701",
                    id = corpus_id
                ),
                filename: format!(
                    "1gram-{lang}-{date}-{id}.txt",
                    lang = self.lang.to_ngram_lang_str(),
                    date = "20120701",
                    id = corpus_id
                ),
            })
            .collect::<Vec<_>>();

        for corpus in corpus_list.iter() {
            self.download_corpus(corpus).await?;
        }

        Ok(corpus_list)
    }

    async fn download_corpus(&self, corpus: &Corpus) -> Result<(), Error> {
        use flate2::read::GzDecoder;

        let filename = Path::new(corpus.filename.as_str());

        // Return early if corpus already exists
        if Path::exists(Path::new(filename)) {
            println!("Corpus {} already exists, skipping.", corpus.filename);
            return Ok(());
        }

        // Download corpus
        println!("Downloading corpus {}", corpus.filename);
        let resp_bytes = reqwest::get(corpus.url.as_str())
            .await
            .map_err(|error| Error::ReqwestError { error })?
            .bytes()
            .await
            .map_err(|error| Error::ReqwestError { error })?;

        // Decompress corpus
        println!("Decompressing corpus {}", corpus.filename);
        let mut decoder = GzDecoder::new(resp_bytes.reader());
        let mut output = String::new();
        decoder
            .read_to_string(&mut output)
            .map_err(|error| Error::IoError { error })?;

        // Write corpus to file
        println!("Writing corpus {} to disk", corpus.filename);
        let mut file = tokio::fs::File::create(filename).await?;
        file.write_all(output.as_bytes()).await?;

        Ok(())
    }
}
