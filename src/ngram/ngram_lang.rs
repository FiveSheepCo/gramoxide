use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NgramLang {
    /// English
    En,
    /// American English
    EnUs,
    /// British English
    EnGb,
    /// German
    De,
    /// French
    Fr,
    /// Spanish
    Es,
}

impl NgramLang {
    /// Get ngram language identification string
    pub fn to_ngram_lang_str(&self) -> &'static str {
        match self {
            Self::En => "eng",
            Self::EnUs => "eng-us",
            Self::EnGb => "eng-gb",
            Self::De => "ger",
            Self::Fr => "fre",
            Self::Es => "spa",
        }
    }
}
