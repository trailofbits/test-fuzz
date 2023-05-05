#![deny(clippy::unwrap_used)]
#![cfg_attr(feature = "__auto_concretize", feature(proc_macro_span))]

use darling::{ast::NestedMeta, FromMeta};
use internal::serde_format;
use itertools::MultiUnzip;
use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    env::var,
    io::Write,
    str::FromStr,
};
use subprocess::{Exec, Redirection};
use syn::{
    parse::Parser, parse_macro_input, parse_quote, parse_str, punctuated::Punctuated, token,
    Attribute, Block, Expr, FnArg, GenericArgument, GenericParam, Generics, Ident, ImplItem,
    ImplItemFn, ItemFn, ItemImpl, ItemMod, LifetimeParam, PatType, Path, PathArguments,
    PathSegment, Receiver, ReturnType, Signature, Stmt, Type, TypeParam, TypePath, TypeReference,
    TypeSlice, Visibility, WhereClause, WherePredicate,
};
use toolchain_find::find_installed_component;

mod auto_concretize;

#[cfg(feature = "__auto_concretize")]
mod mod_utils;

mod ord_type;
use ord_type::OrdType;

mod pat_utils;

mod type_utils;

type Conversions = BTreeMap<OrdType, (Type, bool)>;

lazy_static! {
    static ref CARGO_CRATE_NAME: String =
        var("CARGO_CRATE_NAME").expect("Could not get `CARGO_CRATE_NAME`");
}

#[derive(FromMeta)]
struct TestFuzzImplOpts {}

