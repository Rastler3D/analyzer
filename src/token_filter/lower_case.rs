use std::mem;
use std::mem::take;
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use crate::token::BorrowedToken;
use crate::token_filter::TokenFilter;
use crate::tokenizer::token_stream::TokenStream;
use crate::token_filter::TokenFilterRegistry;

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct LowerCaseFilter{
}
#[typetag::serde]
impl TokenFilter for LowerCaseFilter {
    type TokenStream<'token, T: TokenStream<'token> + 'token> = LowerCaseTokenStream<T>;

    fn apply<'token, T: TokenStream<'token> + 'token>(&'token self, token_stream: T) -> Self::TokenStream<'token, T> {
        LowerCaseTokenStream {
            tail: token_stream,
            buffer: String::with_capacity(100)
        }
    }
}


pub struct LowerCaseTokenStream<T> {
    buffer: String,
    tail: T,
}

// writes a lowercased version of text into output.
fn to_lowercase_unicode(text: &str, output: &mut String) {
    output.clear();
    output.reserve(50);
    for c in text.chars() {
        // Contrary to the std, we do not take care of sigma special case.
        // This will have an normalizationo effect, which is ok for search.
        output.extend(c.to_lowercase());
    }
}

impl<'token, T: TokenStream<'token>> TokenStream<'token> for LowerCaseTokenStream<T> {
    fn next<'a>(&'a mut self) -> Option<BorrowedToken<'a, 'token>> {

        if let Some(mut token) = self.tail.next() {
            if token.text.is_ascii() {
                // fast track for ascii.
                token.text.make_ascii_lowercase();
            } else {
                to_lowercase_unicode(&token.text, &mut self.buffer);
                mem::swap(token.text, &mut self.buffer);
            }

            return Some(token)
        }
        None
    }


}