pub mod character_filter_layer;
pub mod regex_character_filter;

use std::borrow::Cow;
use std::ops::CoerceUnsized;
use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typetag::__private::erased_serde;


pub struct BoxCharacterFilter(Box<dyn BoxableCharacterFilter>);
#[typetag::serde(receiver = BoxableCharacterFilter)]
pub trait CharacterFilter: 'static + Send + Sync + Clone{
    fn apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a,str>;
}


pub trait BoxableCharacterFilter: 'static + Send + Sync + typetag::Serialize{
    /// Clone this tokenizer.
    fn box_clone(&self) -> Box<dyn BoxableCharacterFilter>;
    fn box_apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str>;
    fn type_name<'a>(&self) -> &'static str;
}

impl BoxableCharacterFilter for BoxCharacterFilter {
    fn box_clone(&self) -> Box<dyn BoxableCharacterFilter> {
        self.clone().0
    }
}

impl<T: CharacterFilter + Serialize> BoxableCharacterFilter for T{
    default fn box_clone(&self) -> Box<dyn BoxableCharacterFilter> {
        Box::new(self.clone())
    }

    fn box_apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        self.apply(text)
    }

    fn type_name<'a>(&self) -> &'static str {
        CharacterFilter::type_name(self)
    }
}

impl CharacterFilter for BoxCharacterFilter {
    fn apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        self.0.box_apply(text)
    }

    fn type_name(&self) -> &'static str {
        &self.0.type_name()
    }
}

impl Clone for BoxCharacterFilter {
    fn clone(&self) -> Self {
        BoxCharacterFilter(self.0.box_clone())
    }
}


impl Serialize for BoxCharacterFilter {
    fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}



impl<'de> Deserialize<'de> for BoxCharacterFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableCharacterFilter>::deserialize(deserializer).map(BoxCharacterFilter)
    }
}

impl JsonSchema for BoxCharacterFilter {
    fn schema_name() -> String {
        Box::<dyn BoxableCharacterFilter>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableCharacterFilter>::json_schema(gen)
    }
}


