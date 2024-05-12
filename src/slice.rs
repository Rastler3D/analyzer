use std::ops::{Deref, Range};
use std::slice::SliceIndex;
use crate::rc_cow::RcCow;

pub struct Slice<S,Idx>(pub S,pub Idx)
where
    S: Deref,
    Idx: SliceIndex<S::Target>;

impl<Idx,S> Slice<S,Idx>
where
    S: Deref,
    Idx: SliceIndex<S::Target> + Clone
{
    pub fn new(slice: S, index: Idx) -> Self{
        index.clone().index(slice.deref());

        Slice(slice,index)
    }
}

impl<Idx,S> Deref for Slice<S,Idx>
    where
        S: Deref,
        Idx: SliceIndex<S::Target> + Clone
{
    type Target = Idx::Output;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.1.clone().get_unchecked(self.0.deref()) }
    }
}

impl<Idx,S> Clone for Slice<S,Idx>
    where
        S: Deref + Clone,
        Idx: SliceIndex<S::Target> + Clone
{

    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<'a, T, S> From<T> for Slice<RcCow<'a,S>, Range<usize>>
    where
        T: Into<RcCow<'a, S>> + Sized,
{

    fn from(value: T) -> Self {
        let value = value.into();
        let len = value.len();

        Slice(value, 0..len)
    }
}