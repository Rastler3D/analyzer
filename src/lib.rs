#![feature(impl_trait_in_assoc_type)]
#![feature(lazy_cell)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(type_alias_impl_trait)]
#![feature(unsize)]
#![feature(ptr_metadata)]
#![feature(generic_const_exprs)]
#![feature(coerce_unsized)]
#![feature(slice_index_methods)]
#![feature(coroutines, coroutine_trait)]
#![feature(rustc_attrs)]
#![feature(nll)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(negative_impls)]


pub mod language_detection;
pub mod token;
pub mod token_filter;
pub mod tokenizer;
pub mod char_filter;
pub mod language;
pub mod script;
pub mod lazy;
pub mod inline_dyn;
pub mod analyzer;
//pub mod rc_cow;
//mod slice;





