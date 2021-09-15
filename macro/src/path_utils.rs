use quote::ToTokens;
use std::cmp::Ordering;
use syn::Path;

#[derive(Clone)]
pub struct OrdPath(pub Path);

impl Ord for OrdPath {
    fn cmp(&self, other: &Self) -> Ordering {
        <String as Ord>::cmp(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}

impl PartialOrd for OrdPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <String as PartialOrd>::partial_cmp(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}

impl Eq for OrdPath {}

impl PartialEq for OrdPath {
    fn eq(&self, other: &Self) -> bool {
        <String as PartialEq>::eq(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}
