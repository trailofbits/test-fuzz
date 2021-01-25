use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::convert::identity;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token, Attribute, AttributeArgs, Block,
    Expr, FnArg, GenericArgument, Ident, ImplItem, ImplItemMethod, ItemFn, ItemImpl, ItemMod,
    LitStr, Pat, Path, PathArguments, PathSegment, ReturnType, Signature, Stmt, Type, TypePath,
    TypeReference, Visibility,
};
use unzip_n::unzip_n;

mod util;

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

    // smoelius: Without the next line, you get:
    //   the trait `quote::ToTokens` is not implemented for `(std::option::Option<syn::token::Bang>, syn::Path, syn::token::For)`
    let (trait_path, trait_) = trait_.map_or((None, None), |(bang, path, for_)| {
        (Some(path.clone()), Some(quote! { #bang #path #for_ }))
    });

    let (impl_items, modules) = map_impl_items(&*self_ty, &trait_path, &items);

    let result = quote! {
        #(#attrs)* #defaultness #unsafety #impl_token #generics #trait_ #self_ty {
            #(#impl_items)*
        }

        #(#modules)*
    };
    log(&result.to_token_stream());
    result.into()
}

fn map_impl_items(
    self_ty: &Type,
    trait_path: &Option<Path>,
    items: &[ImplItem],
) -> (Vec<ImplItem>, Vec<ItemMod>) {
    let impl_items_modules = items.iter().map(map_impl_item(self_ty, &trait_path));

    let (impl_items, modules): (Vec<_>, Vec<_>) = impl_items_modules.unzip();

    let modules = modules.into_iter().filter_map(identity).collect();

    (impl_items, modules)
}

fn map_impl_item(
    self_ty: &Type,
    trait_path: &Option<Path>,
) -> impl Fn(&ImplItem) -> (ImplItem, Option<ItemMod>) {
    let self_ty = self_ty.clone();
    let trait_path = trait_path.clone();
    move |impl_item| {
        if let ImplItem::Method(method) = &impl_item {
            method
                .attrs
                .iter()
                .find_map(|attr| {
                    if is_test_fuzz(attr) {
                        Some(map_method(
                            &self_ty,
                            &trait_path,
                            &opts_from_attr(attr),
                            method,
                        ))
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
    trait_path: &Option<Path>,
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
            if is_test_fuzz(&attr) {
                let mut opts = opts_from_attr(&attr);
                opts.skip = true;
                attr.tokens = tokens_from_opts(&opts).into();
            }
            attr
        })
        .collect();

    let (method, module) = map_method_or_fn(
        &Some(self_ty.clone()),
        trait_path,
        &opts,
        &attrs,
        vis,
        defaultness,
        sig,
        block,
    );

    (parse_quote!( #method ), module)
}

// smoelius: Additions to TestFuzzOpts must also be reflected in tokens_from_opts.
#[derive(Clone, Debug, Default, FromMeta)]
struct TestFuzzOpts {
    #[darling(default)]
    force: bool,
    #[darling(default)]
    rename: Option<Ident>,
    #[darling(default)]
    skip: bool,
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
    let (item, module) = map_method_or_fn(&None, &None, &opts, attrs, vis, &None, sig, block);
    let result = quote! {
        #item
        #module
    };
    log(&result.to_token_stream());
    result.into()
}

#[allow(clippy::ptr_arg, clippy::too_many_arguments)]
fn map_method_or_fn(
    self_ty: &Option<Type>,
    trait_path: &Option<Path>,
    opts: &TestFuzzOpts,
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    defaultness: &Option<token::Default>,
    sig: &Signature,
    block: &Block,
) -> (TokenStream2, Option<ItemMod>) {
    let stmts = &block.stmts;
    if opts.skip {
        return (
            parse_quote! {
                #(#attrs)* #vis #defaultness #sig {
                    #(#stmts)*
                }
            },
            None,
        );
    }

    let (receiver, arg_tys, fmt_args, ser_args, de_args) = map_args(self_ty, sig);
    let pub_arg_tys: Vec<TokenStream2> = arg_tys.iter().map(|ty| quote! { pub #ty }).collect();
    let def_args: Vec<Expr> = arg_tys
        .iter()
        .map(|ty| {
            parse_quote! {
                test_fuzz::runtime::TryDefault::<#ty>::default()?
            }
        })
        .collect();
    let ret_ty = match &sig.output {
        ReturnType::Type(_, ty) => self_ty.as_ref().map_or(*ty.clone(), |self_ty| {
            util::expand_self(&self_ty, &trait_path, ty)
        }),
        ReturnType::Default => parse_quote! { () },
    };

    let target_ident = &sig.ident;
    let renamed_target_ident = opts.rename.as_ref().unwrap_or(target_ident);
    let mod_ident = Ident::new(&format!("{}_fuzz", renamed_target_ident), Span::call_site());

    let (forced_write_args, mod_attr) = if opts.force {
        (
            quote! {
                #[cfg(not(test))]
                if test_fuzz::runtime::write_enabled() {
                    test_fuzz::runtime::write_args(&#mod_ident::Args(
                        #(#ser_args),*
                    ));
                }
            },
            quote! {},
        )
    } else {
        (
            quote! {},
            quote! {
                #[cfg(test)]
            },
        )
    };
    let input_args = {
        #[cfg(feature = "persistent")]
        quote! {}
        #[cfg(not(feature = "persistent"))]
        quote! {
            let mut args = test_fuzz::runtime::read_args::<Args, _>(std::io::stdin());
        }
    };
    let output_args = {
        #[cfg(feature = "persistent")]
        quote! {}
        #[cfg(not(feature = "persistent"))]
        quote! {
            args.as_ref().map(|x| {
                if test_fuzz::runtime::pretty_print_enabled() {
                    eprint!("{:#?}", x);
                } else {
                    eprint!("{:?}", x);
                };
            });
            eprintln!();
        }
    };
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
                let mut args = test_fuzz::runtime::read_args::<Args, _>(data);
                let ret = args.map(|mut args|
                    #call
                );
            });
        }
        #[cfg(not(feature = "persistent"))]
        quote! {
            let ret = args.map(|mut args|
                #call
            );
        }
    };
    let output_ret = {
        #[cfg(feature = "persistent")]
        quote! {
            // smoelius: Suppress unused variable warning.
            let _: Option<#ret_ty> = None;
        }
        #[cfg(not(feature = "persistent"))]
        quote! {
            struct Ret(#ret_ty);
            impl std::fmt::Debug for Ret {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use test_fuzz::runtime::TryDebugDefault;
                    let mut debug_tuple = fmt.debug_tuple("Ret");
                    test_fuzz::runtime::TryDebug(&self.0).apply(&mut |value| {
                        debug_tuple.field(value);
                    });
                    debug_tuple.finish()
                }
            }
            let ret = ret.map(Ret);
            ret.map(|x| {
                if test_fuzz::runtime::pretty_print_enabled() {
                    eprint!("{:#?}", x);
                } else {
                    eprint!("{:?}", x);
                };
            });
            eprintln!();
        }
    };
    (
        parse_quote! {
            #(#attrs)* #vis #defaultness #sig {
                #[cfg(test)]
                if !test_fuzz::runtime::test_fuzz_enabled() {
                    test_fuzz::runtime::write_args(&#mod_ident::Args(
                        #(#ser_args),*
                    ));
                }

                #forced_write_args

                #(#stmts)*
            }
        },
        Some(parse_quote! {
            #mod_attr
            mod #mod_ident {
                use super::*;

                #[derive(serde::Deserialize, serde::Serialize)]
                pub(super) struct Args(
                    #(#pub_arg_tys),*
                );

                impl std::fmt::Debug for Args {
                    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        use test_fuzz::runtime::TryDebugDefault;
                        let mut debug_struct = fmt.debug_struct("Args");
                        #(#fmt_args)*
                        debug_struct.finish()
                    }
                }

                #[test]
                fn default() {
                    if !test_fuzz::runtime::test_fuzz_enabled() {
                        use test_fuzz::runtime::TryDefaultDefault;
                        let args = (|| -> Option<#mod_ident::Args> {
                            Some(#mod_ident::Args(
                                #(#def_args),*
                            ))
                        })();
                        args.map(|args| test_fuzz::runtime::write_args(&args));
                    }
                }

                #[test]
                fn entry() {
                    // smoelius: Do not set the panic hook when replaying. Leave cargo test's panic
                    // hook in place.
                    if test_fuzz::runtime::test_fuzz_enabled() {
                        if test_fuzz::runtime::display_enabled()
                            || test_fuzz::runtime::replay_enabled()
                        {
                            #input_args
                            if test_fuzz::runtime::display_enabled() {
                                #output_args
                            }
                            if test_fuzz::runtime::replay_enabled() {
                                #call_with_deserialized_arguments
                                #output_ret
                            }
                        } else {
                            std::panic::set_hook(std::boxed::Box::new(|_| std::process::abort()));
                            #input_args
                            #call_with_deserialized_arguments
                            let _ = std::panic::take_hook();
                        }
                    }
                }
            }
        }),
    )
}

