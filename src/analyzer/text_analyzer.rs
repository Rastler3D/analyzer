use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use owning_ref::OwningHandle;
use polonius_the_crab::{polonius, polonius_return};
use schemars::JsonSchema;
use serde::{Serialize as SerdeSerialize, Serializer};
use serde::ser::SerializeStruct;
use serde_derive::{Deserialize, Serialize};
use crate::char_filter::character_filter_layer::{BoxCharFilterLayer, CharacterFilterLayers};
use crate::{language_detection, tokenizer};
use crate::analyzer::Analyzer;
use crate::char_filter::{BoxCharacterFilter, CharacterFilter};
use crate::language_detection::detection::LanguageDetection;
use crate::language_detection::{BoxableLanguageDetector, BoxLanguageDetector, MultipleLanguageDetector};
use crate::token::BorrowedToken;
use crate::token_filter::token_filter_layer::{BoxTokenFilterLayer, TokenFilterLayers};
use crate::token_filter::{BoxTokenFilter, TokenFilter};
use crate::tokenizer::{BoxableTokenizer, BoxTokenizer};
use crate::tokenizer::token_stream::TokenStream;
use crate::analyzer::AnalyzerRegistry;

#[derive(Clone, Deserialize, JsonSchema)]
pub struct TextAnalyzer<
    CharacterFilters: CharacterFilterLayers,
    LanguageDetector: language_detection::LanguageDetector,
    Tokenizer: tokenizer::Tokenizer,
    TokenFilters: TokenFilterLayers,
> {
    pub character_filters: CharacterFilters,
    pub language_detector: LanguageDetector,
    pub tokenizer: Tokenizer,
    pub token_filters: TokenFilters,
}

#[typetag::serde(name = "CustomTextAnalyzer")]
impl Analyzer for TextAnalyzer<BoxCharFilterLayer, BoxLanguageDetector, BoxTokenizer, BoxTokenFilterLayer> {

}

impl<
    CharacterFilters: CharacterFilterLayers,
    LanguageDetector: MultipleLanguageDetector,
    Tokenizer: tokenizer::Tokenizer,
    TokenFilters: TokenFilterLayers,
> Analyzer for TextAnalyzer<CharacterFilters, LanguageDetector, Tokenizer, TokenFilters>
{
    type TokenStream<'token> =
    AnalyzerStream<'token, CharacterFilters, LanguageDetector, Tokenizer, TokenFilters>;

    fn analyze<'a>(&'a self, text: &'a str) -> Self::TokenStream<'a> {
        AnalyzerStream {
            text: text,
            token_filters: &self.token_filters,
            character_filters: &self.character_filters,
            language_detector: &self.language_detector,
            tokenizer: &self.tokenizer,
            inner_stream: None,
        }
    }

    default fn type_name(&self) -> &'static str {
        "CustomTextAnalyzer"
    }
}

pub struct AnalyzerStream<
    'stream,
    CharacterFilters: CharacterFilterLayers,
    LanguageDetector: MultipleLanguageDetector,
    Tokenizer: tokenizer::Tokenizer,
    TokenFilters: TokenFilterLayers,
> {
    text: &'stream str,
    character_filters: &'stream CharacterFilters,
    language_detector: &'stream LanguageDetector,
    inner_stream: Option<
        OwningHandle<
            Cow<'stream, str>,
            TokenizerStream<
                'stream,
                LanguageDetector::LanguageDetections<'stream, 'stream>,
                TokenFilters::TokenStream<'stream, Tokenizer::TokenStream<'stream>>,
            >,
        >,
    >,
    tokenizer: &'stream Tokenizer,
    token_filters: &'stream TokenFilters,
}

impl<
    'stream,
    CharacterFilters: CharacterFilterLayers,
    LanguageDetector: MultipleLanguageDetector,
    Tokenizer: tokenizer::Tokenizer,
    TokenFilters: TokenFilterLayers,
> TokenStream<'stream>
for AnalyzerStream<'stream, CharacterFilters, LanguageDetector, Tokenizer, TokenFilters>
{
    fn next<'this>(&'this mut self) -> Option<BorrowedToken<'this, 'stream>> {
        loop {
            match self.inner_stream {
                None => {
                    let text = self.character_filters.apply_layer(Cow::Borrowed(self.text));
                    let owning_ref = OwningHandle::new_with_fn(text, |text| TokenizerStream {
                        language_detections: self
                            .language_detector
                            .detect_multiple_languages(unsafe { &*text }),
                        token_stream: None,
                    });
                    self.inner_stream = Some(owning_ref);
                }
                Some(ref mut analyzer) => {
                    let mut analyzer = analyzer;
                    polonius!(|analyzer| -> Option<BorrowedToken<'polonius, 'stream>> {
                        if let Some(token) =
                            analyzer.token_stream.as_mut().and_then(TokenStream::next)
                        {
                            polonius_return!(Some(token));
                        }
                    });
                    match analyzer.language_detections.next() {
                        None => return None,
                        Some(detection) => {
                            let token_stream = self.tokenizer.tokenize(detection);
                            let token_stream = self.token_filters.apply_layer(token_stream);
                            analyzer.token_stream = Some(token_stream);

                            return analyzer.token_stream.as_mut().and_then(TokenStream::next);
                        }
                    }
                }
            }
        }
    }
}

