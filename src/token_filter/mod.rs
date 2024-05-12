use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typetag::__private::erased_serde;
use crate::inline_dyn::{Dynamic, DynamicFrom};
use crate::tokenizer::token_stream::TokenStream;

pub mod token_filter_layer;
pub mod lower_case;

pub struct BoxTokenFilter(Box<dyn BoxableTokenFilter>);

#[typetag::serde(receiver = BoxableTokenFilter)]
pub trait TokenFilter: 'static + Send + Sync + Clone {
    type TokenStream<'token, T: TokenStream<'token> + 'token>: TokenStream<'token> + 'token;

    fn apply<'token, T: TokenStream<'token> + 'token>(&'token self, token_stream: T) -> Self::TokenStream<'token, T>;
}


pub trait BoxableTokenFilter: 'static + Send + Sync + typetag::Serialize {
    /// Clone this tokenizer.
    fn box_clone(&self) -> Box<dyn BoxableTokenFilter>;
    fn box_apply<'token>(&'token self, token_stream: Dynamic<dyn TokenStream<'token>  + 'token>) -> Dynamic<dyn TokenStream<'token>  + 'token>;

    fn type_name<'a>(&self) -> &'static str;
}

impl<T: TokenFilter + Serialize> BoxableTokenFilter for T{
    fn box_clone(&self) -> Box<dyn BoxableTokenFilter> {
        Box::new(self.clone())
    }

    fn box_apply<'token>(&'token self, token_stream: Dynamic<dyn TokenStream<'token>  + 'token>) -> Dynamic<dyn TokenStream<'token>  + 'token> {
        DynamicFrom::from(self.apply(token_stream))
    }

    fn type_name<'a>(&self) -> &'static str {
        TokenFilter::type_name(self)
    }


}

impl TokenFilter for BoxTokenFilter {
    type TokenStream<'token, T: TokenStream<'token> + 'token> = Dynamic<dyn TokenStream<'token>  + 'token>;

    fn apply<'token, T: TokenStream<'token> + 'token>(&'token self, token_stream: T) -> Self::TokenStream<'token, T> {
        self.0.box_apply(DynamicFrom::from(token_stream))
    }
    fn type_name(&self) -> &'static str {
        &self.0.type_name()
    }
}

impl Clone for BoxTokenFilter {
    fn clone(&self) -> Self {
        BoxTokenFilter(self.0.box_clone())
    }
}

impl Serialize for BoxTokenFilter {
    fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}



impl<'de> Deserialize<'de> for BoxTokenFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableTokenFilter>::deserialize(deserializer).map(BoxTokenFilter)
    }
}

impl JsonSchema for BoxTokenFilter {
    fn schema_name() -> String {
        Box::<dyn BoxableTokenFilter>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableTokenFilter>::json_schema(gen)
    }
}
