use proc_macro2::TokenStream;
use quote::ToTokens;
use spanned::Spanned;
use syn::*;

/// Trait for mutable access to the attributes of an item.
pub trait AttrsMut {
    /// Returns a mutable reference to the attributes of the item.
    fn attrs_mut(&mut self) -> &mut Vec<Attribute>;
}

impl<T: AttrsMut> AttrsMut for &mut T {
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        T::attrs_mut(self)
    }
}

macro_rules! impl_attrs_mut {
    (enum $base:ident { $($enum:ident => $type:ident),* $(,)? }) => {
        impl AttrsMut for $base {
            fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
                match self {
                    $($base::$enum(i, ..) => i.attrs_mut(),)*
                    #[allow(unreachable_patterns)]
                    _ => unimplemented!(),
                }
            }
        }

        $(
            impl AttrsMut for $type {
                fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
                    &mut self.attrs
                }
            }
        )*
    };
}

impl_attrs_mut! {
    enum Item {
        Const => ItemConst,
        Enum => ItemEnum,
        ExternCrate => ItemExternCrate,
        Fn => ItemFn,
        ForeignMod => ItemForeignMod,
        Impl => ItemImpl,
        Macro => ItemMacro,
        Mod => ItemMod,
        Static => ItemStatic,
        Struct => ItemStruct,
        Trait => ItemTrait,
        TraitAlias => ItemTraitAlias,
        Type => ItemType,
        Union => ItemUnion,
        Use => ItemUse,
    }
}

impl_attrs_mut! {
    enum ImplItem {
        Const => ImplItemConst,
        Fn => ImplItemFn,
        Type => ImplItemType,
        Macro => ImplItemMacro,
    }
}

impl_attrs_mut! {
    enum TraitItem {
        Const => TraitItemConst,
        Fn => TraitItemFn,
        Type => TraitItemType,
        Macro => TraitItemMacro,
    }
}

/// Returns the path of the attribute
pub fn path(attr: &Attribute) -> &Path {
    match &attr.meta {
        Meta::Path(path) => path,
        Meta::List(list) => &list.path,
        Meta::NameValue(nv) => &nv.path,
    }
}

/// Returns the tokens of the attribute value.
pub fn tokens(attr: Attribute) -> TokenStream {
    match attr.meta {
        Meta::Path(_) => TokenStream::new(),
        Meta::List(list) => list.tokens,
        Meta::NameValue(nv) => nv.value.into_token_stream(),
    }
}

/// Returns `true` if the attribute is a `syncify` attribute with the given name.
pub fn is_syncify(attr: &Attribute, name: &str) -> bool {
    matches!(attr.style, AttrStyle::Outer)
        .then_some(path(attr))
        .and_then(|path| path.segments.last())
        .is_some_and(|segment| segment.ident == name)
}

/// Removes the `syncify` attribute with the given name from the item's attributes,
/// returning it if found.
///
/// Returns an error if the attribute is used multiple times.
pub fn extract(item: &mut impl AttrsMut, name: &str) -> Result<Option<Attribute>> {
    let mut iter = item
        .attrs_mut()
        .extract_if(.., |attr| is_syncify(attr, name));
    let first = iter.next();
    // NOTE: last() ensures the iterator is advanced to the end,
    //       meaning that all the attributes have been extracted/dropped.
    if let Some(last) = iter.last() {
        return Err(Error::new(
            last.span(),
            format!("{name} attribute used multiple times"),
        ));
    }
    Ok(first)
}

/// Removes the `syncify` **path only** attribute with the given name from the item's attributes,
/// returning it if found.
///
/// Returns an error if the attribute is used multiple times or if it has a value
/// (either as a list or a name-value pair).
pub fn extract_path(item: &mut impl AttrsMut, name: &str) -> Result<Option<Attribute>> {
    let path_only = |attr: Attribute| match attr.meta {
        Meta::Path(_) => Ok(attr),
        _ => Err(Error::new(
            attr.span(),
            format!("{name} attribute doesn't accept values"),
        )),
    };
    extract(item, name).and_then(|attr| attr.map(path_only).transpose())
}
