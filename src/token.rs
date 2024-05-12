use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::rc::Rc;
use anymap::{CloneAny};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use crate::language::Language;
use crate::language_detection::detection::{DetectLanguage, DetectScript};
use crate::lazy::Lazy;
use crate::script::Script;

pub type LazyScript<'tokenizer> = Rc<Lazy<Script, DetectScript<'tokenizer, 'tokenizer>>>;
pub type LazyLanguage<'tokenizer> = Rc<Lazy<Language, DetectLanguage<'tokenizer, 'tokenizer>>>;

pub type OwnedToken<'tokenizer> = Token<String, LazyLanguage<'tokenizer>, LazyScript<'tokenizer>, AnyMap>;
pub type BorrowedToken<'borrow, 'tokenizer> = Token<&'borrow mut String, &'borrow LazyLanguage<'tokenizer>, &'borrow LazyScript<'tokenizer>, &'borrow mut AnyMap>;
pub type AnyMap = anymap::Map<dyn CloneAny>;
#[derive(Debug, Copy, Clone)]
pub struct Token<Str, Lang, Script, Attr>
{
    /// Text content of the token.
    pub text: Str,

    /// Starting position of the token in bytes.
    pub offset_from: usize,

    /// Ending position of the token in bytes.
    pub offset_to: usize,

    /// Position, expressed in number of tokens.
    pub position: usize,

    /// The length expressed in terms of number of original tokens.
    pub position_length: usize,

    pub token_kind: TokenKind,
    pub script: Script,
    pub language: Lang,
    pub attributes: Attr
}

impl<'tokenizer, Str, Lang, Script, Attr> Token<Str, Lang, Script, Attr>
    where
        Str: BorrowMut<String>,
        Lang: Borrow<LazyLanguage<'tokenizer>>,
        Script: Borrow<LazyScript<'tokenizer>>,
        Attr: BorrowMut<AnyMap>
{
    /// reset to default
    pub fn reset(&mut self) {
        self.offset_from = 0;
        self.offset_to = 0;
        self.position = usize::MAX;
        self.position_length = 1;
    }

    pub fn is_separator(&self) -> bool{
        matches!(self.token_kind, TokenKind::Separator(_))
    }

    pub fn is_word(&self) -> bool{
        matches!(self.token_kind, TokenKind::Word(_))
    }

    pub fn separator_kind(&self) -> Option<SeparatorKind> {
        if let TokenKind::Separator(s) = self.token_kind {
            Some(s)
        } else {
            None
        }
    }
    pub fn original_lengths(&self, num_bytes: usize) -> (usize, usize) {
        self.text
            .borrow()
            .char_indices()
            .take_while(|(byte_index, _)| *byte_index < num_bytes)
            .enumerate()
            .last()
            .map_or((0, 0), |(char_index, (byte_index, c))| {
                let char_count = char_index + 1;
                let byte_len = byte_index + c.len_utf8();
                (char_count, byte_len)
            })
    }
}

#[derive(Copy, Clone, Debug, Serialize,Deserialize)]
pub enum TokenKind{
    Word(TokenFlags),
    Separator(SeparatorKind)
}

bitflags! {
    #[derive(Copy, Clone, Debug, Serialize,Deserialize)]
    pub struct TokenFlags: u8{
        const Exact = 0b00000001;
        const Prefix = 0b00000010;
    }
}

#[derive(Copy, Clone, Debug, Serialize,Deserialize, PartialEq, Eq)]
pub enum SeparatorKind{
    Hard,
    Soft,
    PhraseQuote,
    Negative
}


 impl<'tokenizer> OwnedToken<'tokenizer>{
     pub fn new(script: LazyScript<'tokenizer>, language: LazyLanguage<'tokenizer>) -> Self{
         Token {
             offset_from: 0,
             offset_to: 0,
             position: usize::MAX,
             text: String::new(),
             position_length: 1,
             token_kind: TokenKind::Word(TokenFlags::empty()),
             script: script,
             language: language,
             attributes: AnyMap::new(),
         }
     }
     pub fn borrowed(&mut self) -> BorrowedToken<'_, 'tokenizer>{
         BorrowedToken{
             text: &mut self.text,
             offset_from: self.offset_from,
             offset_to: self.offset_to,
             position: self.position,
             position_length: self.position,
             token_kind: self.token_kind,
             script: &self.script,
             language: &self.language,
             attributes: &mut self.attributes,
         }
     }

     pub fn copy_from_borrowed(&mut self, borrowed_token: BorrowedToken<'_, 'tokenizer>){
         borrowed_token.clone_into_owned(self)
     }
 }

impl<'borrow, 'tokenizer> BorrowedToken<'borrow, 'tokenizer>{
    pub fn new(text: &'borrow mut String, script: &'borrow LazyScript<'tokenizer>, language: &'borrow LazyLanguage<'tokenizer>, attributes: &'borrow mut AnyMap) -> Self{
        Token {
            offset_from: 0,
            offset_to: 0,
            position: usize::MAX,
            text: text,
            position_length: 1,
            token_kind: TokenKind::Word(TokenFlags::empty()),
            script: script,
            language: language,
            attributes: attributes,
        }
    }
    pub fn to_owned(&self) -> OwnedToken<'tokenizer>{
        OwnedToken{
            text: self.text.to_string(),
            offset_from: self.offset_from,
            offset_to: self.offset_to,
            position: self.position,
            position_length: self.position,
            token_kind: self.token_kind,
            script: Rc::clone(self.script),
            language: Rc::clone(self.language),
            attributes: self.attributes.clone(),
        }
    }

    pub fn clone_into_owned(&self, mut owned_token: &mut OwnedToken<'tokenizer>){
        owned_token.text.clear();
        owned_token.text.push_str(self.text);
        owned_token.offset_from = self.offset_from;
        owned_token.offset_to = self.offset_to;
        owned_token.position = self.position;
        owned_token.position_length = self.position_length;
        owned_token.token_kind = self.token_kind;
        owned_token.script = Rc::clone(&self.script);
        owned_token.language = Rc::clone(&self.language);
        owned_token.attributes = self.attributes.clone();
    }
}

