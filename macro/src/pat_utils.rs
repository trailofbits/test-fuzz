use syn::{
    visit::{visit_pat, Visit},
    Ident, Pat, PatIdent,
};

struct PatVisitor<'a> {
    idents: Vec<&'a Ident>,
}

impl<'a> Visit<'a> for PatVisitor<'a> {
    fn visit_pat(&mut self, pat: &'a Pat) {
        if let Pat::Ident(PatIdent { ident, .. }) = pat {
            self.idents.push(ident);
        }
        visit_pat(self, pat);
    }
}

pub fn pat_idents(pat: &Pat) -> Vec<&Ident> {
    let mut visitor = PatVisitor { idents: Vec::new() };
    visitor.visit_pat(pat);
    visitor.idents
}
