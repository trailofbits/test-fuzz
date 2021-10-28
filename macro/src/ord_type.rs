use quote::ToTokens;
use std::cmp::Ordering;
use syn::Type;

#[derive(Clone)]
pub struct OrdType(pub Type);

impl Ord for OrdType {
    fn cmp(&self, other: &Self) -> Ordering {
        <String as Ord>::cmp(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}

impl PartialOrd for OrdType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <String as PartialOrd>::partial_cmp(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}

impl Eq for OrdType {}

impl PartialEq for OrdType {
    fn eq(&self, other: &Self) -> bool {
        <String as PartialEq>::eq(
            &self.0.to_token_stream().to_string(),
            &other.0.to_token_stream().to_string(),
        )
    }
}
