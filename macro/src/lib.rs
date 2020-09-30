use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::convert::identity;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token, Attribute, AttributeArgs, Block,
    Expr, FnArg, GenericArgument, Ident, ImplItem, ImplItemMethod, ItemFn, ItemImpl, ItemMod,
    LitStr, Pat, PathArguments, PathSegment, Signature, Type, TypePath, Visibility,
};
use unzip_n::unzip_n;

#[derive(FromMeta)]
struct TestFuzzImplOpts {}

#[proc_macro_attribute]
pub fn test_fuzz_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let _ = TestFuzzImplOpts::from_list(&attr_args).unwrap();

    let item = parse_macro_input!(item as ItemImpl);
    let ItemImpl {
        attrs,
        defaultness,
        unsafety,
        impl_token,
        generics,
        trait_,
        self_ty,
        brace_token: _,
        items,
    } = item;
    let (impl_items, modules) = map_impl_items(&*self_ty, &items);

    // smoelius: Without the next line, you get:
    //   the trait `quote::ToTokens` is not implemented for `(std::option::Option<syn::token::Bang>, syn::Path, syn::token::For)`
    let trait_ = trait_.map(|(bang, path, for_)| quote! { #bang #path #for_ });

    let result = quote! {
        #(#attrs)* #defaultness #unsafety #impl_token #generics #trait_ #self_ty {
            #(#impl_items)*
        }

        #(#modules)*
    };
    log(&result.to_token_stream());
    result.into()
}

fn map_impl_items(self_ty: &Type, items: &[ImplItem]) -> (Vec<ImplItem>, Vec<ItemMod>) {
    let impl_items_modules = items.iter().map(map_impl_item(self_ty));

    let (impl_items, modules): (Vec<_>, Vec<_>) = impl_items_modules.unzip();

    let modules = modules.into_iter().filter_map(identity).collect();

    (impl_items, modules)
}

fn map_impl_item(self_ty: &Type) -> impl Fn(&ImplItem) -> (ImplItem, Option<ItemMod>) {
    let self_ty = self_ty.clone();
    move |impl_item| {
        if let ImplItem::Method(method) = &impl_item {
            method
                .attrs
                .iter()
                .find_map(|attr| {
                    if attr.path.is_ident("test_fuzz") {
                        Some(map_method(&self_ty, &opts_from_attr(attr), method))
                    } else {
                        None
                    }
                })
                .unwrap_or((impl_item.clone(), None))
        } else {
            (impl_item.clone(), None)
        }
    }
}

