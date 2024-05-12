use std::borrow::Cow;
use regex::Regex;
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};


use crate::char_filter::CharacterFilterRegistry;

use crate::char_filter::CharacterFilter;
extern crate serde_regex;
#[derive(Clone, JsonSchema, Serialize, Deserialize)]
pub struct RegexCharacterFilter{
    #[serde(with = "serde_regex")]
    #[schemars(with = "std::string::String")]
    pub pattern: Regex,
    pub replacement: String
}

#[typetag::serde]
impl CharacterFilter for RegexCharacterFilter {
    fn apply<'a>(&self, text: Cow<'a, str>) -> Cow<'a, str> {
        let result = self.pattern.replace_all(&text, &self.replacement);
        match result {
            Cow::Borrowed(_) => text,
            Cow::Owned(result) => Cow::Owned(result)
        }
    }
}