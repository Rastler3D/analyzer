use std::marker::Unsize;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::{ Pointee};

pub struct Dynamic<Dyn: Pointee<Metadata = ptr::DynMetadata<Dyn>> + ?Sized, const SIZE: usize = { size_of::<u128>() }> {
    data: [u8;SIZE],
    metadata: ptr::DynMetadata<Dyn>,
}


impl<const SIZE: usize, Dyn: Pointee<Metadata = ptr::DynMetadata<Dyn>> + ?Sized> Dynamic<Dyn, SIZE>{
    pub const SIZE: usize = SIZE;

    #[inline(always)]
    pub fn new<T>(value: T) -> Self
    where
        T: Unsize<Dyn>,
        Box<T>: Unsize<Dyn>
    {
        if size_of::<T>() <= SIZE{
            unsafe { Self::inline_unchecked(value) }
        } else {
            Self::boxed(value)
        }
    }
    #[inline(always)]
    unsafe fn inline_unchecked<T: Unsize<Dyn>>(value: T) -> Self{
        let mut data = [0u8;SIZE];
        let metadata = ptr::metadata(&value as &Dyn);
        let ptr = data.as_mut_ptr() as *mut T;
        unsafe { ptr.write(value) };

        Self{
            data,
            metadata
        }
    }
    #[inline(always)]
    pub fn inline<T: Unsize<Dyn>>(value: T) -> Self
    where
        Assert<{size_of::<T>() <= SIZE}>: True
    {
        unsafe { Self::inline_unchecked(value) }
    }
    #[inline(always)]
    pub fn boxed<T>(value:T) -> Self
    where
        Box<T>: Unsize<Dyn>
    {
        let boxed = Box::new(value);
        let mut data = [0u8;SIZE];
        let metadata = ptr::metadata(&boxed as &Dyn);
        let ptr = data.as_mut_ptr() as *mut Box<T>;
        unsafe { ptr.write(boxed) };

        Self{
            data,
            metadata
        }
    }
}

impl<const SIZE: usize, Dyn: Pointee<Metadata = ptr::DynMetadata<Dyn>> + ?Sized> Deref for Dynamic<Dyn, SIZE> {
    type Target = Dyn;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(&self.data as *const u8 as *const (), self.metadata) }
    }
}
impl<const SIZE: usize, Dyn: Pointee<Metadata = ptr::DynMetadata<Dyn>> + ?Sized> DerefMut for Dynamic<Dyn, SIZE> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ptr::from_raw_parts_mut(&mut self.data as *mut u8 as *mut (), self.metadata) }
    }
}

pub trait DynamicFrom<T>{
    fn from(value: Self) -> T;
}
// default impl<T> DynamicInto for T
// {
//     fn into_dynamic<Dy1n: Pointee<Metadata = ptr::DynMetadata<Dy1n>> + ?Sized>(self) -> Dynamic<Dy1n> where Self: Unsize<Dy1n>{
//         Dynamic::new(self)
//     }
// }
//
// impl<const SIZE: usize, Dyn: Pointee<Metadata = ptr::DynMetadata<Dyn>> + ?Sized> DynamicInto for Dynamic<Dyn, SIZE> {
//
//     fn into_dynamic<Dy1n: Pointee<Metadata = ptr::DynMetadata<Dy1n>> + ?Sized>(self) -> Dynamic<Dy1n> where Self: Unsize<Dy1n>{
//         self
//     }
// }

impl<D: Pointee<Metadata = ptr::DynMetadata<D>> + ?Sized + Iterator, const SIZE: usize> Iterator for Dynamic<D,SIZE> {
    type Item = D::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.deref_mut().next()
    }
}

impl<D: Pointee<Metadata = ptr::DynMetadata<D>> + ?Sized, const SIZE: usize> Drop for Dynamic<D, SIZE> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.deref_mut()) }
    }
}



pub struct Assert<const COND: bool>;

pub trait True{}
pub trait False{}
impl True for Assert<true>{}
impl False for Assert<false>{}


#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::mem::size_of_val;
    use super::*;

    #[test]
    fn inline_dyn() {
        let str = "String";
        println!("{}", size_of_val(&str));
        let mut inline_dyn: Dynamic<dyn Debug, { size_of::<u128>() }> = Dynamic::new(str);

        println!("{:?}", inline_dyn.deref_mut());

    }

    #[test]
    fn boxed() {
        let str = "String";
        println!("{}", size_of_val(&str));
        let mut inline_dyn: Dynamic<dyn Debug, { size_of::<u128>() }> = Dynamic::boxed(str);
        println!("{:?}", inline_dyn.deref_mut());

    }
}

