use syn::visit_mut::VisitMut;

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
            impl VisitorMut for syn::$t {
                fn visit_mut<V: VisitMut + ?Sized>(&mut self, v: &mut V) {
                    syn::visit_mut::$impl(v, self);
                }
            }

            impl Empty for syn::$t {
                fn empty() -> Self {
                    syn::$t::Verbatim(Default::default())
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