fn map_method(
    self_ty: &Type,
    opts: &TestFuzzOpts,
    method: &ImplItemMethod,
) -> (ImplItem, Option<ItemMod>) {
    let ImplItemMethod {
        attrs,
        vis,
        defaultness,
        sig,
        block,
    } = &method;

    let attrs = attrs
        .iter()
        .map(|attr| {
            let mut attr = attr.clone();
            if attr.path.is_ident("test_fuzz") {
                let mut opts = opts_from_attr(&attr);
                opts.skip = Some(());
                attr.tokens = tokens_from_opts(&opts).into();
            }
            attr
        })
        .collect();

    let (method, module) = map_method_or_fn(
        &Some(self_ty.clone()),
        &opts,
        &attrs,
        vis,
        defaultness,
        sig,
        block,
    );

    (parse_quote!( #method ), module)
}

#[derive(Clone, Debug, Default, FromMeta)]
struct TestFuzzOpts {
    #[darling(default)]
    skip: Option<()>,
    #[darling(default)]
    rename: Option<Ident>,
}

#[proc_macro_attribute]
pub fn test_fuzz(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let opts = TestFuzzOpts::from_list(&attr_args).unwrap();

    let item = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = &item;
    let (item, module) = map_method_or_fn(&None, &opts, attrs, vis, &None, sig, block);
    let result = quote! {
        #item
        #module
    };
    log(&result.to_token_stream());
    result.into()
}

#[allow(clippy::ptr_arg)]
fn map_method_or_fn(
    self_ty: &Option<Type>,
    opts: &TestFuzzOpts,
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    defaultness: &Option<token::Default>,
    sig: &Signature,
    block: &Block,
) -> (TokenStream2, Option<ItemMod>) {
    let stmts = &block.stmts;
    if opts.skip.is_some() {
        return (
            parse_quote! {
                #(#attrs)* #vis #defaultness #sig {
                    #(#stmts)*
                }
            },
            None,
        );
    }

    let (receiver, tys, ser_args, de_args) = map_args(self_ty, sig);
    let pub_tys: Vec<TokenStream2> = tys.iter().map(|ty| quote! { pub #ty }).collect();
    let target_ident = &sig.ident;
    let renamed_target_ident = opts.rename.as_ref().unwrap_or(target_ident);
    let mod_ident = Ident::new(&format!("{}_fuzz", renamed_target_ident), Span::call_site());
    let call: Expr = if receiver {
        let mut de_args = de_args.iter();
        let self_arg = de_args
            .next()
            .expect("should have at least one deserialized argument");
        parse_quote! {
            #self_arg . #target_ident(
                #(#de_args),*
            )
        }
    } else if let Some(self_ty) = self_ty {
        parse_quote! {
            #self_ty :: #target_ident(
                #(#de_args),*
            )
        }
    } else {
        parse_quote! {
            super :: #target_ident(
                #(#de_args),*
            )
        }
    };
    let call_with_deserialized_arguments = {
        #[cfg(feature = "persistent")]
        quote! {
            // smoelius: Remove the next line once 5142c995 appears in afl.rs on crates.io.
            use test_fuzz::{afl, __fuzz};
            afl::fuzz!(|data: &[u8]| {
                let _ = test_fuzz::runtime::read_args::<Args, _>(data).map(|args|
                    #call
                );
            });
        }
        #[cfg(not(feature = "persistent"))]
        quote! {
            std::panic::set_hook(std::boxed::Box::new(|_| std::process::abort()));
            let _ = test_fuzz::runtime::read_args::<Args, _>(std::io::stdin()).map(|args|
                #call
            );
            let _ = std::panic::take_hook();
        }
    };
    (
        parse_quote! {
            #(#attrs)* #vis #defaultness #sig {
                #[cfg(test)]
                if !test_fuzz::runtime::fuzzing() {
                    test_fuzz::runtime::write_args(&#mod_ident::Args(
                        #(#ser_args),*
                    ));
                }

                #(#stmts)*
            }
        },
        Some(parse_quote! {
            #[cfg(test)]
            mod #mod_ident {
                use super::*;

                #[derive(serde::Deserialize, serde::Serialize)]
                pub(super) struct Args(
                    #(#pub_tys),*
                );

                #[test]
                fn entry() {
                    if test_fuzz::runtime::fuzzing() {
                        #call_with_deserialized_arguments
                    }
                }
            }
        }),
    )
}

fn map_args(
    self_ty: &Option<Type>,
    sig: &Signature,
) -> (
    bool,
    Vec<Type>,
    Vec<Expr>,
    Vec<Expr>,
) {
    unzip_n!(4);

    let (receiver, ty, ser, de): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = sig
        .inputs
        .iter()
        .enumerate()
        .map(map_arg(self_ty))
        .unzip_n();

    let receiver = receiver.first().map_or(false, |&x| x);

    (receiver, ty, ser, de)
}

