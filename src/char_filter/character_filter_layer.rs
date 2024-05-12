use std::borrow::Cow;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::DeserializeOwned;
use serde::ser::SerializeSeq;
use typetag::__private::erased_serde;
use typetag::__private::erased_serde::Error;
use typetag::__private::schemars::gen::SchemaGenerator;
use typetag::__private::schemars::JsonSchema;
use typetag::__private::schemars::schema::{ArrayValidation, InstanceType, Schema, SchemaObject};
use crate::char_filter::{BoxableCharacterFilter, BoxCharacterFilter, CharacterFilter};
use crate::char_filter::CharacterFilterRegistry;


#[derive(Clone)]
pub struct CharacterFilterLayer<F: CharacterFilter, L: CharacterFilterLayers>{
    filter: F,
    upper_layer: L
}

pub struct BoxCharFilterLayer(Box<dyn BoxableLayer>);

impl BoxCharFilterLayer{
    pub fn new<L: CharacterFilterLayers>(layer: L) -> Self{
        BoxCharFilterLayer(Box::new(layer))
    }
}


#[typetag::serde(name="CharacterFilterLayers")]
impl CharacterFilter for BoxCharFilterLayer
{
    fn apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        <Self as CharacterFilterLayers>::apply_layer(self, text)
    }
}

impl JsonSchema for BoxCharFilterLayer {
    fn schema_name() -> String {
        Box::<dyn BoxableLayer>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableLayer>::json_schema(gen)
    }
}

impl Serialize for BoxCharFilterLayer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}

impl DeserializeOwned for BoxCharFilterLayer {}

impl<'de> Deserialize<'de> for BoxCharFilterLayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableLayer>::deserialize(deserializer).map(BoxCharFilterLayer)

    }
}

impl<F: CharacterFilter, L: CharacterFilterLayers> CharacterFilterLayers for CharacterFilterLayer<F,L>
{
    fn apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        let result = self.upper_layer.apply_layer(text);

        self.filter.apply(result)
    }

    default fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter)) {
        self.upper_layer.inspect_layer(fun);
    }
}


impl<F: CharacterFilter + Serialize, L: CharacterFilterLayers> CharacterFilterLayers for CharacterFilterLayer<F,L>
{
    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter)) {
        self.upper_layer.inspect_layer(fun);
        fun(&self.filter)
    }
}



pub trait BoxableLayer: Send + Sync{
    fn box_apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str>;
    fn box_inspect_layer<'a>(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter));
    fn box_clone(&self) -> BoxCharFilterLayer;
}

impl<T: CharacterFilterLayers> BoxableLayer for T {
    fn box_apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        self.apply_layer(text)
    }

    fn box_inspect_layer<'a>(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter) ) {
        self.inspect_layer(fun)
    }

    fn box_clone(&self) -> BoxCharFilterLayer {
        BoxCharFilterLayer(Box::new(self.clone()))
    }
}

impl CharacterFilterLayers for BoxCharFilterLayer {
    fn apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        self.0.box_apply_layer(text)
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter)) {
        self.0.box_inspect_layer(fun)
    }
}

impl Clone for BoxCharFilterLayer {
    fn clone(&self) -> Self {
        BoxableLayer::box_clone(self)
    }
}



pub trait CharacterFilterLayers: 'static + Clone + Send + Sync{

    fn dynamic(self) -> BoxCharFilterLayer{
        BoxCharFilterLayer::new(self)
    }
    fn apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str>;
    fn wrap_layer<F: CharacterFilter>(self, filter: F) -> CharacterFilterLayer<F,Self>{
        CharacterFilterLayer {
            filter,
            upper_layer: self
        }
    }
    fn wrap_dynamic_layer<F: CharacterFilter + Serialize>(self, filter: F) -> BoxCharFilterLayer{
        self.wrap_layer(filter).dynamic()
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter));
}

#[derive(Copy,Clone)]
pub struct BaseLevel;
impl CharacterFilterLayers for BaseLevel {
    fn apply_layer<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        text
    }

    fn inspect_layer(&self, fun: &mut dyn FnMut(&dyn BoxableCharacterFilter)) {
    }
}

impl JsonSchema for dyn BoxableLayer {
    fn schema_name() -> String {
        "CharacterFilterLayers".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                unique_items: Some(true),
                items: Some(gen.subschema_for::<BoxCharacterFilter>().into()),
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
        let mut visitor = |filter: & dyn BoxableCharacterFilter| {
            seq.serialize_element(filter);
        };
        self.box_inspect_layer(&mut visitor);
        seq.end()
    }
}

impl DeserializeOwned for Box<dyn BoxableLayer> {}

impl<'de> Deserialize<'de> for Box<dyn BoxableLayer> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let filters = Vec::<BoxCharacterFilter>::deserialize(deserializer)?;
        let mut layers = BaseLevel.dynamic();
        for filter in filters{
            layers = layers.wrap_dynamic_layer(filter);
        }

        Ok(layers.0)

    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of_val;
    use regex::Regex;
    use crate::char_filter::regex_character_filter::RegexCharacterFilter;
    use super::*;

    #[test]
    fn char_filter_layers() {
        let base = BaseLevel;
        let base = base.dynamic();
        let regex = BoxCharacterFilter(Box::new(RegexCharacterFilter{
            pattern: Regex::new(r"foo").unwrap(),
            replacement: "+".to_string()
        }));
        let a = base.wrap_layer(regex);
        let c = a.wrap_layer(RegexCharacterFilter{
            pattern: Regex::new(r"bar").unwrap(),
            replacement: "-".to_string()
        });

        let dynm = c.wrap_dynamic_layer(RegexCharacterFilter{
            pattern: Regex::new(r"r").unwrap(),
            replacement: "0".to_string()
        });
        let result = dynm.apply_layer("foobarfoorbar".into());


        println!("{}", result);

    }

    #[test]
    fn char_filter_layers_serialize() {
        let base = BaseLevel;
        let base = base.dynamic();
        let regex = RegexCharacterFilter{
            pattern: Regex::new(r"foo").unwrap(),
            replacement: "+".to_string()
        };
        let a = base.wrap_layer(regex);
        let c = a.wrap_layer(BoxCharacterFilter(Box::new(RegexCharacterFilter{
            pattern: Regex::new(r"bar").unwrap(),
            replacement: "-".to_string()
        })));

        let dynm: BoxCharFilterLayer = c.wrap_dynamic_layer(RegexCharacterFilter{
            pattern: Regex::new(r"r").unwrap(),
            replacement: "0".to_string()
        });

        let result = serde_json::to_string(&dynm).unwrap();
        println!("{:#}", result);
        let layers: BoxCharFilterLayer = serde_json::from_str(&result).unwrap();
        let stop = "Helloword";
        let result = serde_json::to_string(&layers).unwrap();
        println!("{:#}", result);



    }
}
