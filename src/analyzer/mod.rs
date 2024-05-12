use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typetag::__private::erased_serde;
use crate::inline_dyn::{Dynamic, DynamicFrom};
use crate::tokenizer::token_stream::TokenStream;
use crate::tokenizer::{BoxableTokenizer, BoxTokenizer, Tokenizer};


pub mod text_analyzer;

pub struct BoxAnalyzer(Box<dyn BoxableAnalyzer>);

impl BoxAnalyzer{
    pub fn new<T: BoxableAnalyzer>(analyzer: T) -> Self{
        BoxAnalyzer(Box::new(analyzer))
    }
}

#[typetag::serde(receiver = BoxableAnalyzer)]
pub trait Analyzer: 'static + Send + Sync + Clone  {
    type TokenStream<'token>: TokenStream<'token> + 'token
    where
        Self: 'token;
    fn analyze<'token>(&'token self, text: &'token str) -> Self::TokenStream<'token>;
}

impl Analyzer for BoxAnalyzer {
    type TokenStream<'token> = Dynamic<dyn TokenStream<'token> + 'token>;

    fn analyze<'token>(&'token self, text: &'token str) -> Self::TokenStream<'token> {
        self.0.box_analyze(text.into())
    }
    fn type_name<'a>(&self) -> &'static str {
        &self.0.type_name()
    }
}

impl Clone for BoxAnalyzer {
    fn clone(&self) -> Self {
        BoxAnalyzer(self.0.box_clone())
    }
}

pub trait BoxableAnalyzer: 'static + Send + Sync + typetag::Serialize  {
    fn box_analyze<'token>(&'token self, text: &'token str) -> Dynamic<dyn TokenStream<'token> + 'token>;
    fn box_clone(&self) -> Box<dyn BoxableAnalyzer>;
    fn type_name<'a>(&self) -> &'static str;
}

impl<T: Analyzer + Serialize> BoxableAnalyzer for T
{
    fn box_analyze<'token>(&'token self, text: &'token str) -> Dynamic<dyn TokenStream<'token> + 'token> {
        DynamicFrom::from(self.analyze(text))
    }

    fn box_clone(&self) -> Box<dyn BoxableAnalyzer> {
        Box::new(self.clone())
    }

    fn type_name<'a>(&self) -> &'static str {
        Analyzer::type_name(self)
    }
}

impl Serialize for BoxAnalyzer {
    fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.0.serialize(serializer)
    }
}




impl<'de> Deserialize<'de> for BoxAnalyzer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableAnalyzer>::deserialize(deserializer).map(BoxAnalyzer)
    }
}

impl JsonSchema for BoxAnalyzer {
    fn schema_name() -> String {
        Box::<dyn BoxableAnalyzer>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableAnalyzer>::json_schema(gen)
    }
}
