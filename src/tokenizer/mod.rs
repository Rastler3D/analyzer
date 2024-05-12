use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typetag::__private::erased_serde;
use crate::inline_dyn::Dynamic;
use crate::language_detection::detection::LanguageDetection;
use crate::tokenizer::token_stream::TokenStream;
pub mod token_stream;
pub mod whitespace_tokenizer;

pub struct BoxTokenizer(Box<dyn BoxableTokenizer>);

#[typetag::serde(receiver = BoxableTokenizer)]
pub trait Tokenizer: 'static + Clone + Send + Sync {
    type TokenStream<'token>: TokenStream<'token>;

    fn tokenize<'token>(&'token self, text: impl Into<LanguageDetection<'token,'token>>) -> Self::TokenStream<'token>;
}

impl Tokenizer for BoxTokenizer {
    type TokenStream<'token> = Dynamic<dyn TokenStream<'token> + 'token>;

    fn tokenize<'token>(&'token self, text: impl Into<LanguageDetection<'token,'token>>) -> Self::TokenStream<'token>{
        self.0.box_tokenize(text.into())
    }

    fn type_name(&self) -> &'static str {
        &self.0.type_name()
    }
}

impl Clone for BoxTokenizer {
    fn clone(&self) -> Self {
        BoxTokenizer(self.0.box_clone())
    }
}

pub trait BoxableTokenizer: 'static + Send + Sync + typetag::Serialize {
    fn box_tokenize<'token>(&'token self, text: LanguageDetection<'token,'token>) -> Dynamic<dyn TokenStream + 'token>;
    fn box_clone(&self) -> Box<dyn BoxableTokenizer>;
    fn type_name<'a>(&self) -> &'static str;

}

impl<T: Tokenizer + Serialize> BoxableTokenizer for T
{
    fn box_tokenize<'token>(&'token self, text: LanguageDetection<'token,'token>) -> Dynamic<dyn TokenStream + 'token>{
        let token_stream = self.tokenize(text);

        Dynamic::new(token_stream)

    }
    fn box_clone(&self) -> Box<dyn BoxableTokenizer> {
        Box::new(self.clone())
    }

    fn type_name<'a>(&self) -> &'static str {
        Tokenizer::type_name(self)
    }
}


impl Serialize for BoxTokenizer {
    fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}



impl<'de> Deserialize<'de> for BoxTokenizer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableTokenizer>::deserialize(deserializer).map(BoxTokenizer)
    }
}

impl JsonSchema for BoxTokenizer {
    fn schema_name() -> String {
        Box::<dyn BoxableTokenizer>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableTokenizer>::json_schema(gen)
    }
}