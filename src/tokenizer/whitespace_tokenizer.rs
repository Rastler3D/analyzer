use std::str::CharIndices;
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use crate::language_detection::detection::LanguageDetection;
use crate::token::{LazyLanguage, LazyScript, BorrowedToken, OwnedToken};
use crate::tokenizer::token_stream::TokenStream;
use crate::tokenizer::Tokenizer;
use crate::tokenizer::TokenizerRegistry;

#[derive(Clone, Default, Deserialize, Serialize, Debug, JsonSchema)]
pub struct WhitespaceTokenizer{}


#[typetag::serde]
impl Tokenizer for WhitespaceTokenizer {
    type TokenStream<'token> = WhitespaceTokenStream<'token>;

    fn tokenize<'token>(&self, text: impl Into<LanguageDetection<'token,'token>>) -> Self::TokenStream<'token> {
        let detection = text.into();
        WhitespaceTokenStream {
            text: detection.text(),
            chars: detection.text().char_indices(),
            token: OwnedToken::new(detection.script.into(), detection.language.into())
        }
    }
}

pub struct WhitespaceTokenStream<'token> {
    text: &'token str,
    token: OwnedToken<'token>,
    chars: CharIndices<'token>,
}

impl<'token> WhitespaceTokenStream<'token> {
    // search for the end of the current token.
    fn search_token_end(&mut self) -> usize {
        (&mut self.chars)
            .filter(|(_, c)| c.is_ascii_whitespace())
            .map(|(offset, _)| offset)
            .next()
            .unwrap_or(self.text.len())
    }
}

impl<'token> TokenStream<'token> for WhitespaceTokenStream<'token>{
    fn next<'a>(&'a mut self) -> Option<BorrowedToken<'a, 'token>>{
        self.token.text.clear();
        self.token.position = self.token.position.wrapping_add(1);
        while let Some((offset_from, c)) = self.chars.next() {
            if !c.is_ascii_whitespace() {

                let offset_to = self.search_token_end();
                self.token.text.push_str(&self.text[offset_from..offset_to]);
                self.token.offset_from = offset_from;
                self.token.offset_to = offset_to;

                return Some(self.token.borrowed());
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use crate::tokenizer::{BoxableTokenizer, BoxTokenizer};
    use super::*;

    #[test]
    fn whitespace_tokenizer() {
        let mut tokenizer = WhitespaceTokenizer{

        };
        let text = "Helloworld WorldHello";
        let mut stream = tokenizer.tokenize(text);

        while let Some(token) = stream.next() {
            println!("{:?}", token);
        }
    }

    #[test]
    fn whitespace_tokenizer_serialize() {
        let mut tokenizer = WhitespaceTokenizer{

        };
        let text = "Helloworld WorldHello";
        let mut stream = tokenizer.tokenize(text);

        while let Some(token) = stream.next() {
            println!("{:?}", token);
        }

        let result = serde_json::to_string(&tokenizer as &dyn BoxableTokenizer).unwrap();
        println!("{:#}", result);
        let tokenizer: BoxTokenizer = serde_json::from_str(&result).unwrap();
        let mut stream = tokenizer.tokenize(text);

        while let Some(token) = stream.next() {
            println!("{:?}", token);
        }
        let result = serde_json::to_string(&tokenizer).unwrap();
        println!("{:#}", result);
    }
}