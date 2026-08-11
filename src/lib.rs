#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    AttrStyle, Expr, Ident, ItemMod, ItemUse, Meta, MetaList, Signature, parse_macro_input,
    spanned::Spanned,
    visit_mut::{VisitMut, visit_expr_mut, visit_signature_mut},
};

#[proc_macro_attribute]
/// Annotate `use` declarations with this to replace them in the syncified copy.
pub fn syncify_replace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Avoid usage on items that are not `use` declarations.
    let clone = item.clone();
    let _ = parse_macro_input!(clone as ItemUse);
    item
}

#[proc_macro_attribute]
/// Produce a synchronous copy of the annotated module, stripping `async` and `.await`.
pub fn syncify(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sync_mod_name = parse_macro_input!(attr as Ident);

    let input_mod = parse_macro_input!(item as ItemMod);

    let mut sync_copy = input_mod.clone();
    sync_copy.ident = sync_mod_name;

    let mut visitor = SyncifyVisitor::new();
    visitor.visit_item_mod_mut(&mut sync_copy);
    if !visitor.errors.is_empty() {
        return visitor
            .errors
            .into_iter()
            .map(|e| e.to_compile_error())
            .collect::<proc_macro2::TokenStream>()
            .into();
    }

    let mut out = proc_macro2::TokenStream::new();
    input_mod.to_tokens(&mut out);
    sync_copy.to_tokens(&mut out);
    out.into()
}

/// Convert async code into its blocking equivalent:
/// * `async fn` becomes `fn`
/// * `expr.await` becomes `expr`
struct SyncifyVisitor {
    errors: Vec<syn::Error>,
}

impl SyncifyVisitor {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }
}

impl VisitMut for SyncifyVisitor {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        visit_expr_mut(self, i);
        if let Expr::Await(expr) = i {
            *i = *expr.base.clone();
        }
    }

    fn visit_signature_mut(&mut self, i: &mut Signature) {
        i.asyncness = None;
        visit_signature_mut(self, i);
    }

    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        // Honour `syncify_replace` attributes: replace the `use` item with the
        // replacement it carries, and drop the attribute. Any other attributes are
        // moved onto the replacement.
        let mut replacement = None;
        strip_replace(i, |meta| {
            let None = replacement else {
                let err =
                    syn::Error::new(meta.span(), "syncify_replace attribute used multiple times");
                self.errors.push(err);
                return;
            };
            match syn::parse2::<ItemUse>(meta.tokens.clone()) {
                Ok(rep) => replacement = Some(rep),
                Err(err) => self.errors.push(err),
            }
        });
        if let Some(mut replacement) = replacement {
            replacement.attrs = i.attrs.clone();
            *i = replacement;
        }
    }
}

/// Remove `syncify_replace` attributes from `use` item, calling `on_replace` with the replacement token stream and span.
fn strip_replace(item: &mut ItemUse, mut on_replace: impl FnMut(&MetaList)) {
    item.attrs.retain_mut(|attr| {
        if !matches!(attr.style, AttrStyle::Outer) {
            return true;
        }
        let Meta::List(meta_list) = &attr.meta else {
            return true;
        };
        let Some(segment) = meta_list.path.segments.last() else {
            return true;
        };
        if segment.ident != "syncify_replace" {
            return true;
        }
        on_replace(meta_list);
        false
    });
}
