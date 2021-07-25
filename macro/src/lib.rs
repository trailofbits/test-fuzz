#![allow(clippy::default_trait_access)]
#![deny(clippy::unwrap_used)]

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::{io::Write, str::FromStr};
use subprocess::{Exec, Redirection};
use syn::{
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, token, Attribute,
    AttributeArgs, Block, Expr, FnArg, GenericArgument, GenericMethodArgument, GenericParam,
    Generics, Ident, ImplItem, ImplItemMethod, ItemFn, ItemImpl, ItemMod, Pat, Path, PathArguments,
    PathSegment, ReturnType, Signature, Stmt, Type, TypePath, TypeReference, Visibility,
};
use toolchain_find::find_installed_component;
use unzip_n::unzip_n;

mod util;

#[derive(FromMeta)]
struct TestFuzzImplOpts {}

#[proc_macro_attribute]
pub fn test_fuzz_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
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

    // smoelius: Without the next line, you get:
    //   the trait `quote::ToTokens` is not implemented for `(std::option::Option<syn::token::Bang>, syn::Path, syn::token::For)`
    let (trait_path, trait_) = trait_.map_or((None, None), |(bang, path, for_)| {
        (Some(path.clone()), Some(quote! { #bang #path #for_ }))
    });

    let (impl_items, modules) = map_impl_items(&generics, &trait_path, &*self_ty, &items);

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
        if let ImplItem::Method(method) = &impl_item {
            map_method(&generics, &trait_path, &self_ty, method)
        } else {
            (impl_item.clone(), None)
        }
    }
}

fn map_method(
    generics: &Generics,
    trait_path: &Option<Path>,
    self_ty: &Type,
    method: &ImplItemMethod,
) -> (ImplItem, Option<ItemMod>) {
    let ImplItemMethod {
        attrs,
        vis,
        defaultness,
        sig,
        block,
    } = &method;

    let mut attrs = attrs.clone();

    if let Some(i) = attrs.iter().position(is_test_fuzz) {
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
    } else {
        (parse_quote!( #method ), None)
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
struct TestFuzzOpts {
    #[darling(default)]
    concretize: Option<String>,
    #[darling(default)]
    concretize_impl: Option<String>,
    #[darling(default)]
    enable_in_production: bool,
    #[darling(default)]
    no_auto: bool,
    #[darling(default)]
    only_concretizations: bool,
    #[darling(default)]
    rename: Option<Ident>,
}

#[proc_macro_attribute]
pub fn test_fuzz(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
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
    let opts_concretize = opts
        .concretize
        .as_ref()
        .map(|s| parse_generic_method_arguments(s));
    let opts_concretize_impl = opts
        .concretize_impl
        .as_ref()
        .map(|s| parse_generic_method_arguments(s));

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

    let ty_generics_as_turbofish = ty_generics.as_turbofish();

    let impl_concretization = opts_concretize_impl.as_ref().map(args_as_turbofish);
    let concretization = opts_concretize.as_ref().map(args_as_turbofish);
    let combined_concretization =
        combine_options(opts_concretize_impl, opts_concretize, |mut left, right| {
            left.extend(right);
            left
        })
        .as_ref()
        .map(args_as_turbofish);

    let self_ty_base = self_ty.as_ref().map(type_base);

    let (receiver, mut arg_tys, fmt_args, mut ser_args, de_args) =
        map_args(self_ty, trait_path, sig.inputs.iter());
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
            util::expand_self(self_ty, trait_path, ty)
        }),
        ReturnType::Default => parse_quote! { () },
    };

    let target_ident = &sig.ident;
    let renamed_target_ident = opts.rename.as_ref().unwrap_or(target_ident);
    let mod_ident = Ident::new(&format!("{}_fuzz", renamed_target_ident), Span::call_site());

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
    let auto = if opts.no_auto {
        quote! {}
    } else {
        quote! {
            // smoelius: `#autos` could refer to type parameters. Expanding it in a method
            // definition like this ensures such type parameters resolve.
            impl #impl_generics Args #ty_generics #where_clause {
                fn write_auto() {
                    let autos = ( #(#autos,)* );
                    for args in #args_from_autos {
                        write_args(args);
                    }
                }
            }

            #[test]
            fn auto() {
                if !test_fuzz::runtime::test_fuzz_enabled() {
                    Args #combined_concretization :: write_auto();
                }
            }
        }
    };
    let input_args = {
        #[cfg(feature = "persistent")]
        quote! {}
        #[cfg(not(feature = "persistent"))]
        quote! {
            let mut args = UsingReader::<_>::read_args #combined_concretization (std::io::stdin());
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
            .expect("Should have at least one deserialized argument");
        parse_quote! {
            #self_arg . #target_ident #concretization (
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
    let call_with_deserialized_arguments = {
        #[cfg(feature = "persistent")]
        quote! {
            test_fuzz::afl::fuzz!(|data: &[u8]| {
                let mut args = UsingReader::<_>::read_args #combined_concretization (data);
                let ret: Option< #ret_ty > = args.map(|mut args|
                    #call
                );
            });
        }
        #[cfg(not(feature = "persistent"))]
        quote! {
            let ret: Option< #ret_ty > = args.map(|mut args|
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
    let (mod_items, entry_stmts) = if opts.only_concretizations {
        (quote! {}, quote! {})
    } else {
        (
            quote! {
                pub(super) fn write_args #impl_generics (args: Args #ty_generics_as_turbofish) #where_clause {
                    #[derive(serde::Serialize)]
                    struct Args #ty_generics (
                        #(#pub_arg_tys),*
                    );
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
                        );
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

                #auto
            },
            quote! {
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

                pub(super) struct Args #ty_generics (
                    #(#pub_arg_tys),*
                );

                #mod_items

                #[test]
                fn entry() {
                    #entry_stmts
                }
            }
        }),
    )
}

fn map_args<'a, I>(
    self_ty: &Option<Type>,
    trait_path: &Option<Path>,
    inputs: I,
) -> (bool, Vec<Type>, Vec<Stmt>, Vec<Expr>, Vec<Expr>)
where
    I: Iterator<Item = &'a FnArg>,
{
    unzip_n!(5);

    let (receiver, ty, fmt, ser, de): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = inputs
        .enumerate()
        .map(map_arg(self_ty, trait_path))
        .unzip_n();

    let receiver = receiver.first().map_or(false, |&x| x);

    (receiver, ty, fmt, ser, de)
}

