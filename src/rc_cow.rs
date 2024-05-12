use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

pub enum RcCow<'a, B: ?Sized + 'a>
    where
        B: ToOwned,
{
    Borrowed(&'a B),

    Owned(Rc<<B as ToOwned>::Owned>),
}


impl<B: ?Sized + ToOwned> Clone for RcCow<'_, B> {
    fn clone(&self) -> Self {
        match *self {
            RcCow::Borrowed(b) => RcCow::Borrowed(b),
            RcCow::Owned(ref o) => RcCow::Owned(Rc::clone(o))
        }
    }
}

impl<B: ?Sized + ToOwned> RcCow<'_, B> {
    pub const fn is_borrowed(&self) -> bool {
        match *self {
            RcCow::Borrowed(_) => true,
            RcCow::Owned(_) => false,
        }
    }


    pub const fn is_owned(&self) -> bool {
        !self.is_borrowed()
    }
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned {
        match *self {
            RcCow::Borrowed(borrowed) => {
                *self = RcCow::Owned(Rc::new(borrowed.to_owned()));
                match *self {
                    RcCow::Borrowed(..) => unreachable!(),
                    RcCow::Owned(ref mut owned) => owned,
                }
            }
            RcCow::Owned(ref mut owned) => owned,
        }
    }


    pub fn into_owned(self) -> <B as ToOwned>::Owned {
        match self {
            RcCow::Borrowed(borrowed) => borrowed.to_owned(),
            RcCow::Owned(owned) => owned,
        }
    }
}


impl<B: ?Sized + ToOwned> Deref for RcCow<'_, B>
    where
        B::Owned: Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            RcCow::Borrowed(borrowed) => borrowed,
            RcCow::Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<B: ?Sized> Default for RcCow<'_, B>
    where
        B: ToOwned<Owned: Default>,
{
    /// Creates an owned Cow<'a, B> with the default value for the contained owned value.
    fn default() -> Self {
        Rc::Owned(Rc::new(<B as ToOwned>::Owned::default()))
    }
}

impl<B: ?Sized> fmt::Debug for RcCow<'_, B>
    where
        B: fmt::Debug + ToOwned<Owned: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RcCow::Borrowed(ref b) => fmt::Debug::fmt(b, f),
            RcCow::Owned(ref o) => fmt::Debug::fmt(o, f),
        }
    }
}



impl<'a, B: ?Sized + 'a> From<Cow<'a,B>> for RcCow<'a, B>
where
    B: ToOwned
{
    fn from(value: Cow<'a,B>) -> Self {
        match value {
            Cow::Borrowed(borrowed) => RcCow::Borrowed(borrowed),
            Cow::Owned(owned) => RcCow::Owned(Rc::new(owned))
        }
    }
}

impl<'a, B: ?Sized + 'a> From<&'a B> for RcCow<'a, B>
where
    B: ToOwned
{
    fn from(value: &'a B) -> Self {
        RcCow::Borrowed(value)
    }
}

impl<'a, B: ?Sized + 'a> From<Rc<B::Owned>> for RcCow<'a, B>
where
    B: ToOwned
{
    fn from(value: B::Owned) -> Self {
        RcCow::Owned(value)
    }
}