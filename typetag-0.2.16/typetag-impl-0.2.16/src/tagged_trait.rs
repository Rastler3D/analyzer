use crate::{Mode, TraitArgs};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, Error, ItemTrait, LitStr, TraitBoundModifier, TypeParamBound};
use crate::parse::Tag;

pub(crate) fn expand(args: TraitArgs, mut input: ItemTrait, mode: Mode) -> TokenStream {
    if mode.de && !input.generics.params.is_empty() {
        let msg = "deserialization of generic traits is not supported yet; \
                   use #[typetag::serialize] to generate serialization only";
        return Error::new_spanned(input.generics, msg).to_compile_error();
    }

    augment_trait(&mut input, mode, &args.0);
    let (serialize_impl, deserialize_impl) = match args.1 {
        Tag::External => externally_tagged(&input, &args.0),
        Tag::Internal {
            tag,
            default_variant,
        } => internally_tagged(tag, default_variant, &input, &args.0),
        Tag::Adjacent {
            tag,
            content,
            default_variant,
            deny_unknown_fields,
        } => adjacently_tagged(tag, content, default_variant, deny_unknown_fields, &input, &args.0),
    };

    let schema_registry = schema_registry();
    let schema_impl = quote! {
        #schema_registry
        typetag::__private::externally::schema(registry, gen)
    };

    let ident = &input.ident;
    let object = if let Some(receiver) = &args.0{
        receiver
    } else {
        ident
    };

    let mut expanded = TokenStream::new();
    let mut impl_generics = input.generics.clone();
    impl_generics.params.push(parse_quote!('typetag));
    let (impl_generics, _, _) = impl_generics.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    if mode.ser {

        expanded.extend(quote! {
            impl #impl_generics typetag::__private::serde::Serialize
            for dyn #object #ty_generics + 'typetag #where_clause {
                fn serialize<S>(&self, serializer: S) -> typetag::__private::Result<S::Ok, S::Error>
                where
                    S: typetag::__private::serde::Serializer,
                {
                    #serialize_impl
                }
            }
        });

        for marker_traits in &[quote!(Send), quote!(Sync), quote!(Send + Sync)] {
            expanded.extend(quote! {
                impl #impl_generics typetag::__private::serde::Serialize
                for dyn #object #ty_generics + #marker_traits + 'typetag #where_clause {
                    fn serialize<S>(&self, serializer: S) -> typetag::__private::Result<S::Ok, S::Error>
                    where
                        S: typetag::__private::serde::Serializer,
                    {
                        typetag::__private::serde::Serialize::serialize(self as &dyn #object #ty_generics, serializer)
                    }
                }
            });
        }
    }
    let registry = build_registry(&input, object);
    if mode.de {

        let is_send = has_supertrait(&input, "Send");
        let is_sync = has_supertrait(&input, "Sync");
        let (strictest, others) = match (is_send, is_sync) {
            (false, false) => (quote!(), vec![]),
            (true, false) => (quote!(Send), vec![quote!()]),
            (false, true) => (quote!(Sync), vec![quote!()]),
            (true, true) => (
                quote!(Send + Sync),
                vec![quote!(), quote!(Send), quote!(Sync)],
            ),
        };

        expanded.extend(quote! {

            impl typetag::__private::Strictest for dyn #object {
                type Object = dyn #object + #strictest ;
            }

            #[allow(unknown_lints, non_local_definitions)] // false positive: https://github.com/rust-lang/rust/issues/121621
            impl<'de> typetag::__private::serde::Deserialize<'de> for typetag::__private::Box<dyn #object + #strictest> {
                fn deserialize<D>(deserializer: D) -> typetag::__private::Result<Self, D::Error>
                where
                    D: typetag::__private::serde::Deserializer<'de>,
                {
                    #deserialize_impl
                }
            }
        });

        for marker_traits in others {
            expanded.extend(quote! {
                #[allow(unknown_lints, non_local_definitions)] // false positive: https://github.com/rust-lang/rust/issues/121621
                impl<'de> typetag::__private::serde::Deserialize<'de> for typetag::__private::Box<dyn #object + #marker_traits> {
                    fn deserialize<D>(deserializer: D) -> typetag::__private::Result<Self, D::Error>
                    where
                        D: typetag::__private::serde::Deserializer<'de>,
                    {
                        typetag::__private::Result::Ok(
                            <typetag::__private::Box<dyn #object + #strictest>
                                as typetag::__private::serde::Deserialize<'de>>::deserialize(deserializer)?
                        )
                    }
                }
            });
        }
    }

    expanded.extend(quote! {
        #[allow(unknown_lints, non_local_definitions)] // false positive: https://github.com/rust-lang/rust/issues/121621
        impl #impl_generics typetag::__private::schemars::JsonSchema
        for dyn #object #ty_generics + 'typetag #where_clause {
            fn schema_name() -> std::string::String{
                String::from(std::stringify!(#ident))
            }
            fn schema_id() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(std::concat!(std::module_path!(), "::", std::stringify!(#ident) ))
            }

            fn json_schema(gen: &mut typetag::__private::schemars::gen::SchemaGenerator) -> typetag::__private::schemars::schema::Schema {
                #schema_impl
            }
        }
    });

    for marker_traits in &[quote!(Send), quote!(Sync), quote!(Send + Sync)] {
        expanded.extend(quote! {
            #[allow(unknown_lints, non_local_definitions)] // false positive: https://github.com/rust-lang/rust/issues/121621
            impl #impl_generics typetag::__private::schemars::JsonSchema
            for dyn #object #ty_generics + #marker_traits + 'typetag #where_clause {
                fn schema_name() -> std::string::String{
                    String::from(std::stringify!(#ident))
                }
                fn schema_id() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Borrowed(std::concat!(std::module_path!(), "::", std::stringify!(#ident) ))
                }

                fn json_schema(gen: &mut typetag::__private::schemars::gen::SchemaGenerator) -> typetag::__private::schemars::schema::Schema {
                    #schema_impl
                }
            }
        });
    }

    quote! {
        #input

        #registry

        #[allow(non_upper_case_globals)]
        const _: () = {
            #expanded
        };
    }
}

fn augment_trait(input: &mut ItemTrait, mode: Mode, receiver: &Option<Ident>) {
    if mode.ser {
        if receiver.is_none() {
            input.supertraits.push(parse_quote!(typetag::Serialize));
        }
        input.items.push(parse_quote! {
            #[doc(hidden)]
            fn type_name(&self) -> &'static str;
        });
    }
}

fn build_registry(input: &ItemTrait, receiver: &Ident) -> TokenStream {
    let vis = &input.vis;
    let object = &input.ident;
    let struct_name = format_ident!("{}Registry", object);
    quote! {
        type TypetagStrictest = <dyn #receiver as typetag::__private::Strictest>::Object;
        type TypetagFn = typetag::__private::DeserializeFn<TypetagStrictest>;
        #vis struct TypetagRegistration<T> {
            name: &'static str,
            deserializer: T,
            schema: typetag::__private::SchemaFn
        }
        typetag::__private::inventory::collect!(TypetagRegistration<TypetagFn>);

        #[doc(hidden)]
        #vis struct #struct_name;

        impl #struct_name {
            #[doc(hidden)]
            #vis const fn typetag_register(name: &'static str, deserializer: TypetagFn, schema: typetag::__private::SchemaFn) -> TypetagRegistration<TypetagFn> {
                TypetagRegistration { name, deserializer, schema }
            }
        }
    }
}

fn static_registry() -> TokenStream {
    quote! {
        static TYPETAG: typetag::__private::once_cell::race::OnceBox<typetag::__private::Registry<TypetagStrictest>> = typetag::__private::once_cell::race::OnceBox::new();
        let registry = TYPETAG.get_or_init(|| {
            let mut map = typetag::__private::BTreeMap::new();
            let mut names = typetag::__private::Vec::new();
            for registered in typetag::__private::inventory::iter::<TypetagRegistration<TypetagFn>> {
                match map.entry(registered.name) {
                    typetag::__private::btree_map::Entry::Vacant(entry) => {
                        entry.insert(typetag::__private::Option::Some(registered.deserializer));
                    }
                    typetag::__private::btree_map::Entry::Occupied(mut entry) => {
                        entry.insert(typetag::__private::Option::None);
                    }
                }
                names.push(registered.name);
            }
            names.sort_unstable();
            typetag::__private::Box::new(typetag::__private::Registry { map, names })
        });
    }
}

fn schema_registry() -> TokenStream {
    quote! {
        static TYPETAG: typetag::__private::once_cell::race::OnceBox<typetag::__private::SchemaRegistry> = typetag::__private::once_cell::race::OnceBox::new();
        let registry = TYPETAG.get_or_init(|| {
            let mut schemas = typetag::__private::Vec::new();
            for registered in typetag::__private::inventory::iter::<TypetagRegistration<TypetagFn>> {
                schemas.push((registered.name, registered.schema));
            }
            typetag::__private::Box::new(typetag::__private::SchemaRegistry { schemas })
        });
    }
}

fn externally_tagged(input: &ItemTrait, receiver: &Option<Ident>) -> (TokenStream, TokenStream) {
    let (object, object_name) = if let Some(receiver) = receiver{
        let object_name = receiver.to_string();
        (quote!(#receiver),object_name)
    } else {
        let object = &input.ident;
        let object_name = object.to_string();
        let (_, ty_generics, _) = input.generics.split_for_impl();
        (quote!(#object #ty_generics), object_name)
    };
    let static_registry = static_registry();

    let serialize_impl = quote! {
        let name = <Self as #object>::type_name(self);
        typetag::__private::externally::serialize(serializer, name, self)
    };

    let deserialize_impl = quote! {
        #static_registry
        typetag::__private::externally::deserialize(deserializer, #object_name, registry)
    };



    (serialize_impl, deserialize_impl)
}

fn internally_tagged(
    tag: LitStr,
    default_variant: Option<LitStr>,
    input: &ItemTrait,
    receiver: &Option<Ident>
) -> (TokenStream, TokenStream) {
    let (object, object_name) = if let Some(receiver) = receiver{
        let object_name = receiver.to_string();
        (quote!(#receiver),object_name)
    } else {
        let object = &input.ident;
        let object_name = object.to_string();
        let (_, ty_generics, _) = input.generics.split_for_impl();
        (quote!(#object #ty_generics), object_name)
    };
    let static_registry = static_registry();
    let default_variant_literal = match default_variant {
        Some(variant) => quote!(typetag::__private::Option::Some(#variant)),
        None => quote!(typetag::__private::Option::None),
    };

    let serialize_impl = quote! {
        let name = <Self as #object>::type_name(self);
        typetag::__private::internally::serialize(serializer, #tag, name, self)
    };

    let deserialize_impl = quote! {
        #static_registry
        typetag::__private::internally::deserialize(deserializer, #object_name, #tag, #default_variant_literal, registry)
    };


    (serialize_impl, deserialize_impl)
}

fn adjacently_tagged(
    tag: LitStr,
    content: LitStr,
    default_variant: Option<LitStr>,
    deny_unknown_fields: bool,
    input: &ItemTrait,
    receiver: &Option<Ident>
) -> (TokenStream, TokenStream) {
    let (object, object_name) = if let Some(receiver) = receiver{
        let object_name = receiver.to_string();
        (quote!(#receiver),object_name)
    } else {
        let object = &input.ident;
        let object_name = object.to_string();
        let (_, ty_generics, _) = input.generics.split_for_impl();
        (quote!(#object #ty_generics), object_name)
    };
    let static_registry = static_registry();
    let default_variant_literal = match default_variant {
        Some(variant) => quote!(typetag::__private::Option::Some(#variant)),
        None => quote!(typetag::__private::Option::None),
    };

    let serialize_impl = quote! {
        let name = <Self as #object>::type_name(self);
        typetag::__private::adjacently::serialize(serializer, #object_name, #tag, name, #content, self)
    };

    let deserialize_impl = quote! {
        #static_registry
        typetag::__private::adjacently::deserialize(
            deserializer,
            #object_name,
            &[#tag, #content],
            #default_variant_literal,
            registry,
            #deny_unknown_fields,
        )
    };

    (serialize_impl, deserialize_impl)
}

fn has_supertrait(input: &ItemTrait, find: &str) -> bool {
    for supertrait in &input.supertraits {
        if let TypeParamBound::Trait(trait_bound) = supertrait {
            if let TraitBoundModifier::None = trait_bound.modifier {
                if trait_bound.path.is_ident(find) {
                    return true;
                }
            }
        }
    }
    false
}
