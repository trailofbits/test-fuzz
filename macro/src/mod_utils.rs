use proc_macro::Span;
use std::fs::read_to_string;
use syn::{
    parse_str,
    spanned::Spanned,
    visit::{visit_item_mod, Visit},
    File, Ident, ItemMod,
};

struct ModVisitor<'span, 'ast> {
    pub target: &'span Span,
    pub stack: Vec<&'ast ItemMod>,
    pub result: Option<Vec<Ident>>,
}

impl<'span, 'ast> Visit<'ast> for ModVisitor<'span, 'ast> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if contains(&module.span().unwrap(), self.target) {
            self.stack.push(module);

            visit_item_mod(self, module);

            if self.result.is_none() {
                self.result = Some(
                    self.stack
                        .iter()
                        .map(|module| module.ident.clone())
                        .collect(),
                )
            }

            self.stack.pop();
        }
    }
}

fn contains(left: &Span, right: &Span) -> bool {
    left.join(*right).map_or(false, |join| join.eq(left))
}

pub fn module_path(span: &Span) -> Vec<Ident> {
    let source = span.source_file();
    let path = source.path();
    let contents =
        read_to_string(&path).expect(&format!("`read_to_string` failed for `{:?}`", path));
    let file: File =
        parse_str(&contents).expect(&format!("Could not parse `{:?}` contents", source));
    let mut visitor = ModVisitor {
        target: span,
        stack: Vec::new(),
        result: None,
    };
    visitor.visit_file(&file);
    visitor.result.unwrap_or_default()
}
