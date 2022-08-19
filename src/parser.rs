use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/corpus.pest"]
pub struct CorpusParser;

pub struct CorpusEntry {
    word: String,
    year: u32,
    frequency: u32,
}

impl CorpusEntry {
    pub fn word(&self) -> &str {
        self.word.as_ref()
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn frequency(&self) -> u32 {
        self.frequency
    }
}

pub struct CorpusContent {
    pub entries: Vec<CorpusEntry>,
}

pub fn parse_corpus<S>(str: S) -> Result<CorpusContent, pest::error::Error<Rule>>
where
    S: AsRef<str>,
{
    let parser = CorpusParser::parse(Rule::corpus, str.as_ref())?
        .next()
        .unwrap();
    for line in parser.into_inner() {
        match line.as_rule() {
            Rule::entry => {
                println!("line: {:?}", line);
            }
            _ => unreachable!(),
        }
    }
    Ok(CorpusContent { entries: vec![] })
}

pub fn parse_corpus_entry<S>(str: S) -> Result<CorpusEntry, pest::error::Error<Rule>>
where
    S: AsRef<str>,
{
    let parser = CorpusParser::parse(Rule::entry, str.as_ref())?
        .next()
        .unwrap();
    let mut pair = parser.into_inner();
    let word = pair.next().unwrap().as_str();
    let year: u32 = pair.next().unwrap().as_str().parse().unwrap();
    let frequency: u32 = pair.next().unwrap().as_str().parse().unwrap();
    Ok(CorpusEntry {
        word: word.to_string(),
        year,
        frequency,
    })
}
