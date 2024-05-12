use crate::language::Language;
use crate::language_detection::detection::LanguageDetection;
use crate::language_detection::{LanguageDetector, MultipleLanguageDetector};
use crate::language_detection::LanguageDetectorRegistry;
use std::sync::Arc;
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct LinguaDetector {
    #[serde(skip, default = "build_lingua")]
    inner: Arc<lingua::LanguageDetector>,
}

impl LinguaDetector {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(
                lingua::LanguageDetectorBuilder::from_all_languages()
                    .with_preloaded_language_models()
                    .build(),
            ),
        }
    }
}



#[typetag::serde]
impl LanguageDetector for LinguaDetector {
    fn detect<'detector, 'str>(&'detector self, text: &'str str) -> LanguageDetection<'detector, 'str> {
        LanguageDetection::new_lazy(text, self)
    }

    fn detect_lang(&self, text: &str) -> Language {
        self.inner
            .detect_language_of(text)
            .map_or(Language::Unknown, Into::into)
    }
}

impl MultipleLanguageDetector for LinguaDetector {

    type LanguageDetections<'detector: 'text, 'text> = impl Iterator<Item = LanguageDetection<'detector,'text>>;

    fn detect_multiple_languages<'detector: 'text, 'text>(
        &'detector self,
        text: &'text str,
    ) -> Self::LanguageDetections<'detector, 'text> {
        let detection = self.inner.detect_multiple_languages_of(text);
        let len = detection.len();
        detection.into_iter().map(move |x| {
            let (start, end) = if len == 1 {
                (0, text.len())
            } else {
                (x.start_index(), x.end_index())
            };
            let text = &text[start..end];

            LanguageDetection::new_init(
                text,
                Language::from(x.language()),
                self.detect_script(text),
            )
        })
    }
}


#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use crate::language_detection::lingua::LinguaDetector;
    use crate::language_detection::{BoxableLanguageDetector, BoxLanguageDetector, MultipleLanguageDetector};
    use crate::language_detection::whatlang::WhatLangDetector;

    #[test]
    fn lingua() {
        let text = "Οι θερμοκρασίες είναι σπάνια υπερβολικές στις παραθαλάσσιες περιοχές. 제119조 ① 대한민국의 경제질서는 개인과 기업의 경제상의 자유와 창의를 존중함을 기본으로 한다. La ville avait d'abord été nommée My name is Aleksey, What is your name?";

        let detector = LinguaDetector::new();
        let lang = detector.detect_multiple_languages(text);
        for each in lang {
            println!("{:?}", each);
        }
    }

    #[test]
    fn serialize() {
        let text = "Οι θερμοκρασίες είναι σπάνια υπερβολικές στις παραθαλάσσιες περιοχές. 제119조 ① 대한민국의 경제질서는 개인과 기업의 경제상의 자유와 창의를 존중함을 기본으로 한다. La ville avait d'abord été nommée My name is Aleksey, What is your name?";

        let detector = LinguaDetector::new();;
        let lang = detector.detect_multiple_languages(text);
        for each in lang {
            println!("{:?}", each);
        }

        let serialized = serde_json::to_string(&detector as &dyn BoxableLanguageDetector).unwrap();
        println!("{}", serialized);
        let deserialized: BoxLanguageDetector = serde_json::from_str(&serialized).unwrap();
        let lang = deserialized.detect_multiple_languages(text);
        for each in lang {
            println!("{:?}", each);
        }

        let schema = schema_for!(BoxLanguageDetector);
        let schema = serde_json::to_string_pretty(&schema).unwrap();;
        println!("{}", schema);



    }
}


fn build_lingua() -> Arc<lingua::LanguageDetector>{
    Arc::new(
        lingua::LanguageDetectorBuilder::from_all_languages()
            .with_preloaded_language_models()
            .build(),
    )
}

impl From<()> for LinguaDetector {
    fn from(value: ()) -> Self {
        LinguaDetector{
            inner: build_lingua()
        }
    }
}

impl Into<()> for LinguaDetector {
    fn into(self) -> () {
        ()
    }
}