#[proc_macro_attribute]
pub fn test_fuzz_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args =
        NestedMeta::parse_meta_list(args.into()).expect("Could not parse attribute args");
    let _ =
        TestFuzzImplOpts::from_list(&attr_args).expect("Could not parse `test_fuzz_impl` options");

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

    let (_, _, where_clause) = generics.split_for_impl();

    // smoelius: Without the next line, you get:
    //   the trait `quote::ToTokens` is not implemented for `(std::option::Option<syn::token::Bang>, syn::Path, syn::token::For)`
    let (trait_path, trait_) = trait_.map_or((None, None), |(bang, path, for_)| {
        (Some(path.clone()), Some(quote! { #bang #path #for_ }))
    });

    let (impl_items, modules) = map_impl_items(&generics, &trait_path, &self_ty, &items);

    let result = quote! {
        #(#attrs)* #defaultness #unsafety #impl_token #generics #trait_ #self_ty #where_clause {
            #(#impl_items)*
        }

        #(#modules)*
    };
    log(&result.to_token_stream());
    result.into()
}

fn map_impl_items(
    generics: &Generics,
    trait_path: &Option<Path>,
    self_ty: &Type,
    items: &[ImplItem],
) -> (Vec<ImplItem>, Vec<ItemMod>) {
    let impl_items_modules = items
        .iter()
        .map(map_impl_item(generics, trait_path, self_ty));

    let (impl_items, modules): (Vec<_>, Vec<_>) = impl_items_modules.unzip();

    let modules = modules.into_iter().flatten().collect();

    (impl_items, modules)
}

fn map_impl_item(
    generics: &Generics,
    trait_path: &Option<Path>,
    self_ty: &Type,
) -> impl Fn(&ImplItem) -> (ImplItem, Option<ItemMod>) {
    let generics = generics.clone();
    let trait_path = trait_path.clone();
    let self_ty = self_ty.clone();
    move |impl_item| {
        if let ImplItem::Fn(impl_item_fn) = &impl_item {
            map_impl_item_fn(&generics, &trait_path, &self_ty, impl_item_fn)
        } else {
            (impl_item.clone(), None)
        }
    }
}

// smoelius: This function is slightly misnamed. The mapped item could actually be an associated
// function. I am keeping this name to be consistent with `ImplItem::Method`.
// smoelius: In `syn` 2.0, `ImplItem::Method` was renamed to `ImplItem::Fn`:
// https://github.com/dtolnay/syn/releases/tag/2.0.0
fn map_impl_item_fn(
    generics: &Generics,
    trait_path: &Option<Path>,
    self_ty: &Type,
    impl_item_fn: &ImplItemFn,
) -> (ImplItem, Option<ItemMod>) {
    let ImplItemFn {
        attrs,
        vis,
        defaultness,
        sig,
        block,
    } = &impl_item_fn;

    let mut attrs = attrs.clone();

    attrs.iter().position(is_test_fuzz).map_or_else(
        || (parse_quote!( #impl_item_fn ), None),
        |i| {
            let attr = attrs.remove(i);
            let opts = opts_from_attr(&attr);
            let (method, module) = map_method_or_fn(
                &generics.clone(),
                trait_path,
                &Some(self_ty.clone()),
                &opts,
                &attrs,
                vis,
                defaultness,
                sig,
                block,
            );
            (parse_quote!( #method ), module)
        },
    )
}

#[derive(Clone, Debug, Default, FromMeta)]
struct TestFuzzOpts {
    #[darling(default)]
    bounds: Option<String>,
    #[darling(default)]
    concretize: Option<String>,
    #[darling(default)]
    concretize_impl: Option<String>,
    #[darling(multiple)]
    convert: Vec<String>,
    #[darling(default)]
    enable_in_production: bool,
    #[darling(default)]
    execute_with: Option<String>,
    #[darling(default)]
    no_auto_generate: bool,
    #[darling(default)]
    only_concretizations: bool,
    #[darling(default)]
    rename: Option<Ident>,
}

#[proc_macro_attribute]
pub fn test_fuzz(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args =
        NestedMeta::parse_meta_list(args.into()).expect("Could not parse attribute args");
    let opts = TestFuzzOpts::from_list(&attr_args).expect("Could not parse `test_fuzz` options");

    let item = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = &item;
    let (item, module) = map_method_or_fn(
        &Generics::default(),
        &None,
        &None,
        &opts,
        attrs,
        vis,
        &None,
        sig,
        block,
    );
    let result = quote! {
        #item
        #module
    };
    log(&result.to_token_stream());
    result.into()
}

#[allow(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref
)]
fn map_method_or_fn(
    generics: &Generics,
    trait_path: &Option<Path>,
    self_ty: &Option<Type>,
    opts: &TestFuzzOpts,
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    defaultness: &Option<token::Default>,
    sig: &Signature,
    block: &Block,
) -> (TokenStream2, Option<ItemMod>) {
    let stmts = &block.stmts;

    let mut conversions = Conversions::new();
    opts.convert.iter().for_each(|s| {
        let tokens = TokenStream::from_str(s).expect("Could not tokenize string");
        let args = Parser::parse(Punctuated::<Type, token::Comma>::parse_terminated, tokens)
            .expect("Could not parse `convert` argument");
        assert!(args.len() == 2, "Could not parse `convert` argument");
        let mut iter = args.into_iter();
        let key = iter.next().expect("Should have two `convert` arguments");
        let value = iter.next().expect("Should have two `convert` arguments");
        conversions.insert(OrdType(key), (value, false));
    });

    #[allow(unused_mut, unused_variables)]
    let mut impl_concretization_error: Option<auto_concretize::Error> = None;
    #[allow(unused_mut, unused_variables)]
    let mut concretization_error: Option<auto_concretize::Error> = None;

    let opts_concretize_impl = opts
        .concretize_impl
        .as_ref()
        .map(|s| parse_generic_arguments(s, false))
        .or_else(|| {
            #[cfg(feature = "__auto_concretize")]
            return auto_concretize::unique_impl_concretization(sig)
                .map(|s| parse_generic_arguments(&s, true))
                .map_err(|error| impl_concretization_error = Some(error))
                .ok();
            #[cfg(not(feature = "__auto_concretize"))]
            return None;
        });

    let opts_concretize = opts
        .concretize
        .as_ref()
        .map(|s| parse_generic_arguments(s, false))
        .or_else(|| {
            #[cfg(feature = "__auto_concretize")]
            return auto_concretize::unique_concretization(sig)
                .map(|s| parse_generic_arguments(&s, true))
                .map_err(|error| concretization_error = Some(error))
                .ok();
            #[cfg(not(feature = "__auto_concretize"))]
            return None;
        });

    // smoelius: Error early.
    #[cfg(fuzzing)]
    if !opts.only_concretizations {
        if is_generic(generics) && opts_concretize_impl.is_none() {
            panic!(
                "`{}` appears in a generic impl but `concretize_impl` was not specified{}",
                sig.ident.to_string(),
                impl_concretization_error.map_or("".to_owned(), |error| format!(" and {}", error))
            );
        }

        if is_generic(&sig.generics) && opts_concretize.is_none() {
            panic!(
                "`{}` is generic but `concretize` was not specified{}",
                sig.ident.to_string(),
                concretization_error.map_or("".to_owned(), |error| format!(" and {}", error))
            );
        }
    }

    let impl_ty_idents = type_idents(generics);
    let ty_idents = type_idents(&sig.generics);
    let combined_type_idents = [impl_ty_idents.clone(), ty_idents.clone()].concat();

    let impl_ty_names: Vec<Expr> = impl_ty_idents
        .iter()
        .map(|ident| parse_quote! { std::any::type_name::< #ident >() })
        .collect();
    let ty_names: Vec<Expr> = ty_idents
        .iter()
        .map(|ident| parse_quote! { std::any::type_name::< #ident >() })
        .collect();

    let combined_generics = combine_generics(generics, &sig.generics);
    let combined_generics_deserializable = restrict_to_deserialize(&combined_generics);

    let (impl_generics, ty_generics, where_clause) = combined_generics.split_for_impl();
    let (impl_generics_deserializable, _, _) = combined_generics_deserializable.split_for_impl();

    let args_where_clause: Option<WhereClause> = opts.bounds.as_ref().map(|bounds| {
        let tokens = TokenStream::from_str(bounds).expect("Could not tokenize string");
        let where_predicates = Parser::parse(
            Punctuated::<WherePredicate, token::Comma>::parse_terminated,
            tokens,
        )
        .expect("Could not parse type bounds");
        parse_quote! {
            where #where_predicates
        }
    });

    // smoelius: "Constraints don’t count as 'using' a type parameter," as explained by Daniel Keep
    // here: https://users.rust-lang.org/t/error-parameter-t-is-never-used-e0392-but-i-use-it/5673
    // So, for each type parameter `T`, add a `PhantomData<T>` member to `Args` to ensure that `T`
    // is used. See also: https://github.com/rust-lang/rust/issues/23246
    let phantom_tys = type_generic_phantom_types(&combined_generics);
    let phantoms: Vec<Expr> = phantom_tys
        .iter()
        .map(|_| {
            parse_quote! { std::marker::PhantomData }
        })
        .collect();

    let impl_concretization = opts_concretize_impl.as_ref().map(args_as_turbofish);
    let concretization = opts_concretize.as_ref().map(args_as_turbofish);
    let combined_concretization_base =
        combine_options(opts_concretize_impl, opts_concretize, |mut left, right| {
            left.extend(right);
            left
        });
    let combined_concretization = combined_concretization_base.as_ref().map(args_as_turbofish);
    // smoelius: The macro generates code like this:
    //  struct Ret(<Args as HasRetTy>::RetTy);
    // If `Args` has lifetime parameters, this code won't compile. Insert `'static` for each
    // parameter that is not filled.
    let combined_concretization_with_dummy_lifetimes = {
        let mut args = combined_concretization_base.unwrap_or_default();
        let n_lifetime_params = combined_generics.lifetimes().count();
        let n_lifetime_args = args
            .iter()
            .filter(|arg| matches!(arg, GenericArgument::Lifetime(..)))
            .count();
        #[allow(clippy::cast_possible_wrap)]
        let n_missing_lifetime_args =
            usize::try_from(n_lifetime_params as isize - n_lifetime_args as isize)
                .expect("n_lifetime_params < n_lifetime_args");
        let dummy_lifetime = GenericArgument::Lifetime(parse_quote! { 'static });
        args.extend(std::iter::repeat(dummy_lifetime).take(n_missing_lifetime_args));
        args_as_turbofish(&args)
    };

    let self_ty_base = self_ty.as_ref().map(type_utils::type_base);

    let (receiver, mut arg_tys, fmt_args, mut ser_args, de_args) = {
        let mut candidates = BTreeSet::new();
        let result = map_args(
            &mut conversions,
            &mut candidates,
            self_ty,
            trait_path,
            sig.inputs.iter(),
        );
        for (from, (to, used)) in conversions {
            assert!(
                used,
                r#"Conversion "{}" -> "{}" does not apply to the following candidates: {:#?}"#,
                from,
                OrdType(to),
                candidates
            );
        }
        result
    };
    arg_tys.extend_from_slice(&phantom_tys);
    ser_args.extend_from_slice(&phantoms);
    let pub_arg_tys: Vec<TokenStream2> = arg_tys.iter().map(|ty| quote! { pub #ty }).collect();
    let args_is: Vec<Expr> = arg_tys
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let i = Literal::usize_unsuffixed(i);
            parse_quote! { args . #i }
        })
        .collect();
    let autos: Vec<Expr> = arg_tys
        .iter()
        .map(|ty| {
            parse_quote! {
                test_fuzz::runtime::auto!( #ty ).collect::<Vec<_>>()
            }
        })
        .collect();
    let args_from_autos = args_from_autos(&autos);
    let ret_ty = match &sig.output {
        ReturnType::Type(_, ty) => self_ty.as_ref().map_or(*ty.clone(), |self_ty| {
            type_utils::expand_self(self_ty, trait_path, ty)
        }),
        ReturnType::Default => parse_quote! { () },
    };

    let target_ident = &sig.ident;
    let renamed_target_ident = opts.rename.as_ref().unwrap_or(target_ident);
    let mod_ident = Ident::new(&format!("{renamed_target_ident}_fuzz"), Span::call_site());

    // smoelius: This is a hack. When `only_concretizations` is specified, the user should not have
    // to also specify trait bounds. But `Args` is used to get the module path at runtime via
    // `type_name`. So when `only_concretizations` is specified, `Args` gets an empty declaration.
    let empty_generics = Generics {
        lt_token: None,
        params: parse_quote! {},
        gt_token: None,
        where_clause: None,
    };
    let (_, empty_ty_generics, _) = empty_generics.split_for_impl();
    let (ty_generics_as_turbofish, struct_args) = if opts.only_concretizations {
        (
            empty_ty_generics.as_turbofish(),
            quote! {
                pub(super) struct Args;
            },
        )
    } else {
        (
            ty_generics.as_turbofish(),
            quote! {
                pub(super) struct Args #ty_generics (
                    #(#pub_arg_tys),*
                ) #args_where_clause;
            },
        )
    };

    let serde_format = serde_format();
    let write_concretizations = quote! {
        let impl_concretization = [
            #(#impl_ty_names),*
        ];
        let concretization = [
            #(#ty_names),*
        ];
        test_fuzz::runtime::write_impl_concretization::< #mod_ident :: Args #ty_generics_as_turbofish>(&impl_concretization);
        test_fuzz::runtime::write_concretization::< #mod_ident :: Args #ty_generics_as_turbofish>(&concretization);
    };
    let write_args = if opts.only_concretizations {
        quote! {}
    } else {
        quote! {
            #mod_ident :: write_args::< #(#combined_type_idents),* >(#mod_ident :: Args(
                #(#ser_args),*
            ));
        }
    };
    let write_concretizations_and_args = quote! {
        #[cfg(test)]
        if !test_fuzz::runtime::test_fuzz_enabled() {
            #write_concretizations
            #write_args
        }
    };
    let (in_production_write_concretizations_and_args, mod_attr) = if opts.enable_in_production {
        (
            quote! {
                #[cfg(not(test))]
                if test_fuzz::runtime::write_enabled() {
                    #write_concretizations
                    #write_args
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
    let auto_generate = if opts.no_auto_generate {
        quote! {}
    } else {
        quote! {
            #[test]
            fn auto_generate() {
                Args #combined_concretization :: auto_generate();
            }
        }
    };
    let input_args = {
        #[cfg(feature = "__persistent")]
        quote! {}
        #[cfg(not(feature = "__persistent"))]
        quote! {
            let mut args = UsingReader::<_>::read_args #combined_concretization (std::io::stdin());
        }
    };
    let output_args = {
        #[cfg(feature = "__persistent")]
        quote! {}
        #[cfg(not(feature = "__persistent"))]
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
    let args_ret_ty: Type = parse_quote! {
        <Args #combined_concretization_with_dummy_lifetimes as HasRetTy>::RetTy
    };
    let call: Expr = if receiver {
        let mut de_args = de_args.iter();
        let self_arg = de_args
            .next()
            .expect("Should have at least one deserialized argument");
        parse_quote! {
            ( #self_arg ). #target_ident #concretization (
                #(#de_args),*
            )
        }
    } else if let Some(self_ty_base) = self_ty_base {
        parse_quote! {
            #self_ty_base #impl_concretization :: #target_ident #concretization (
                #(#de_args),*
            )
        }
    } else {
        parse_quote! {
            super :: #target_ident #concretization (
                #(#de_args),*
            )
        }
    };
    let call_in_environment = if let Some(s) = &opts.execute_with {
        let execute_with: Expr = parse_str(s).expect("Could not parse `execute_with` argument");
        parse_quote! {
            #execute_with (|| #call)
        }
    } else {
        call
    };
    let call_in_environment_with_deserialized_arguments = {
        #[cfg(feature = "__persistent")]
        quote! {
            test_fuzz::afl::fuzz!(|data: &[u8]| {
                let mut args = UsingReader::<_>::read_args #combined_concretization (data);
                let ret: Option< #args_ret_ty > = args.map(|mut args|
                    #call_in_environment
                );
            });
        }
        #[cfg(not(feature = "__persistent"))]
        quote! {
            let ret: Option< #args_ret_ty > = args.map(|mut args|
                #call_in_environment
            );
        }
    };
    let output_ret = {
        #[cfg(feature = "__persistent")]
        quote! {
            // smoelius: Suppress unused variable warning.
            let _: Option< #args_ret_ty > = None;
        }
        #[cfg(not(feature = "__persistent"))]
        quote! {
            struct Ret( #args_ret_ty );
            impl std::fmt::Debug for Ret {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use test_fuzz::runtime::TryDebugFallback;
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
    let mod_items = if opts.only_concretizations {
        quote! {}
    } else {
        quote! {
            // smoelius: It is tempting to want to put all of these functions under `impl Args`.
            // But `write_args` and `read args` impose different bounds on their arguments. So
            // I don't think that idea would work.
            pub(super) fn write_args #impl_generics (args: Args #ty_generics_as_turbofish) #where_clause {
                #[derive(serde::Serialize)]
                struct Args #ty_generics (
                    #(#pub_arg_tys),*
                ) #args_where_clause;
                let args = Args(
                    #(#args_is),*
                );
                test_fuzz::runtime::write_args(#serde_format, &args);
            }

            struct UsingReader<R>(R);

            impl<R: std::io::Read> UsingReader<R> {
                pub fn read_args #impl_generics_deserializable (reader: R) -> Option<Args #ty_generics_as_turbofish> #where_clause {
                    #[derive(serde::Deserialize)]
                    struct Args #ty_generics (
                        #(#pub_arg_tys),*
                    ) #args_where_clause;
                    let args = test_fuzz::runtime::read_args::<Args #ty_generics_as_turbofish, _>(#serde_format, reader);
                    args.map(|args| #mod_ident :: Args(
                        #(#args_is),*
                    ))
                }
            }

            impl #impl_generics std::fmt::Debug for Args #ty_generics #where_clause {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use test_fuzz::runtime::TryDebugFallback;
                    let mut debug_struct = fmt.debug_struct("Args");
                    #(#fmt_args)*
                    debug_struct.finish()
                }
            }

            // smoelius: Inherent associated types are unstable:
            // https://github.com/rust-lang/rust/issues/8995
            trait HasRetTy {
                type RetTy;
            }

            impl #impl_generics HasRetTy for Args #ty_generics #where_clause {
                type RetTy = #ret_ty;
            }
        }
    };
    // smoelius: The `Args`' implementation and the `auto_generate` test won't compile without
    // concretizations.
    //   Also, cargo-test-fuzz finds targets by looking for tests that end with `_fuzz::entry`. So
    // create such a test regardless. If say `only_concretizations` was specified, then give the
    // test an empty body.
    let (concretization_dependent_mod_items, entry_stmts) = if opts.only_concretizations
        || (generics.type_params().next().is_some() && impl_concretization.is_none())
        || (sig.generics.type_params().next().is_some() && concretization.is_none())
    {
        (quote! {}, quote! {})
    } else {
        (
            quote! {
                impl #impl_generics Args #ty_generics #where_clause {
                    // smoelius: `#autos` could refer to type parameters. Expanding it in a method
                    // definition like this ensures such type parameters resolve.
                    fn auto_generate() {
                        if !test_fuzz::runtime::test_fuzz_enabled() {
                            let autos = ( #(#autos,)* );
                            for args in #args_from_autos {
                                write_args(args);
                            }
                        }
                    }

                    fn entry() {
                        // smoelius: Do not set the panic hook when replaying. Leave cargo test's
                        // panic hook in place.
                        if test_fuzz::runtime::test_fuzz_enabled() {
                            if test_fuzz::runtime::display_enabled()
                                || test_fuzz::runtime::replay_enabled()
                            {
                                #input_args
                                if test_fuzz::runtime::display_enabled() {
                                    #output_args
                                }
                                if test_fuzz::runtime::replay_enabled() {
                                    #call_in_environment_with_deserialized_arguments
                                    #output_ret
                                }
                            } else {
                                std::panic::set_hook(std::boxed::Box::new(|_| std::process::abort()));
                                #input_args
                                #call_in_environment_with_deserialized_arguments
                                let _ = std::panic::take_hook();
                            }
                        }
                    }
                }

                #auto_generate
            },
            quote! {
                Args #combined_concretization :: entry();
            },
        )
    };
    (
        parse_quote! {
            #(#attrs)* #vis #defaultness #sig {
                #write_concretizations_and_args

                #in_production_write_concretizations_and_args

                #(#stmts)*
            }
        },
        Some(parse_quote! {
            #mod_attr
            mod #mod_ident {
                use super::*;

                #struct_args

                #mod_items

                #concretization_dependent_mod_items

                #[test]
                fn entry() {
                    #entry_stmts
                }
            }
        }),
    )
}

fn map_args<'a, I>(
    conversions: &mut Conversions,
    candidates: &mut BTreeSet<OrdType>,
    self_ty: &Option<Type>,
    trait_path: &Option<Path>,
    inputs: I,
) -> (bool, Vec<Type>, Vec<Stmt>, Vec<Expr>, Vec<Expr>)
where
    I: Iterator<Item = &'a FnArg>,
{
    let (receiver, ty, fmt, ser, de): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = inputs
        .enumerate()
        .map(map_arg(conversions, candidates, self_ty, trait_path))
        .multiunzip();

    let receiver = receiver.first().map_or(false, |&x| x);

    (receiver, ty, fmt, ser, de)
}

fn map_arg<'a>(
    conversions: &'a mut Conversions,
    candidates: &'a mut BTreeSet<OrdType>,
    self_ty: &Option<Type>,
    trait_path: &Option<Path>,
) -> impl FnMut((usize, &FnArg)) -> (bool, Type, Stmt, Expr, Expr) + 'a {
    let self_ty = self_ty.clone();
    let trait_path = trait_path.clone();
    move |(i, arg)| {
        let i = Literal::usize_unsuffixed(i);
        let (receiver, expr, ty, fmt) = match arg {
            FnArg::Receiver(Receiver {
                reference,
                mutability,
                ..
            }) => {
                let expr = parse_quote! { self };
                let reference = reference
                    .as_ref()
                    .map(|(and, lifetime)| quote! { #and #lifetime });
                let ty = parse_quote! { #reference #mutability #self_ty };
                let fmt = parse_quote! {
                    test_fuzz::runtime::TryDebug(&self.#i).apply(&mut |value| {
                        debug_struct.field("self", value);
                    });
                };
                (true, expr, ty, fmt)
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident = match *pat_utils::pat_idents(pat).as_slice() {
                    [] => Ident::new("_", Span::call_site()),
                    [ident] => ident.clone(),
                    _ => panic!("Unexpected pattern: {}", pat.to_token_stream()),
                };
                let expr = parse_quote! { #ident };
                let ty = self_ty.as_ref().map_or(*ty.clone(), |self_ty| {
                    type_utils::expand_self(self_ty, &trait_path, ty)
                });
                let name = ident.to_string();
                let fmt = parse_quote! {
                    test_fuzz::runtime::TryDebug(&self.#i).apply(&mut |value| {
                        debug_struct.field(#name, value);
                    });
                };
                (false, expr, ty, fmt)
            }
        };
        let (ty, ser, de) = map_typed_arg(conversions, candidates, &i, &expr, &ty);
        (receiver, ty, fmt, ser, de)
    }
}

fn map_typed_arg(
    conversions: &mut Conversions,
    candidates: &mut BTreeSet<OrdType>,
    i: &Literal,
    expr: &Expr,
    ty: &Type,
) -> (Type, Expr, Expr) {
    candidates.insert(OrdType(ty.clone()));
    if let Some((arg_ty, used)) = conversions.get_mut(&OrdType(ty.clone())) {
        *used = true;
        return (
            parse_quote! { #arg_ty },
            parse_quote! { <#arg_ty as test_fuzz::FromRef::<#ty>>::from_ref( & #expr ) },
            parse_quote! { <_ as test_fuzz::Into::<_>>::into(args.#i) },
        );
    }
    match &ty {
        Type::Path(path) => map_path_arg(conversions, candidates, i, expr, path),
        Type::Reference(ty) => map_ref_arg(conversions, candidates, i, expr, ty),
        _ => (
            parse_quote! { #ty },
            parse_quote! { #expr.clone() },
            parse_quote! { args.#i },
        ),
    }
}

fn map_path_arg(
    _conversions: &mut Conversions,
    _candidates: &mut BTreeSet<OrdType>,
    i: &Literal,
    expr: &Expr,
    path: &TypePath,
) -> (Type, Expr, Expr) {
    (
        parse_quote! { #path },
        parse_quote! { #expr.clone() },
        parse_quote! { args.#i },
    )
}

fn map_ref_arg(
    conversions: &mut Conversions,
    candidates: &mut BTreeSet<OrdType>,
    i: &Literal,
    expr: &Expr,
    ty: &TypeReference,
) -> (Type, Expr, Expr) {
    let mutability = if ty.mutability.is_some() {
        quote! { mut }
    } else {
        quote! {}
    };
    let ty = &*ty.elem;
    match ty {
        Type::Path(path) => {
            if type_utils::match_type_path(path, &["str"]) == Some(PathArguments::None) {
                (
                    parse_quote! { String },
                    parse_quote! { #expr.to_owned() },
                    parse_quote! { args.#i.as_str() },
                )
            } else {
                let expr = parse_quote! { (*#expr) };
                let (ty, ser, de) = map_path_arg(conversions, candidates, i, &expr, path);
                (ty, ser, parse_quote! { & #mutability #de })
            }
        }
        Type::Slice(TypeSlice { elem, .. }) => (
            parse_quote! { Vec<#elem> },
            parse_quote! { #expr.to_vec() },
            parse_quote! { args.#i.as_slice() },
        ),
        _ => {
            let expr = parse_quote! { (*#expr) };
            let (ty, ser, de) = map_typed_arg(conversions, candidates, i, &expr, ty);
            (ty, ser, parse_quote! { & #mutability #de })
        }
    }
}

fn opts_from_attr(attr: &Attribute) -> TestFuzzOpts {
    attr.parse_args::<TokenStream2>()
        .map_or(TestFuzzOpts::default(), |tokens| {
            let attr_args =
                NestedMeta::parse_meta_list(tokens).expect("Could not parse attribute args");
            TestFuzzOpts::from_list(&attr_args).expect("Could not parse `test_fuzz` options")
        })
}

fn is_test_fuzz(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .iter()
        .all(|PathSegment { ident, .. }| ident == "test_fuzz")
}

fn parse_generic_arguments(
    s: &str,
    collapse_crate: bool,
) -> Punctuated<GenericArgument, token::Comma> {
    let tokens = TokenStream::from_str(s).expect("Could not tokenize string");
    let args = Parser::parse(
        Punctuated::<GenericArgument, token::Comma>::parse_terminated,
        tokens,
    )
    .expect("Could not parse generic arguments");
    if collapse_crate {
        args.into_iter()
            .map(|mut arg| {
                if let GenericArgument::Type(ref mut ty) = arg {
                    *ty = type_utils::collapse_crate(ty);
                }
                arg
            })
            .collect()
    } else {
        args
    }
}

#[cfg(fuzzing)]
fn is_generic(generics: &Generics) -> bool {
    generics
        .params
        .iter()
        .filter(|param| !matches!(param, GenericParam::Lifetime(_)))
        .next()
        .is_some()
}

fn type_idents(generics: &Generics) -> Vec<Ident> {
    generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(ty_param) = param {
                Some(ty_param.ident.clone())
            } else {
                None
            }
        })
        .collect()
}

fn combine_generics(left: &Generics, right: &Generics) -> Generics {
    let mut generics = left.clone();
    generics.params.extend(right.params.clone());
    generics.where_clause = combine_options(
        generics.where_clause,
        right.where_clause.clone(),
        |mut left, right| {
            left.predicates.extend(right.predicates);
            left
        },
    );
    generics
}

// smoelius: Is there a better name for this operation? The closest thing I've found is the `<|>`
// operation in Haskell's `Alternative` class (thanks, @incertia):
// https://en.wikibooks.org/wiki/Haskell/Alternative_and_MonadPlus
// ... (<|>) is a binary function which combines two computations.
//                                      ^^^^^^^^

fn combine_options<T, F>(x: Option<T>, y: Option<T>, f: F) -> Option<T>
where
    F: FnOnce(T, T) -> T,
{
    match (x, y) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (x, None) => x,
        (None, y) => y,
    }
}

fn restrict_to_deserialize(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    generics.params.iter_mut().for_each(|param| {
        if let GenericParam::Type(ty_param) = param {
            ty_param
                .bounds
                .push(parse_quote! { serde::de::DeserializeOwned });
        }
    });
    generics
}

fn type_generic_phantom_types(generics: &Generics) -> Vec<Type> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(TypeParam { ident, .. }) => {
                Some(parse_quote! { std::marker::PhantomData< #ident > })
            }
            GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => {
                Some(parse_quote! { std::marker::PhantomData< & #lifetime () > })
            }
            GenericParam::Const(_) => None,
        })
        .collect()
}

fn args_as_turbofish(args: &Punctuated<GenericArgument, token::Comma>) -> TokenStream2 {
    quote! {
        ::<#args>
    }
}

// smoelius: The current strategy for combining auto-generated values is a kind of "round robin."
// The strategy ensures that each auto-generated value gets into at least one `Arg` value.
fn args_from_autos(autos: &[Expr]) -> Expr {
    let lens: Vec<Expr> = (0..autos.len())
        .map(|i| {
            let i = Literal::usize_unsuffixed(i);
            parse_quote! {
                autos.#i.len()
            }
        })
        .collect();
    let args: Vec<Expr> = (0..autos.len())
        .map(|i| {
            let i = Literal::usize_unsuffixed(i);
            parse_quote! {
                autos.#i[(i + #i) % lens[#i]].clone()
            }
        })
        .collect();
    parse_quote! {{
        let lens = [ #(#lens),* ];
        let max = if lens.iter().copied().min().unwrap_or(1) > 0 {
            lens.iter().copied().max().unwrap_or(1)
        } else {
            0
        };
        (0..max).map(move |i|
            Args( #(#args),* )
        )
    }}
}

fn log(tokens: &TokenStream2) {
    if log_enabled() {
        find_installed_component("rustfmt").map_or_else(
            || {
                println!("{tokens}");
            },
            |rustfmt| {
                let mut popen = Exec::cmd(rustfmt)
                    .stdin(Redirection::Pipe)
                    .popen()
                    .expect("`popen` failed");
                let mut stdin = popen
                    .stdin
                    .take()
                    .expect("Could not take `rustfmt`'s standard input");
                write!(stdin, "{tokens}").expect("Could not write to `rustfmt`'s standard input");
                drop(stdin);
                let status = popen.wait().expect("`wait` failed");
                assert!(status.success(), "`rustfmt` failed");
            },
        );
    }
}

fn log_enabled() -> bool {
    option_env!("TEST_FUZZ_LOG").map_or(false, |value| value == "1" || value == *CARGO_CRATE_NAME)
}
