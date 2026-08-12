#![doc = include_str!("../README.md")]

mod attr;

use std::marker::PhantomData;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    Expr, Ident, ItemMod, ItemUse, Signature, parse_macro_input,
    visit_mut::{VisitMut, visit_expr_mut, visit_signature_mut},
};

#[proc_macro_attribute]
/// Annotate `use` declarations with this to replace them in the syncified copy.
pub fn syncify_replace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let error = outside_use_error("syncify_replace").into_compile_error();
    join(item, error)
}

#[proc_macro_attribute]
/// Produce a synchronous copy of the annotated module, stripping `async` and `.await`.
pub fn syncify(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sync_mod_name = parse_macro_input!(attr as Ident);

    let mut input_mod = parse_macro_input!(item as ItemMod);

    let mut sync_copy = input_mod.clone();
    sync_copy.ident = sync_mod_name;

    let mut visitor = SyncifyVisitor::<Sync>::new();
    visitor.visit_item_mod_mut(&mut sync_copy);
    if let Some(tokens) = visitor.errors_tokens() {
        return tokens;
    }

    let mut visitor = SyncifyVisitor::<Async>::new();
    visitor.visit_item_mod_mut(&mut input_mod);
    if let Some(tokens) = visitor.errors_tokens() {
        return tokens;
    }

    let mut out = proc_macro2::TokenStream::new();
    input_mod.to_tokens(&mut out);
    sync_copy.to_tokens(&mut out);
    out.into()
}

/// Sync visitor mode:
/// * `async fn` becomes `fn`
/// * `expr.await` becomes `expr`
/// * `#[syncify_replace]` items are replaced
struct Sync;

/// Async visitor mode:
/// * `#[syncify_replace]` attributes are removed
struct Async;

/// Handle changes across the sync and async versions of the
/// modules to which `syncify` is applied.
struct SyncifyVisitor<Mode> {
    /// Errors collected during the syncify process.
    errors: Vec<syn::Error>,
    _mode: PhantomData<Mode>,
}

impl<Mode> SyncifyVisitor<Mode> {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            _mode: PhantomData,
        }
    }

    /// Returns the errors collected during the syncify process, if any.
    ///
    /// The errors are returned as a [`TokenStream`] so they can be emitted as compile errors.
    fn errors_tokens(self) -> Option<TokenStream> {
        if self.errors.is_empty() {
            return None;
        }
        Some(
            self.errors
                .into_iter()
                .map(|e| e.to_compile_error())
                .collect::<proc_macro2::TokenStream>()
                .into(),
        )
    }
}

impl VisitMut for SyncifyVisitor<Sync> {
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
        // `syncify_replace` attribute:
        // replace the `use` item with the replacement it carries, and drop the attribute.
        // Any other attributes are moved onto the replacement.
        match attr::extract(i, "syncify_replace") {
            Ok(Some(attr)) => {
                let mut replacement = None;
                match syn::parse2::<syn::UseTree>(attr::tokens(attr).clone()) {
                    Ok(rep) => replacement = Some(rep),
                    Err(err) => self.errors.push(err),
                }
                if let Some(replacement) = replacement {
                    i.tree = replacement;
                }
            }
            Err(e) => self.errors.push(e),
            Ok(None) => {}
        }
    }
}

impl VisitMut for SyncifyVisitor<Async> {
    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        let _ = attr::extract(i, "syncify_replace");
    }
}

/// Returns an error indicating that the `name` macro can only be used inside a
/// module marked with `#[syncify::syncify]`.
fn outside_use_error(name: &str) -> syn::Error {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("`{name}` can only be used inside a module marked with `#[syncify::syncify]`"),
    )
}

/// Joins two token streams together, in the order they are provided.
fn join(a: impl Into<TokenStream>, b: impl Into<TokenStream>) -> TokenStream {
    [a.into(), b.into()].into_iter().collect()
}