impl<
    'analyzer,
    LanguageDetections: Iterator<Item = LanguageDetection<'analyzer, 'analyzer>>,
    TokenStream: tokenizer::token_stream::TokenStream<'analyzer>,
> DerefMut for TokenizerStream<'analyzer, LanguageDetections, TokenStream>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl<
    'analyzer,
    LanguageDetections: Iterator<Item = LanguageDetection<'analyzer, 'analyzer>>,
    TokenStream: tokenizer::token_stream::TokenStream<'analyzer>,
> Deref for TokenizerStream<'analyzer, LanguageDetections, TokenStream>
{
    type Target = TokenizerStream<'analyzer, LanguageDetections, TokenStream>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

struct TokenizerStream<
    'analyzer,
    LanguageDetections: Iterator<Item = LanguageDetection<'analyzer, 'analyzer>>,
    TokenStream: tokenizer::token_stream::TokenStream<'analyzer>,
> {
    language_detections: LanguageDetections,
    token_stream: Option<TokenStream>,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use schemars::{Map, schema_for};
    use crate::analyzer::{BoxableAnalyzer, BoxAnalyzer};
    use super::*;
    use crate::inline_dyn::Dynamic;
    use crate::language_detection::whichlang::WhichLangDetector;
    use crate::token_filter::lower_case::LowerCaseFilter;
    use crate::token_filter::token_filter_layer::BaseLevel;

    #[test]
    fn analyzer() {
        let base = BaseLevel;
        let base = base;
        let a = base.wrap_layer(LowerCaseFilter {});
        let c = a.wrap_layer(LowerCaseFilter {});

        let mut dynm = c.wrap_layer(LowerCaseFilter {});

        let mut tokenizer = crate::tokenizer::whitespace_tokenizer::WhitespaceTokenizer {};
        let text = "Helloworld WorldHello";

        let mut analyzer = TextAnalyzer {
            character_filters: crate::char_filter::character_filter_layer::BaseLevel,
            language_detector: WhichLangDetector{},
            tokenizer: tokenizer,
            token_filters: dynm,
        };

        let mut stream = analyzer.analyze(text);
        let mut stream = test_move(stream);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);
    }

    fn test_move<'a>(mut stream: impl TokenStream<'a>) -> impl TokenStream<'a>{
        println!("{:?}", stream.next());
        stream
    }

    #[test]
    fn analyzer_serialize() {
        let base = BaseLevel;
        let base = base;
        let a = base.wrap_layer(LowerCaseFilter {});
        let c = a.wrap_layer(LowerCaseFilter {});

        let mut dynm = c.wrap_layer(LowerCaseFilter {});

        let mut tokenizer = crate::tokenizer::whitespace_tokenizer::WhitespaceTokenizer {};
        let text = "Helloworld WorldHello";

        let mut analyzer = TextAnalyzer {
            character_filters: crate::char_filter::character_filter_layer::BaseLevel,
            language_detector: WhichLangDetector{},
            tokenizer: tokenizer,
            token_filters: dynm,
        };

        let schema = schema_for!(BTreeMap<String,BoxAnalyzer>);
        let schema = serde_json::to_string(&schema).unwrap();
        println!("{}", schema);

        let mut stream = analyzer.analyze(text);
        let mut stream = test_move(stream);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);

        let result = serde_json::to_string(&analyzer as &dyn BoxableAnalyzer).unwrap();
        println!("{:#}", result);
        let result = serde_json::to_string(&BoxAnalyzer::new(analyzer.clone())).unwrap();
        println!("{:#}", result);
        let layers: BoxAnalyzer = serde_json::from_str(&result).unwrap();
        let mut stream = layers.analyze(text);
        let mut stream = test_move(stream);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);
        let token = stream.next();
        println!("{:?}", token);

        let result = serde_json::to_string(&layers).unwrap();
        println!("{:#}", result);
    }
}


impl<
    CharacterFilters: CharacterFilterLayers + crate::char_filter::character_filter_layer::BoxableLayer,
    LanguageDetector: MultipleLanguageDetector + BoxableLanguageDetector,
    Tokenizer: crate::tokenizer::Tokenizer + BoxableTokenizer,
    TokenFilters: TokenFilterLayers + crate::token_filter::token_filter_layer::BoxableLayer,
> SerdeSerialize for TextAnalyzer<CharacterFilters, LanguageDetector, Tokenizer, TokenFilters>
{
    default fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut struct_serializer = serializer.serialize_struct("TextAnalyzer", 4)?;
        struct_serializer.serialize_field("character_filters", &self.character_filters as &dyn crate::char_filter::character_filter_layer::BoxableLayer)?;
        struct_serializer.serialize_field("language_detector", &self.language_detector as &dyn BoxableLanguageDetector)?;
        struct_serializer.serialize_field("tokenizer", &self.tokenizer as &dyn BoxableTokenizer)?;
        struct_serializer.serialize_field("token_filters", &self.token_filters as &dyn crate::token_filter::token_filter_layer::BoxableLayer)?;
        struct_serializer.end()
    }
}