use proc_macro::TokenStream;
use proc_macro2::{Group, TokenTree};
use quote::{quote, ToTokens};
use syn::{braced, parenthesized, parse_macro_input, Attribute, Block, Expr, Ident, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

#[derive(Debug)]
struct Class {
    ident: Ident,
    vis: Visibility,
    fields: Vec<Field>,
    getters: Vec<Getter>,
    setters: Vec<Setter>,
    methods: Vec<Method>,
}

impl Parse for Class {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        let body;
        braced!(body in input);
        let mut fields = Vec::new();
        let mut getters = Vec::new();
        let mut setters = Vec::new();
        let mut methods = Vec::new();
        while !body.is_empty() {
            let lookahead = body.lookahead1();
            if lookahead.peek(Token![let]) {
                fields.push(body.parse()?);
            } else if lookahead.peek(Token![pub]) {
                getters.push(body.parse()?);
            } else if lookahead.peek(Token![mut]) {
                setters.push(body.parse()?);
            } else if lookahead.peek(Token![fn]) {
                methods.push(body.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(Self { ident, vis, fields, getters, setters, methods })
    }
}

#[derive(Debug)]
struct Field {
    ident: Ident,
    ty: FreezableType,
    default: Option<proc_macro2::TokenStream>,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![let]>()?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let default = if input.parse::<Token![=]>().is_ok() {
            Some(input.parse::<Expr>()
                .map(ToTokens::into_token_stream)
                .unwrap_or(quote!(::std::default::Default::default())))
        } else {
            None
        };
        input.parse::<Token![;]>()?;
        Ok(Self { ident, ty, default })
    }
}

#[derive(Debug)]
struct FreezableType {
    mutable: proc_macro2::TokenStream,
    frozen: proc_macro2::TokenStream,
    new: proc_macro2::TokenStream,
    has_lifetime: bool,
}

impl Parse for FreezableType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let raw = input.parse::<Type>()?.into_token_stream();
        let (new, has_lifetime) = raw.clone().replace_ident("Value", quote!(::starlark::values::Value<'v>), true)
            .replace_ident("Class", quote!(Mut), false)
            .replace_ident("ClassV", quote!(Mut<'v>), true);
        let mutable = quote!(::std::cell::RefCell<#new>);
        let (frozen, _) = raw.replace_ident("Value", quote!(::starlark::values::FrozenValue), false)
            .replace_ident("Class", quote!(Frozen), false)
            .replace_ident("ClassV", quote!(Frozen), false);
        Ok(FreezableType { mutable, frozen, new, has_lifetime })
    }
}

#[derive(Debug)]
struct Getter {
    name: String,
    body: Block,
}

impl Parse for Getter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![pub]>()?;
        let name = input.parse::<Ident>()?.to_string();
        let body = input.parse()?;
        Ok(Self { name, body })
    }
}

#[derive(Debug)]
struct Setter {
    name: String,
    body: Block,
}

impl Parse for Setter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![mut]>()?;
        let name = input.parse::<Ident>()?.to_string();
        let body = input.parse()?;
        Ok(Self { name, body })
    }
}

#[derive(Debug)]
struct Method {
    ident: Ident,
    params: Punctuated<Param, Token![,]>,
    ret: Type,
    body: Block,
}

impl Method {
    fn to_tokens(&self, v: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let Self { ident, params, ret, body } = self;
        let param_idents = params.iter().map(|param| param.ident.clone());
        let param_types = params.iter().map(|param| param.ty.clone());
        let param_attrs = params.iter().map(|param| param.attrs.clone());
        quote! {
            #[allow(clippy::unnecessary_wraps)]
            fn #ident<'v>(this: &Mut #v, heap: &'v ::starlark::values::Heap, #(#(#param_attrs)* #param_idents: #param_types),*) -> ::anyhow::Result<#ret> #body
        }
    }
}

impl Parse for Method {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![fn]>()?;
        let ident = input.parse()?;
        let params_parens;
        parenthesized!(params_parens in input);
        let params = params_parens.parse_terminated(Param::parse, Token![,])?;
        input.parse::<Token![->]>()?;
        let ret = input.parse()?;
        let body = input.parse()?;
        Ok(Self { ident, params, ret, body })
    }
}

