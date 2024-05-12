pub mod detection;
pub mod whichlang;
pub mod whatlang;
pub mod lingua;

use schemars::gen::SchemaGenerator;
use schemars::JsonSchema;
use schemars::schema::Schema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use unicode_script::UnicodeScript;
use typetag::__private::erased_serde;
use crate::char_filter::CharacterFilter;
use crate::script::Script;
use crate::language::Language;
use crate::language_detection::detection::LanguageDetection;

pub struct BoxLanguageDetector(Box<dyn BoxableLanguageDetector>);

#[typetag::serde(receiver = BoxableLanguageDetector)]
pub trait LanguageDetector: 'static + Send + Sync{

    fn detect<'detector, 'text>(&'detector self, text: &'text str) -> LanguageDetection<'detector, 'text> where Self: Sized{
        LanguageDetection::new_lazy(text, self)
    }

    fn detect_script(&self, text: &str) -> Script {
        text.chars()
            .map(|char| char.script())
            .find(|&script| script != Script::Common)
            .unwrap_or(Script::Unknown)
    }
    fn detect_lang(&self, text: &str) -> Language;


}


pub trait MultipleLanguageDetector: LanguageDetector + Clone{
    type LanguageDetections<'detector: 'text, 'text>: Iterator<Item = LanguageDetection<'detector, 'text>>;
    fn detect_multiple_languages<'detector: 'text, 'text>(
        &'detector self,
        text: &'text str,
    ) -> Self::LanguageDetections<'detector, 'text>;
}

pub trait BoxableLanguageDetector: 'static + Send + Sync + typetag::Serialize {

    #[doc(hidden)]
    fn box_detect<'detector, 'text>(&'detector self, text: &'text str) -> LanguageDetection<'detector, 'text>;

    #[doc(hidden)]
    fn box_detect_script(&self, text: &str) -> Script;

    #[doc(hidden)]
    fn box_detect_lang(&self, text: &str) -> Language;

    #[doc(hidden)]
    fn box_detect_multiple_languages<'detector: 'text, 'text>(
        &'detector self,
        text: &'text str,
    ) -> Box<dyn Iterator<Item = LanguageDetection<'detector, 'text>> + 'text>;

    #[doc(hidden)]
    fn box_clone(&self) -> Box<dyn BoxableLanguageDetector>;

    fn type_name<'a>(&self) -> &'static str;
}

#[doc(hidden)]
impl<T: MultipleLanguageDetector + Serialize> BoxableLanguageDetector for T {

    fn box_detect<'detector, 'text>(&'detector self, text: &'text str) -> LanguageDetection<'detector, 'text> {
        <T as LanguageDetector>::detect(self, text)
    }

    fn box_detect_script(&self, text: &str) -> Script {
        <T as LanguageDetector>::detect_script(self, text)
    }

    fn box_detect_lang(&self, text: &str) -> Language {
        <T as LanguageDetector>::detect_lang(self, text)
    }

    fn box_detect_multiple_languages<'detector: 'text, 'text>(
        &'detector self,
        text: &'text str,
    ) -> Box<dyn Iterator<Item = LanguageDetection<'detector, 'text>> + 'text> {
        Box::new(<T as MultipleLanguageDetector>::detect_multiple_languages(self, text))
    }

    fn box_clone(&self) -> Box<dyn BoxableLanguageDetector>{
        Box::new(self.clone())
    }

    fn type_name<'a>(&self) -> &'static str {
        LanguageDetector::type_name(self)
    }


}

impl LanguageDetector for BoxLanguageDetector {
    fn detect<'detector, 'str>(&'detector self, text: &'str str) -> LanguageDetection<'detector, 'str> {
        <Self as BoxableLanguageDetector>::box_detect(self, text)
    }

    fn detect_script(&self, text: &str) -> Script {
        <Self as BoxableLanguageDetector>::box_detect_script(self, text)
    }

    fn detect_lang(&self, text: &str) -> Language {
        <Self as BoxableLanguageDetector>::box_detect_lang(self, text)
    }

    fn type_name(&self) -> &'static str {
        &self.0.type_name()
    }
}

impl MultipleLanguageDetector for BoxLanguageDetector {
    type LanguageDetections<'detector: 'text, 'text> = Box<dyn Iterator<Item=LanguageDetection<'detector, 'text>> + 'text>;

    fn detect_multiple_languages<'detector: 'str, 'str>(
        &'detector self,
        text: &'str str,
    ) -> Self::LanguageDetections<'detector, 'str> {
        self.0.box_detect_multiple_languages(text, )
    }
}

impl Clone for BoxLanguageDetector {
    fn clone(&self) -> Self {
        BoxLanguageDetector(self.0.box_clone())
    }
}


impl Serialize for BoxLanguageDetector {
    fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        erased_serde::serialize(&*self.0, serializer)
    }
}



impl<'de> Deserialize<'de> for BoxLanguageDetector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Box::<dyn BoxableLanguageDetector>::deserialize(deserializer).map(BoxLanguageDetector)
    }
}

impl JsonSchema for BoxLanguageDetector {
    fn schema_name() -> String {
        Box::<dyn BoxableLanguageDetector>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        Box::<dyn BoxableLanguageDetector>::json_schema(gen)
    }
}


