use crate::language::Language;
use crate::language_detection::LanguageDetector;
use crate::lazy::Lazy;
use crate::script::Script;
use std::fmt::{Debug, Formatter};

pub struct LanguageDetection<'detector:'text, 'text> {
    pub text: &'text str,
    pub script: Lazy<Script, DetectScript<'detector, 'text>>,
    pub language: Lazy<Language, DetectLanguage<'detector, 'text>>,
}

impl<'detector, 'text> LanguageDetection<'detector, 'text> {
    pub fn new_lazy(
        text: &'text str,
        detector: &'detector dyn LanguageDetector
    ) -> LanguageDetection<'detector, 'text> {
        LanguageDetection {
            text,
            language: Lazy::new(DetectLanguage{ text, detector }),
            script: Lazy::new(DetectScript{ text, detector }),
        }
    }

    pub fn new_init(
        text: &'text str,
        language: Language,
        script: Script
    ) -> LanguageDetection<'detector, 'text> {
        LanguageDetection {
            text,
            language: Lazy::init(language),
            script: Lazy::init(script),
        }
    }

    pub fn text(&self) -> &'text str{
        self.text
    }

    fn language(&self) -> Language{
        *self.language
    }

    fn script(&self) -> Script{
        *self.script
    }
}

impl<'text> From<&'text str> for LanguageDetection<'text, 'text> {
    fn from(value: &'text str) -> Self {
        LanguageDetection::new_init(value, Language::Unknown, Script::Unknown)
    }
}

impl<'text, 'detector> Debug for LanguageDetection<'detector, 'text> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageDetection")
            .field("text", &self.text)
            .field("script", &self.script)
            .field("language", &self.language)
            .finish()
    }
}


#[derive(Copy, Clone)]
pub struct DetectScript<'detector, 'text>{
    pub detector : &'detector dyn LanguageDetector,
    pub text: &'text str
}

impl<'detector, 'text> FnOnce<()> for DetectScript<'detector, 'text> {
    type Output = Script;

    extern "rust-call" fn call_once(self, args: ()) -> Self::Output {
        self.detector.detect_script(self.text)
    }
}

#[derive(Copy, Clone)]
pub struct DetectLanguage<'detector, 'text>{
    pub detector : &'detector dyn LanguageDetector,
    pub text: &'text str
}



impl<'detector, 'text> FnOnce<()> for DetectLanguage<'detector, 'text> {
    type Output = Language;

    extern "rust-call" fn call_once(self, args: ()) -> Self::Output {

        self.detector.detect_lang(self.text)
    }
}