fn map_arg(
    self_ty: &Option<Type>,
    trait_path: &Option<Path>,
) -> impl Fn((usize, &FnArg)) -> (bool, Type, Stmt, Expr, Expr) {
    let self_ty = self_ty.clone();
    let trait_path = trait_path.clone();
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
                let ty = self_ty.as_ref().map_or(*pat_ty.ty.clone(), |self_ty| {
                    util::expand_self(self_ty, &trait_path, &*pat_ty.ty)
                });
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
                match &ty {
                    Type::Path(path) => map_arc_arg(&i, pat, path)
                        .map_or(default, |(ty, ser, de)| (false, ty, fmt, ser, de)),
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
            let attr_args = parse_macro_input::parse::<AttributeArgs>(tokens.into())
                .expect("Could not parse attribute args");
            TestFuzzOpts::from_list(&attr_args).expect("Could not parse `test_fuzz` options")
        })
}

fn is_test_fuzz(attr: &Attribute) -> bool {
    attr.path
        .segments
        .iter()
        .all(|PathSegment { ident, .. }| ident == "test_fuzz")
}

fn parse_generic_method_arguments(s: &str) -> Punctuated<GenericMethodArgument, token::Comma> {
    let tokens = TokenStream::from_str(s).expect("Could not tokenize string");
    let args = Parser::parse(Punctuated::<Type, token::Comma>::parse_terminated, tokens)
        .expect("Could not parse generic method arguments");
    args.into_iter().map(GenericMethodArgument::Type).collect()
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
        .filter_map(|param| {
            if let GenericParam::Type(ty_param) = param {
                let ident = &ty_param.ident;
                Some(parse_quote! { std::marker::PhantomData< #ident > })
            } else {
                None
            }
        })
        .collect()
}

fn args_as_turbofish(args: &Punctuated<GenericMethodArgument, token::Comma>) -> TokenStream2 {
    quote! {
        ::<#args>
    }
}

fn type_base(ty: &Type) -> Type {
    let mut ty = ty.clone();

    if let Type::Path(ref mut path) = ty {
        if let Some(segment) = path.path.segments.last_mut() {
            let ident = &segment.ident;
            *segment = parse_quote! { #ident };
        }
    }

    ty
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
        let max = if std::array::IntoIter::new(lens).min().unwrap_or(1) > 0 {
            std::array::IntoIter::new(lens).max().unwrap_or(1)
        } else {
            0
        };
        (0..max).map(move |i|
            Args( #(#args),* )
        )
    }}
}

fn serde_format() -> Expr {
    let mut formats = vec![];
    #[cfg(feature = "serde_bincode")]
    formats.push(parse_quote! { test_fuzz::runtime::SerdeFormat::Bincode });
    #[cfg(feature = "serde_cbor")]
    formats.push(parse_quote! { test_fuzz::runtime::SerdeFormat::Cbor });
    assert!(
        formats.len() <= 1,
        "Multiple serde formats selected: {:?}",
        formats
    );
    formats.pop().expect("No serde format selected")
}

fn log(tokens: &TokenStream2) {
    if log_enabled() {
        if let Some(rustfmt) = find_installed_component("rustfmt") {
            let mut popen = Exec::cmd(rustfmt)
                .stdin(Redirection::Pipe)
                .popen()
                .expect("`popen` failed");
            let mut stdin = popen
                .stdin
                .take()
                .expect("Could not take `rustfmt`'s standard input");
            write!(stdin, "{}", tokens).expect("Could not write to `rustfmt`'s standard input");
            let status = popen.wait().expect("`wait` failed");
            assert!(status.success(), "`rustfmt` failed");
        } else {
            println!("{}", tokens);
        }
    }
}

fn log_enabled() -> bool {
    option_env!("TEST_FUZZ_LOG").map_or(false, |value| value != "0")
}
