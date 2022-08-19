use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use ngram::{Corpus, NgramDownloader, NgramLang};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
};

use crate::ngram::NgramAnalyzer;

mod error;
mod ngram;
mod parser;

const CORPUS_LIST_PATH: &str = "ngramindex.toml";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "corpus_list")]
pub struct CorpusList {
    #[serde(rename = "language")]
    lang: NgramLang,
    #[serde(rename = "part")]
    list: Vec<Corpus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorpusListContainer {
    #[serde(rename = "corpus")]
    corpus_lists: Vec<CorpusList>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus_list_path = Path::new(CORPUS_LIST_PATH);
    let corpus_container: CorpusListContainer = {
        // Read corpus list from file
        if corpus_list_path.exists() {
            let source = {
                let mut file = File::open(corpus_list_path).await?;
                let mut str = String::new();
                file.read_to_string(&mut str).await?;
                str
            };
            toml::from_str(&source)?
        }
        // Build corpus list from ngram API
        else {
            let languages = [
                NgramLang::EnUs,
                NgramLang::EnGb,
                NgramLang::De,
                NgramLang::Es,
                NgramLang::Fr,
            ];
            let mut corpus_lists = Vec::new();
            for lang in languages {
                let downloader = NgramDownloader::new(lang);
                let list = downloader.download_all().await?;
                corpus_lists.push(CorpusList { lang, list })
            }
            CorpusListContainer { corpus_lists }
        }
    };

    // Serialize corpus container to disk
    {
        let str = toml::to_string(&corpus_container)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(corpus_list_path)
            .await?;
        file.write_all(str.as_bytes()).await?;
    }

    for corpus_list in corpus_container.corpus_lists {
        println!("Found corpus for {:?}", corpus_list.lang);
        let path = PathBuf::new().join(format!("gramoxide_{}.txt", corpus_list.lang.to_ngram_lang_str()));
        let analyzer = NgramAnalyzer::new(corpus_list);
        let frequencies = analyzer.analyze().await?;
        let mut frequencies = frequencies.frequencies.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
        frequencies.sort_by(|a, b| b.1.cmp(&a.1));
        println!("Writing {} entries to file", frequencies.len());
        let file = File::create(path).await?;
        let mut writer = BufWriter::new(file);
        for (k, v) in frequencies.iter().take(100000) {
            writer.write(format!("{} {}\n", k, v).as_bytes()).await?;
        }
    }

    Ok(())
}