fn map_args(
    self_ty: &Option<Type>,
    sig: &Signature,
) -> (bool, Vec<Type>, Vec<Stmt>, Vec<Expr>, Vec<Expr>) {
    unzip_n!(5);

    let (receiver, ty, fmt, ser, de): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = sig
        .inputs
        .iter()
        .enumerate()
        .map(map_arg(self_ty))
        .unzip_n();

    let receiver = receiver.first().map_or(false, |&x| x);

    (receiver, ty, fmt, ser, de)
}

fn map_arg(self_ty: &Option<Type>) -> impl Fn((usize, &FnArg)) -> (bool, Type, Stmt, Expr, Expr) {
    let self_ty = self_ty.clone();
    move |(i, arg)| {
        let i = Literal::usize_unsuffixed(i);
        match arg {
            FnArg::Receiver(_) => (
                true,
                parse_quote! { #self_ty },
                parse_quote! {
                    test_fuzz::runtime::TryDebug(&self.#i).apply(&mut |value| {
                        debug_struct.field("self", value);
                    });
                },
                parse_quote! { self.clone() },
                parse_quote! { args.#i },
            ),
            FnArg::Typed(pat_ty) => {
                let pat = &*pat_ty.pat;
                let ty = &*pat_ty.ty;
                let name = format!("{}", pat.to_token_stream());
                let fmt = parse_quote! {
                    test_fuzz::runtime::TryDebug(&self.#i).apply(&mut |value| {
                        debug_struct.field(#name, value);
                    });
                };
                let default = (
                    false,
                    parse_quote! { #ty },
                    parse_quote! { #fmt },
                    parse_quote! { #pat.clone() },
                    parse_quote! { args.#i },
                );
                match ty {
                    Type::Path(path) => map_arc_arg(&i, pat, path)
                        .map(|(ty, ser, de)| (false, ty, fmt, ser, de))
                        .unwrap_or(default),
                    Type::Reference(ty) => {
                        let (ty, ser, de) = map_ref_arg(&i, pat, ty);
                        (false, ty, fmt, ser, de)
                    }
                    _ => default,
                }
            }
        }
    }
}

fn map_arc_arg(i: &Literal, pat: &Pat, path: &TypePath) -> Option<(Type, Expr, Expr)> {
    if let Some(PathArguments::AngleBracketed(args)) =
        util::match_type_path(path, &["std", "sync", "Arc"])
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

fn map_ref_arg(i: &Literal, pat: &Pat, ty: &TypeReference) -> (Type, Expr, Expr) {
    match &*ty.elem {
        Type::Path(path) if util::match_type_path(path, &["str"]) == Some(PathArguments::None) => (
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
        _ => {
            let mutability = if ty.mutability.is_some() {
                quote! { mut }
            } else {
                quote! {}
            };
            let ty = &*ty.elem;
            (
                parse_quote! { #ty },
                parse_quote! { (*#pat).clone() },
                parse_quote! { & #mutability args.#i },
            )
        }
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
    if opts.force {
        attrs.push(quote! { force });
    }
    if let Some(rename) = &opts.rename {
        let rename_str = stringify(rename);
        attrs.push(quote! { rename = #rename_str });
    }
    if opts.skip {
        attrs.push(quote! { skip });
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

fn is_test_fuzz(attr: &Attribute) -> bool {
    attr.path
        .segments
        .iter()
        .all(|PathSegment { ident, .. }| ident == "test_fuzz")
}

fn log(tokens: &TokenStream2) {
    if cfg!(feature = "logging") {
        println!("{}", tokens);
    }
}
