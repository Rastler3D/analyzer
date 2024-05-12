use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::{ArrayValidation, InstanceType, Schema, SchemaObject};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeSeq;
use typetag::__private::erased_serde;
use crate::inline_dyn::{Dynamic, DynamicFrom};
use crate::token_filter::{BoxableTokenFilter, BoxTokenFilter, TokenFilter};
use crate::tokenizer::token_stream::TokenStream;
use crate::token_filter::TokenFilterRegistry;

#[derive(Clone)]
pub struct TokenFilterLayer<F: TokenFilter, L: TokenFilterLayers> {
    filter: F,
    upper_layer: L
}

pub struct BoxTokenFilterLayer(Box<dyn BoxableLayer>);

#[typetag::serde(name="TokenFilterLayers")]
impl TokenFilter for BoxTokenFilterLayer {
    type TokenStream<'token, T: TokenStream<'token> + 'token> = <Self as TokenFilterLayers>::TokenStream<'token, T>;

    fn apply<'token, T: TokenStream<'token> + 'token>(&'token self, token_stream: T) -> Self::TokenStream<'token, T> {
        <Self as TokenFilterLayers>::apply_layer(self, token_stream)
    }
}

impl JsonSchema for BoxTokenFilterLayer {
    fn schema_name() -> String {
        Box::<dyn BoxableLayer>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableLayer>::json_schema(gen)
    }
}

impl Serialize for BoxTokenFilterLayer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}

impl DeserializeOwned for BoxTokenFilterLayer {}

impl<'de> Deserialize<'de> for BoxTokenFilterLayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableLayer>::deserialize(deserializer).map(BoxTokenFilterLayer)
    }
}

pub trait TokenFilterLayers: 'static + Clone + Send + Sync
{
    type TokenStream<'token, T: TokenStream<'token>  + 'token>: TokenStream<'token>  + 'token;
    fn dynamic(self) -> BoxTokenFilterLayer{
        BoxTokenFilterLayer::new(self)
    }
    fn apply_layer<'token, S: TokenStream<'token> + 'token>(&'token self, stream: S) -> Self::TokenStream<'token, S>;
    fn wrap_layer<F: TokenFilter>(self, filter: F) -> TokenFilterLayer<F,Self>{
        TokenFilterLayer {
            filter,
            upper_layer: self
        }
    }
    fn wrap_dynamic_layer<F: TokenFilter + Serialize>(self, filter: F) -> BoxTokenFilterLayer{
        self.wrap_layer(filter).dynamic()
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter));


}

impl BoxTokenFilterLayer{
    pub fn new<L: TokenFilterLayers>(layer: L) -> Self{
        BoxTokenFilterLayer(Box::new(layer))
    }
}

pub trait BoxableLayer: Send + Sync{
    fn box_apply_layer<'token>(&'token self, stream: Dynamic<dyn TokenStream<'token>  + 'token>) -> Dynamic<dyn TokenStream<'token> + 'token>;
    fn box_inspect_layer<'a>(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter) );


    fn box_clone(&self) -> BoxTokenFilterLayer;
}

impl<T: TokenFilterLayers> BoxableLayer for T {
    fn box_apply_layer<'token>(&'token self, stream: Dynamic<dyn TokenStream<'token>  + 'token>) -> Dynamic<dyn TokenStream<'token> + 'token>{
        DynamicFrom::from(self.apply_layer(stream))
    }

    fn box_inspect_layer<'a>(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter) ) {
        self.inspect_layer(fun)
    }

    fn box_clone(&self) -> BoxTokenFilterLayer {
        BoxTokenFilterLayer(Box::new(self.clone()))
    }
}

impl TokenFilterLayers for BoxTokenFilterLayer {
    type TokenStream<'token, T: TokenStream<'token> + 'token> = Dynamic<dyn TokenStream<'token> + 'token>;

    fn apply_layer<'token, S: TokenStream<'token>  + 'token>(&'token self, stream: S) -> Self::TokenStream<'token, S> {
        self.0.box_apply_layer(Dynamic::new(stream))
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter)) {
        self.0.box_inspect_layer(fun)
    }
}

