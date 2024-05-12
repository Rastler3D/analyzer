use std::marker::PhantomData;
use crate::inline_dyn::{Dynamic, DynamicFrom};
use crate::token::{BorrowedToken, OwnedToken};

pub trait TokenStream<'token> {
    fn next<'this>(&'this mut self) -> Option<BorrowedToken<'this, 'token>>;

    #[allow(clippy::wrong_self_convention)]
    fn as_iter(self) -> TokenStreamIterator<'token, Self> where Self: Sized{
        TokenStreamIterator::from(self)
    }
}

impl<'token, T: TokenStream<'token>> From<T> for TokenStreamIterator<'token, T>{
    fn from(value: T) -> Self {
        TokenStreamIterator{
            token_stream: value,
            _phantom: PhantomData,
        }
    }
}

pub struct TokenStreamIterator<'token, T: TokenStream<'token>>{
    token_stream: T,
    _phantom: PhantomData<&'token ()>
}

impl<'token, T: TokenStream<'token>> Iterator for TokenStreamIterator<'token, T> {
    type Item = OwnedToken<'token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|x| x.to_owned())
    }
}

impl<'token, T: TokenStream<'token>> TokenStream<'token> for Box<T> {
    fn next<'a>(&'a mut self) -> Option<BorrowedToken<'a, 'token>>{
        (**self).next()
    }
}

impl<'token> TokenStream<'token> for Dynamic<dyn TokenStream<'token>  + 'token> {
    fn next<'a>(&'a  mut self) -> Option<BorrowedToken<'a, 'token>> {
        (**self).next()
    }

}

impl<'token, T: TokenStream<'token> + 'token> DynamicFrom<Dynamic<dyn TokenStream<'token>  + 'token>> for T{
    default fn from(value: Self) -> Dynamic<dyn TokenStream<'token>  + 'token> {
        Dynamic::new(value)
    }
}

impl<'token> DynamicFrom<Dynamic<dyn TokenStream<'token>  + 'token>> for Dynamic<dyn TokenStream<'token>  + 'token>{
    fn from(value: Dynamic<dyn TokenStream<'token>  + 'token>) -> Dynamic<dyn TokenStream<'token>  + 'token>{
        value
    }
}