use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use unicode_script::{Script, UnicodeScript};
use crate::language::Language;
use crate::language_detection::{LanguageDetector, MultipleLanguageDetector};
use crate::language_detection::detection::LanguageDetection;
use crate::language_detection::LanguageDetectorRegistry;

#[derive(Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WhichLangDetector{}

#[typetag::serde]
impl LanguageDetector for WhichLangDetector {
    fn detect_lang(&self, text: &str) -> Language {
        whichlang::detect_language(text).into()
    }
}

impl MultipleLanguageDetector for WhichLangDetector {
    type LanguageDetections<'detector: 'text, 'text> = impl Iterator<Item = LanguageDetection<'detector,'text>>;

    fn detect_multiple_languages<'detector: 'text, 'text>(&'detector self, text: &'text str) -> Self::LanguageDetections<'detector, 'text> {
        let mut chars = text.char_indices().peekable();
        let (mut prev_index, mut prev_script) = chars
            .peek()
            .map(|&(index, char)| (index, char.script()))
            .unwrap_or_default();
        std::iter::from_fn(move || {
            while let Some(_) = chars.next() {
                let Some(&(index, char)) = chars.peek() else {
                    return text.get(prev_index..);
                };
                let script = char.script();

                if script != Script::Common && script != prev_script {
                    if prev_script != Script::Common {
                        let text = text.get(prev_index..index);
                        prev_script = script;
                        prev_index = index;
                        return text;
                    };
                    prev_script = script;
                }
            }
            return None;
        })
            .map(move |text| self.detect(text))
    }
}

#[cfg(test)]
mod tests {
    use crate::language_detection::MultipleLanguageDetector;
    use crate::language_detection::whichlang::WhichLangDetector;

    #[test]
    fn whichlang() {
        let text = "Οι θερμοκρασίες είναι σπάνια υπερβολικές στις παραθαλάσσιες περιοχές. 제119조 ① 대한민국의 경제질서는 개인과 기업의 경제상의 자유와 창의를 존중함을 기본으로 한다. La ville avait d'abord été nommée My name is Aleksey, What is your name?";

        let detector = WhichLangDetector{};
        let lang = detector.detect_multiple_languages(text);
        for each in lang {
            println!("{:?}", each);
        }

    }
}