impl Clone for BoxTokenFilterLayer {
    fn clone(&self) -> Self {
        BoxableLayer::box_clone(self)
    }
}


impl<F: TokenFilter + Serialize, L: TokenFilterLayers> TokenFilterLayers for TokenFilterLayer<F,L>
{
    type TokenStream<'token, T: TokenStream<'token>  + 'token> = F::TokenStream<'token, L::TokenStream<'token, T>>;

    fn apply_layer<'token, S: TokenStream<'token> + 'token>(&'token self, stream: S) -> Self::TokenStream<'token, S> {
        let result = self.upper_layer.apply_layer(stream);

        self.filter.apply(result)
    }

    default fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter)) {
        self.upper_layer.inspect_layer(fun);
        fun(&self.filter)
    }
}



#[derive(Copy, Clone, Debug)]
pub struct BaseLevel;

impl TokenFilterLayers for BaseLevel {
    type TokenStream<'token, T: TokenStream<'token>  + 'token> = T;

    fn apply_layer<'token, S: TokenStream<'token> + 'token>(&'token self, stream: S) -> Self::TokenStream<'token, S> {
        stream
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableTokenFilter)) {
    }
}

impl JsonSchema for dyn BoxableLayer {
    fn schema_name() -> String {
        "TokenFilterLayers".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                unique_items: Some(true),
                items: Some(gen.subschema_for::<BoxTokenFilter>().into()),
                ..Default::default()
            })),
            ..Default::default()
        }
            .into()
    }
}

impl Serialize for dyn BoxableLayer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(None)?;
        let mut visitor = |filter: & dyn BoxableTokenFilter| {
            seq.serialize_element(filter);
        };
        self.box_inspect_layer(&mut visitor);
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Box<dyn BoxableLayer> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let filters = Vec::<BoxTokenFilter>::deserialize(deserializer)?;
        let mut layers = BaseLevel.dynamic();
        for filter in filters{
            layers = layers.wrap_dynamic_layer(filter);
        }

        Ok(layers.0)

    }
}



#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use crate::token_filter::lower_case::LowerCaseFilter;
    use crate::tokenizer::Tokenizer;
    use crate::tokenizer::whitespace_tokenizer::WhitespaceTokenizer;
    use super::*;

    #[test]
    fn token_filter_layers() {
        let base = BaseLevel;
        let base = base;
        let a = base.wrap_layer(LowerCaseFilter {

        });
        let c = a.wrap_layer(LowerCaseFilter {

        });

        let mut dynm = c.wrap_dynamic_layer(LowerCaseFilter {

        });

        let mut tokenizer = WhitespaceTokenizer {

        };
        let text = "Helloworld WorldHello";
        let mut stream = tokenizer.tokenize(text);


        let mut result = dynm.apply_layer(stream);

        while let Some(token) = result.next() {
            println!("{:?}", token);
        }
        //println!("{}", result);

    }

    #[test]
    fn token_filter_layers_serialize() {
        let base = BaseLevel;
        let base = base;
        let a = base.wrap_layer(LowerCaseFilter {

        });
        let c = a.wrap_layer(LowerCaseFilter {

        });

        let mut dynm = c.wrap_dynamic_layer(LowerCaseFilter {

        });

        let mut tokenizer = WhitespaceTokenizer {

        };
        let text = "Helloworld WorldHello";
        let mut stream = tokenizer.tokenize(text);


        let mut result = dynm.apply_layer(stream);

        while let Some(token) = result.next() {
            println!("{:?}", token);
        }
        let token_filter = LowerCaseFilter {
        };
        let result = serde_json::to_string(&token_filter as &dyn BoxableTokenFilter).unwrap();
        println!("{:#}", result);
        let layers: BoxTokenFilter = serde_json::from_str(&result).unwrap();
        let result = serde_json::to_string(&dynm).unwrap();
        println!("{:#}", result);
        let layers: BoxTokenFilterLayer = serde_json::from_str(&result).unwrap();
        let result = serde_json::to_string(&layers).unwrap();
        println!("{:#}", result);

        let schema = schema_for!(BoxTokenFilterLayer);
        let schema = serde_json::to_string_pretty(&schema).unwrap();;
        println!("{}", schema);

    }
}