#[derive(Debug)]
struct Param {
    ident: Ident,
    ty: Type,
    attrs: Vec<Attribute>
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { ident, ty, attrs })
    }
}

trait ReplaceIdent {
    fn replace_ident(self, name: &str, replacement: proc_macro2::TokenStream, track: bool) -> (proc_macro2::TokenStream, bool);
}

impl ReplaceIdent for proc_macro2::TokenStream {
    fn replace_ident(self, name: &str, replacement: proc_macro2::TokenStream, track: bool) -> (proc_macro2::TokenStream, bool) {
        let mut replaced = false;
        let result = self.into_iter().map(|token| {
            match token {
                TokenTree::Ident(ident) if ident == name => {
                    replaced = true;
                    replacement.clone()
                }
                TokenTree::Group(group) => {
                    let (replacement, replacement_replaced) = group.stream().replace_ident(name, replacement.clone(), track);
                    replaced |= replacement_replaced;
                    let mut new_group = TokenTree::Group(Group::new(group.delimiter(), replacement));
                    new_group.set_span(group.span());
                    new_group.to_token_stream()
                }
                _ => token.into_token_stream(),
            }
        }).collect();
        (result, replaced && track)
    }
}

impl ReplaceIdent for (proc_macro2::TokenStream, bool) {
    fn replace_ident(self, name: &str, replacement: proc_macro2::TokenStream, track: bool) -> (proc_macro2::TokenStream, bool) {
        let (new, replaced) = self.0.replace_ident(name, replacement, track);
        (new, replaced || self.1)
    }
}

fn fields(class: &Class, v: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let field_names: Vec<_> = class.fields.iter().map(|field| field.ident.clone()).collect();
    let mut_types: Vec<_> = class.fields.iter().map(|field| field.ty.mutable.clone()).collect();
    let frozen_types: Vec<_> = class.fields.iter().map(|field| field.ty.frozen.clone()).collect();
    let default_fields: Vec<_> = class.fields.iter().filter(|field| field.default.is_some()).map(|field| field.ident.clone()).collect();
    let new_fields: Vec<_> = class.fields.iter().filter(|field| field.default.is_none()).map(|field| field.ident.clone()).collect();
    let new_types: Vec<_> = class.fields.iter().filter(|field| field.default.is_none()).map(|field| field.ty.new.clone()).collect();
    let defaults: Vec<_> = class.fields.iter().flat_map(|field| field.default.clone()).collect();
    let default = if !class.fields.iter().all(|field| field.default.is_some()) { proc_macro2::TokenStream::new() } else {
        quote! {
            impl #v ::std::default::Default for Mut #v {
                fn default() -> Self {
                    Self {
                        #(#default_fields: ::std::cell::RefCell::new(#defaults)),*
                    }
                }
            }
        }
    };
    let new = class.fields.iter().map(|Field { ident, default, .. }| {
        if let Some(default) = default {
            quote!(#ident: ::std::cell::RefCell::new(#default))
        } else {
            quote!(#ident: ::std::cell::RefCell::new(#ident))
        }
    });
    quote! {
        #[repr(C)]
        #[derive(::std::fmt::Debug, ::starlark::values::NoSerialize, ::allocative::Allocative, ::starlark::values::ProvidesStaticType, ::starlark::values::Trace)]
        pub struct Mut #v {
            #(pub #field_names: #mut_types),*
        }

        #[inline]
        pub fn new #v (#(#new_fields: #new_types),*) -> Mut #v {
            Mut {
                #(#new),*
            }
        }

        #[inline]
        pub fn from_value<'v>(value: ::starlark::values::Value<'v>) -> ::anyhow::Result<&'v Mut #v> {
            value.try_into()
        }

        impl<'v> ::std::convert::TryFrom<::starlark::values::Value<'v>> for &'v Mut #v {
            type Error = ::anyhow::Error;

            #[inline]
            fn try_from(value: ::starlark::values::Value<'v>) -> ::std::result::Result<Self, Self::Error> {
                ::starlark::values::ValueLike::downcast_ref_err(value)
            }
        }

        #default

        #[repr(C)]
        #[derive(::std::fmt::Debug, ::starlark::values::NoSerialize, ::allocative::Allocative, ::starlark::values::ProvidesStaticType, ::starlark::values::Trace)]
        pub struct Frozen {
            #(pub #field_names: #frozen_types),*
        }

        impl<'v> ::starlark::values::AllocValue<'v> for Mut #v {
            #[inline]
            fn alloc_value(self, heap: &'v ::starlark::values::Heap) -> ::starlark::values::Value<'v> {
                heap.alloc_complex(self)
            }
        }

        impl ::starlark::values::AllocFrozenValue for Frozen {
            #[inline]
            fn alloc_frozen_value(self, heap: &::starlark::values::FrozenHeap) -> ::starlark::values::FrozenValue {
                heap.alloc_simple(self)
            }
        }

        impl #v ::starlark::values::Freeze for Mut #v {
            type Frozen = Frozen;

            fn freeze(self, freezer: &::starlark::values::Freezer) -> ::starlark::values::FreezeResult<Self::Frozen> {
                ::starlark::values::FreezeResult::Ok(Self::Frozen {
                    #(#field_names: self.#field_names.freeze(freezer)?),*
                })
            }
        }
    }
}

