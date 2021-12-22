use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use strum_macros::Display;

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum SerdeFormat {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    Bincode,
    #[cfg(feature = "__serde_cbor")]
    Cbor,
}

#[allow(clippy::vec_init_then_push)]
#[must_use]
pub fn serde_format() -> SerdeFormat {
    let mut formats = vec![];
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    formats.push(SerdeFormat::Bincode);
    #[cfg(feature = "__serde_cbor")]
    formats.push(SerdeFormat::Cbor);
    assert!(
        formats.len() <= 1,
        "Multiple serde formats selected: {:?}",
        formats
    );
    formats.pop().expect("No serde format selected")
}

impl SerdeFormat {
    #[must_use]
    pub fn as_feature(self) -> &'static str {
        match self {
            #[cfg(any(serde_default, feature = "__serde_bincode"))]
            SerdeFormat::Bincode => "serde_bincode",
            #[cfg(feature = "__serde_cbor")]
            SerdeFormat::Cbor => "serde_cbor",
        }
    }
}

impl ToTokens for SerdeFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = Ident::new(&self.to_string(), Span::call_site());
        tokens.append_all(quote! {
            test_fuzz::SerdeFormat::#ident
        });
    }
}
