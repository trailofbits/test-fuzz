use std::fmt;
use syn::Signature;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum Kind {
    None,
    Multiple,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Error {
    impl_: bool,
    sig: Signature,
    krate: String,
    target: String,
    kind: Kind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::None => write!(
                f,
                "either (1) tests have not been run or (2) tests produced no {}concretizations for \
                `{}`",
                if self.impl_ { "impl " } else { "" },
                self.target,
            ),
            Kind::Multiple => write!(
                f,
                "tests produced multiple {}concretizations for {}. View them with \
                `--display-{}concretizations --exact {}`.",
                if self.impl_ { "impl " } else { "" },
                self.target,
                self.target,
                if self.impl_ { "impl-" } else { "" },
            ),
        }
    }
}

#[cfg(feature = "__auto_concretize")]
pub use functions::*;

#[cfg(feature = "__auto_concretize")]
mod functions {
    use super::*;
    use crate::{mod_utils, CARGO_CRATE_NAME};
    use proc_macro::Span;
    use std::{
        fs::{read_dir, read_to_string},
        iter,
    };
    use syn::Signature;

    pub fn unique_impl_concretization(sig: &Signature) -> Result<String, Error> {
        let target = target_from_sig(sig);
        unique_file_as_string(&internal::dirs::impl_concretizations_directory_from_target(
            &CARGO_CRATE_NAME,
            &target,
        ))
        .map_err(|kind| Error {
            impl_: true,
            sig: sig.clone(),
            krate: CARGO_CRATE_NAME.clone(),
            target,
            kind,
        })
    }

    pub fn unique_concretization(sig: &Signature) -> Result<String, Error> {
        let target = target_from_sig(sig);
        unique_file_as_string(&internal::dirs::concretizations_directory_from_target(
            &CARGO_CRATE_NAME,
            &target,
        ))
        .map_err(|kind| Error {
            impl_: false,
            sig: sig.clone(),
            krate: CARGO_CRATE_NAME.clone(),
            target,
            kind,
        })
    }

    fn target_from_sig(sig: &Signature) -> String {
        mod_utils::module_path(&Span::call_site())
            .iter()
            .chain(iter::once(&sig.ident))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("::")
    }

    fn unique_file_as_string(dir: &std::path::Path) -> Result<String, Kind> {
        let mut unique_file = None;

        for entry in read_dir(dir).map_err(|_| Kind::None)? {
            let entry = entry.map_err(|_| Kind::None)?;
            let path = entry.path();

            if unique_file.is_some() {
                return Err(Kind::Multiple);
            }

            unique_file = Some(path);
        }

        unique_file.map_or_else(
            || Err(Kind::None),
            |path| {
                Ok(read_to_string(&path)
                    .expect(&format!("`read_to_string` failed for `{:?}`", path)))
            },
        )
    }
}