fn value(class: &Class, v: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let name = class.ident.to_string();
    let display_mut = format!("<internal class {name}>");
    let display_frozen = format!("<frozen internal class {name}>");
    let getter_names: Vec<_> = class.getters.iter().map(|getter| getter.name.clone()).collect();
    let getters = class.getters.iter().map(|getter| getter.body.clone());
    let setter_names = class.setters.iter().map(|setter| setter.name.clone());
    let setters = class.setters.iter().map(|setter| setter.body.clone());
    let methods = class.methods.iter().map(|method| method.to_tokens(v));
    quote! {
        impl #v ::std::fmt::Display for Mut #v {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::write!(f, #display_mut)
            }
        }

        impl ::std::fmt::Display for Frozen {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::write!(f, #display_frozen)
            }
        }

        #[::starlark::values::starlark_value(type = #name, UnpackValue, StarlarkTypeRepr)]
        impl<'v> ::starlark::values::StarlarkValue<'v> for Mut #v
        where
            Self: ::starlark::any::ProvidesStaticType<'v>,
        {
            fn get_attr(&self, attribute: &str, heap: &'v ::starlark::values::Heap) -> ::std::option::Option<::starlark::values::Value<'v>> {
                match attribute {
                    #(#getter_names => #getters),*
                    _ => ::std::option::Option::None
                }
            }

            fn dir_attr(&self) -> ::std::vec::Vec<::std::string::String> {
                vec! [#(::std::stringify!(#getter_names).to_string()),*]
            }

            fn has_attr(&self, attribute: &str, _heap: &'v ::starlark::values::Heap) -> bool {
                self.dir_attr().contains(&attribute.into())
            }

            fn set_attr(&self, attribute: &str, value: ::starlark::values::Value<'v>) -> ::starlark::Result<()> {
                match attribute {
                    #(#setter_names => #setters),*
                    _ => ::starlark::Result::Err(::starlark::values::ValueError::NoAttr(#name.to_string(), attribute.to_string()).into())
                }
            }

            fn get_methods() -> ::std::option::Option<&'static ::starlark::environment::Methods>
            where
                Self: ::std::marker::Sized,
            {
                static METHODS_STATIC: ::starlark::environment::MethodsStatic = ::starlark::environment::MethodsStatic::new();
                METHODS_STATIC.methods(methods)
            }
        }

        #[::starlark::values::starlark_value(type = #name)]
        impl<'v> ::starlark::values::StarlarkValue<'v> for Frozen
        where
            Self: ::starlark::any::ProvidesStaticType<'v>,
        {
            type Canonical = Mut #v;
        }

        #[::starlark::starlark_module]
        fn methods(builder: &mut ::starlark::environment::MethodsBuilder) {
            #(#methods)*
        }
    }
}

#[proc_macro]
pub fn class(tokens: TokenStream) -> TokenStream {
    let class = parse_macro_input!(tokens as Class);
    let v = if class.fields.iter().any(|field| field.ty.has_lifetime) {quote!(<'v>)} else {proc_macro2::TokenStream::new()};
    let fields = fields(&class, &v);
    let value = value(&class, &v);
    let Class { ident, vis, .. } = class;
    let module = quote! {
        #[allow(non_snake_case)]
        #vis mod #ident {
            use super::*;

            #fields
            #value
        }
    };
    module.into()
}