fn map_arg(
    self_ty: &Option<Type>,
) -> impl Fn((usize, &FnArg)) -> (bool, Type, Expr, Expr) {
    let self_ty = self_ty.clone();
    move |(i, arg)| {
        let i = Literal::usize_unsuffixed(i);
        match arg {
            FnArg::Receiver(_) => (
                true,
                parse_quote! { #self_ty },
                parse_quote! { self.clone() },
                parse_quote! { args.#i },
            ),
            FnArg::Typed(pat_ty) => {
                let pat = &*pat_ty.pat;
                let ty = &*pat_ty.ty;
                let default = (
                    false,
                    parse_quote! { #ty },
                    parse_quote! { #pat },
                    parse_quote! { args.#i },
                );
                match ty {
                    Type::Path(path) => map_arc_arg(&i, pat, path)
                        .map(|(ty, ser, de)| (false, ty, ser, de))
                        .unwrap_or(default),
                    Type::Reference(ty) => {
                        let ty = &*ty.elem;
                        let (ty, ser, de) = map_ref_arg(&i, pat, ty);
                        (false, ty, ser, de)
                    }
                    _ => default,
                }
            }
        }
    }
}

fn map_arc_arg(
    i: &Literal,
    pat: &Pat,
    path: &TypePath,
) -> Option<(Type, Expr, Expr)> {
    if let Some(PathArguments::AngleBracketed(args)) =
        match_type_path(path, &["std", "sync", "Arc"])
    {
        if args.args.len() == 1 {
            if let GenericArgument::Type(ty) = &args.args[0] {
                Some((
                    parse_quote! { #ty },
                    parse_quote! { (*#pat).clone() },
                    parse_quote! { std::sync::Arc::new(args.#i) },
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn map_ref_arg(i: &Literal, pat: &Pat, ty: &Type) -> (Type, Expr, Expr) {
    match ty {
        Type::Path(path) if match_type_path(path, &["str"]) == Some(PathArguments::None) => (
            parse_quote! { String },
            parse_quote! { #pat.to_owned() },
            parse_quote! { args.#i.as_str() },
        ),
        Type::Slice(ty) => {
            let ty = &*ty.elem;
            (
                parse_quote! { Vec<#ty> },
                parse_quote! { #pat.to_vec() },
                parse_quote! { args.#i.as_slice() },
            )
        }
        _ => (
            parse_quote! { #ty },
            parse_quote! { #pat.clone() },
            parse_quote! { &args.#i },
        ),
    }
}

fn opts_from_attr(attr: &Attribute) -> TestFuzzOpts {
    attr.parse_args::<TokenStream2>()
        .map_or(TestFuzzOpts::default(), |tokens| {
            let attr_args = parse_macro_input::parse::<AttributeArgs>(tokens.into()).unwrap();
            TestFuzzOpts::from_list(&attr_args).unwrap()
        })
}

fn tokens_from_opts(opts: &TestFuzzOpts) -> TokenStream {
    let mut attrs = Punctuated::<TokenStream2, token::Comma>::default();
    if let Some(()) = &opts.skip {
        attrs.push(quote! { skip });
    }
    if let Some(rename) = &opts.rename {
        let rename_str = stringify(rename);
        attrs.push(quote! { rename = #rename_str });
    }
    (quote! {
        (
            #attrs
        )
    })
    .into()
}

fn stringify(ident: &Ident) -> LitStr {
    LitStr::new(ident.to_string().as_str(), Span::call_site())
}

fn match_type_path(path: &TypePath, other: &[&str]) -> Option<PathArguments> {
    let mut path = path.clone();
    let args = path.path.segments.last_mut().map(|segment| {
        let args = segment.arguments.clone();
        segment.arguments = PathArguments::None;
        args
    });
    let lhs = path.path.segments.into_iter().collect::<Vec<_>>();
    let rhs = other
        .iter()
        .map(|s| {
            let ident = Ident::new(s, Span::call_site());
            PathSegment {
                ident,
                arguments: PathArguments::None,
            }
        })
        .collect::<Vec<_>>();
    if path.qself.is_none() && lhs == rhs {
        args
    } else {
        None
    }
}

fn log(tokens: &TokenStream2) {
    if cfg!(feature = "logging") {
        println!("{}", tokens);
    }
}
