use quote::ToTokens;
use std::cmp::Ordering;
use syn::Type;

#[derive(Clone)]
pub struct OrdType(pub Type);

impl std::fmt::Display for OrdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_token_stream().fmt(f)
    }
}

impl std::fmt::Debug for OrdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}

impl Ord for OrdType {
    fn cmp(&self, other: &Self) -> Ordering {
        <String as Ord>::cmp(&self.to_string(), &other.to_string())
    }
}

impl PartialOrd for OrdType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <String as PartialOrd>::partial_cmp(&self.to_string(), &other.to_string())
    }
}

impl Eq for OrdType {}

impl PartialEq for OrdType {
    fn eq(&self, other: &Self) -> bool {
        <String as PartialEq>::eq(&self.to_string(), &other.to_string())
    }
}
