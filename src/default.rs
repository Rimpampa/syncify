use syn::*;
use visit_mut::*;

/// A trait for retrieving the default [`VisitMut`] provided by the [`syn`] crate.
pub trait VisitorMut {
    fn visit_mut<V: VisitMut + ?Sized>(&mut self, v: &mut V);
}

/// A trait for retrieving the default empty value of a [`syn`] type.
pub trait Empty {
    fn empty() -> Self;
}

macro_rules! impl_defaults {
    ($($t:ident => $impl:ident),* $(,)?) => {
        $(
            impl VisitorMut for $t {
                fn visit_mut<V: VisitMut + ?Sized>(&mut self, v: &mut V) {
                    $impl(v, self);
                }
            }

            impl Empty for $t {
                fn empty() -> Self {
                    $t::Verbatim(Default::default())
                }
            }
        )*
    };
}

impl_defaults! {
    Item => visit_item_mut,
    ImplItem => visit_impl_item_mut,
    TraitItem => visit_trait_item_mut,
}
