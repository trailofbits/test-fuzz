use proc_macro2::{Punct, Spacing, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{
    parse_quote,
    visit::{visit_path_arguments, Visit},
    visit_mut::{visit_type_mut, VisitMut},
    GenericArgument, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

pub fn map_path_generic_params(map: &BTreeMap<&Ident, &GenericArgument>, path: &Path) -> Path {
    let mut path = path.clone();
    let mut visitor = GenericParamVisitor { map };
    visitor.visit_path_mut(&mut path);
    path
}

pub fn map_type_generic_params(map: &BTreeMap<&Ident, &GenericArgument>, ty: &Type) -> Type {
    let mut ty = ty.clone();
    let mut visitor = GenericParamVisitor { map };
    visitor.visit_type_mut(&mut ty);
    ty
}

struct GenericParamVisitor<'a> {
    map: &'a BTreeMap<&'a Ident, &'a GenericArgument>,
}

impl VisitMut for GenericParamVisitor<'_> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(TypePath { qself: None, path }) = ty {
            if let Some(ident) = path.get_ident() {
                if let Some(generic_arg) = self.map.get(ident) {
                    let GenericArgument::Type(ty_new) = generic_arg else {
                        panic!(
                            "Unexpected generic argument: {}",
                            generic_arg.to_token_stream()
                        );
                    };
                    *ty = ty_new.clone();
                    return;
                }
            }
        }
        visit_type_mut(self, ty);
    }
}

pub fn path_as_turbofish(path: &Path) -> TokenStream {
    let tokens = path.to_token_stream().into_iter().collect::<Vec<_>>();
    let mut visitor = TurbofishVisitor { tokens };
    visitor.visit_path(path);
    visitor.tokens.into_iter().collect()
}

pub fn type_as_turbofish(ty: &Type) -> TokenStream {
    let tokens = ty.to_token_stream().into_iter().collect::<Vec<_>>();
    let mut visitor = TurbofishVisitor { tokens };
    visitor.visit_type(ty);
    visitor.tokens.into_iter().collect()
}

struct TurbofishVisitor {
    tokens: Vec<TokenTree>,
}

impl Visit<'_> for TurbofishVisitor {
    fn visit_path_arguments(&mut self, path_args: &PathArguments) {
        if !path_args.is_none() {
            let mut visitor_token_strings = token_strings(&self.tokens);
            let path_args_tokens = path_args.to_token_stream().into_iter().collect::<Vec<_>>();
            let path_args_token_strings = token_strings(&path_args_tokens);
            let n = path_args_tokens.len();
            let mut i: usize = 0;
            while i + n <= self.tokens.len() {
                if visitor_token_strings[i..i + n] == path_args_token_strings
                    && (i < 2 || visitor_token_strings[i - 2..i] != [":", ":"])
                {
                    self.tokens = [
                        &self.tokens[..i],
                        &[
                            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        ],
                        &self.tokens[i..],
                    ]
                    .concat();
                    visitor_token_strings = token_strings(&self.tokens);
                    i += 2;
                }
                i += 1;
            }
        }
        visit_path_arguments(self, path_args);
    }
}

fn token_strings(tokens: &[TokenTree]) -> Vec<String> {
    tokens.iter().map(ToString::to_string).collect::<Vec<_>>()
}

pub fn expand_self(trait_path: Option<&Path>, self_ty: &Type, ty: &Type) -> Type {
    let mut ty = ty.clone();
    let mut visitor = ExpandSelfVisitor {
        trait_path,
        self_ty,
    };
    visitor.visit_type_mut(&mut ty);
    ty
}

struct ExpandSelfVisitor<'a> {
    trait_path: Option<&'a Path>,
    self_ty: &'a Type,
}

impl VisitMut for ExpandSelfVisitor<'_> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        // smoelius: Rewrite this using if-let-guards once the feature is stable.
        // https://rust-lang.github.io/rfcs/2294-if-let-guard.html
        if let Type::Path(path) = ty {
            if match_type_path(path, &["Self"]) == Some(PathArguments::None) {
                *ty = self.self_ty.clone();
                return;
            } else if path.qself.is_none()
                && path
                    .path
                    .segments
                    .first()
                    .map_or(false, |segment| segment.ident == "Self")
            {
                let segments = path.path.segments.iter().skip(1).collect::<Vec<_>>();
                let self_ty = self.self_ty;
                let trait_path = self
                    .trait_path
                    .as_ref()
                    .expect("`trait_path` should be set");
                *ty = parse_quote! { < #self_ty as #trait_path > :: #(#segments)::* };
                return;
            }
        }
        visit_type_mut(self, ty);
    }
}

pub fn match_type_path(path: &TypePath, other: &[&str]) -> Option<PathArguments> {
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

pub fn type_base(ty: &Type) -> Option<&Ident> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return Some(&segment.ident);
        }
    }

    None
}
