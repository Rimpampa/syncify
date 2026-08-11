#![doc = include_str!("../README.md")]

mod attr;
mod default;

use std::marker::PhantomData;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    Expr, ExprBlock, GenericArgument, Ident, ImplItem, Item, ItemMod, ItemUse, PathArguments,
    ReturnType, Signature, Stmt, TraitItem, Type, TypeParamBound, parse_macro_input,
    visit_mut::{VisitMut, visit_expr_mut, visit_signature_mut},
};

use crate::{
    attr::AttrsMut,
    default::{Empty, VisitorMut},
};

#[proc_macro_attribute]
/// Annotate `use` declarations with this to replace them in the syncified copy.
pub fn syncify_replace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let error = outside_use_error("syncify_replace").into_compile_error();
    join(item, error)
}

#[proc_macro_attribute]
/// Annotate an item with this to **skip** it in the syncified copy.
pub fn syncify_skip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let error = outside_use_error("syncify_skip").into_compile_error();
    join(item, error)
}

#[proc_macro_attribute]
/// Annotate an item with this to include it **only** in the syncified copy.
pub fn syncify_include(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let error = outside_use_error("syncify_include").into_compile_error();
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
/// * `async { .. }` becomes `{ .. }`
/// * `#[syncify_replace]` items are replaced
/// * `#[syncify_skip]` items are removed
/// * `#[syncify_include]` attributes are removed
/// * `-> impl Future<Output = T> + ...` becomes `-> T`
struct Sync;

/// Async visitor mode:
/// * `#[syncify_replace]` attributes are removed
/// * `#[syncify_include]` items are removed
/// * `#[syncify_skip]` attributes are removed
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

    /// Removes the entire `Item` if the fiven attribute is found.
    ///
    /// Removing here means replacing with its [`Empty`] equivalent.
    fn remove_if_attr<T: AttrsMut + Empty>(&mut self, i: &mut T, attr: &str) -> bool {
        let res = attr::extract_path(i, attr);
        match res.map_err(|e| self.errors.push(e)) {
            Ok(Some(_)) => *i = T::empty(),
            _ => return false,
        }
        true
    }
}

impl SyncifyVisitor<Sync> {
    /// [`Sync`] mode implementation of the `syncify_include` and `syncify_skip` attributes for `Item`-like types.
    fn skip_include_impl<T: AttrsMut + Empty + VisitorMut>(&mut self, i: &mut T) {
        // `syncify_include` attribute:
        // drop references to the attribute, any error is handled by the
        // sync mode visitor.
        let _ = attr::extract(i, "syncify_include");
        // `syncify_skip` attribute:
        // remove the `Item` if found.
        if !self.remove_if_attr(i, "syncify_skip") {
            i.visit_mut(self);
        }
    }
}

impl SyncifyVisitor<Async> {
    /// [`Async`] mode implementation of the `syncify_include` and `syncify_skip` attributes for `Item`-like types.
    fn skip_include_impl<T: AttrsMut + Empty + VisitorMut>(&mut self, i: &mut T) {
        // `syncify_skip` attribute:
        // drop references to the attribute, any error is handled by the
        // sync mode visitor.
        let _ = attr::extract(i, "syncify_skip");
        // `syncify_include` attribute:
        // remove the `Item` if found.
        if !self.remove_if_attr(i, "syncify_include") {
            i.visit_mut(self);
        }
    }
}

impl VisitMut for SyncifyVisitor<Sync> {
    fn visit_item_mut(&mut self, i: &mut Item) {
        self.skip_include_impl(i)
    }

    fn visit_impl_item_mut(&mut self, i: &mut ImplItem) {
        self.skip_include_impl(i)
    }

    fn visit_trait_item_mut(&mut self, i: &mut TraitItem) {
        self.skip_include_impl(i)
    }

    fn visit_expr_mut(&mut self, i: &mut Expr) {
        while let Some(replace) = async_expr_pass(i) {
            *i = replace;
        }
        visit_expr_mut(self, i);
    }

    fn visit_signature_mut(&mut self, i: &mut Signature) {
        i.asyncness = None;
        replace_future_return(&mut i.output);
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
    fn visit_item_mut(&mut self, i: &mut Item) {
        self.skip_include_impl(i)
    }

    fn visit_impl_item_mut(&mut self, i: &mut ImplItem) {
        self.skip_include_impl(i)
    }

    fn visit_trait_item_mut(&mut self, i: &mut TraitItem) {
        self.skip_include_impl(i)
    }

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

/// Replaces `await` and `async` expressions with their inner expressions.
fn async_expr_pass(i: &Expr) -> Option<Expr> {
    Some(match i {
        Expr::Await(expr) => *expr.base.clone(),
        Expr::Async(expr) => {
            // Drop the useless braces when the block is a single
            // expression.
            if let [Stmt::Expr(expr, None)] = expr.block.stmts.as_slice() {
                expr.clone()
            } else {
                Expr::Block(ExprBlock {
                    attrs: expr.attrs.clone(),
                    block: expr.block.clone(),
                    label: None,
                })
            }
        }
        _ => return None,
    })
}

/// Replaces `-> impl Future<Output = T> + ...` with `-> T`.
fn replace_future_return(i: &mut ReturnType) {
    let ReturnType::Type(_, ty) = i else { return };
    let Type::ImplTrait(impl_trait) = ty.as_ref() else {
        return;
    };
    let new_ty = impl_trait.bounds.iter().find_map(|bound| {
        let TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        let args = &bound
            .path
            .segments
            .last()
            .filter(|seg| seg.ident == "Future")?
            .arguments;
        let PathArguments::AngleBracketed(args) = args else {
            return None;
        };
        args.args.iter().find_map(|arg| {
            let GenericArgument::AssocType(assoc) = arg else {
                return None;
            };
            assoc.ident.eq("Output").then(|| assoc.ty.clone())
        })
    });
    if let Some(new_ty) = new_ty {
        **ty = new_ty;
    }
}
