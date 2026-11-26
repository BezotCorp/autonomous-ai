use std::collections::BTreeSet;

use quote::ToTokens;

/// Public shape of a Rust source file: its items grouped by syntactic
/// category, function bodies excluded. Comparing two surfaces reveals the
/// structural changes between two revisions of a file.
#[derive(Debug, Default)]
pub(crate) struct RustSurface {
    items: BTreeSet<(String, String)>,
}

impl RustSurface {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
        let mut surface = Self::default();

        for item in &syntax.items {
            surface.collect(item, "");
        }

        Ok(surface)
    }

    /// Items present here and absent from `other`, as (category, signature).
    pub(crate) fn missing_from<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = &'a (String, String)> {
        self.items.difference(&other.items)
    }

    fn collect(&mut self, item: &syn::Item, scope: &str) {
        let (category, signature) = match item {
            syn::Item::Mod(module) => {
                let path = format!("{scope}{}", module.ident);

                if let Some((_, items)) = &module.content {
                    for child in items {
                        self.collect(child, &format!("{path}::"));
                    }
                }

                ("mod", path)
            }
            syn::Item::Fn(function) => (
                "fn",
                format!(
                    "{scope}{} {}",
                    function.vis.to_token_stream(),
                    function.sig.to_token_stream()
                ),
            ),
            syn::Item::Impl(implementation) => {
                let target = implementation.self_ty.to_token_stream().to_string();

                for member in &implementation.items {
                    let signature = match member {
                        syn::ImplItem::Fn(function) => format!(
                            "{} {}",
                            function.vis.to_token_stream(),
                            function.sig.to_token_stream()
                        ),
                        other => other.to_token_stream().to_string(),
                    };

                    self.insert("impl_member", format!("{scope}{target} :: {signature}"));
                }

                let mut header = implementation.clone();
                header.items.clear();

                ("impl", format!("{scope}{}", header.to_token_stream()))
            }
            syn::Item::Struct(_) => ("struct", Self::text(scope, item)),
            syn::Item::Enum(_) => ("enum", Self::text(scope, item)),
            syn::Item::Union(_) => ("union", Self::text(scope, item)),
            syn::Item::Trait(_) => ("trait", Self::text(scope, item)),
            syn::Item::Type(_) => ("type", Self::text(scope, item)),
            syn::Item::Const(_) => ("const", Self::text(scope, item)),
            syn::Item::Static(_) => ("static", Self::text(scope, item)),
            syn::Item::Use(_) => ("use", Self::text(scope, item)),
            syn::Item::Macro(_) => ("macro", Self::text(scope, item)),
            _ => ("other", Self::text(scope, item)),
        };

        self.insert(category, signature);
    }

    fn text(scope: &str, item: &syn::Item) -> String {
        format!("{scope}{}", item.to_token_stream())
    }

    fn insert(&mut self, category: &str, signature: String) {
        self.items.insert((category.to_owned(), signature));
    }
}

#[cfg(test)]
mod tests {
    use super::RustSurface;

    #[test]
    fn ignores_bodies_but_sees_signatures() {
        let before = RustSurface::parse("pub fn f(x: u8) -> u8 { x } struct S { a: u8 }").unwrap();
        let body =
            RustSurface::parse("pub fn f(x: u8) -> u8 { x + 1 } struct S { a: u8 }").unwrap();
        let after = RustSurface::parse("pub fn f(x: u16) -> u8 { 0 } struct S { a: u8 }").unwrap();

        assert_eq!(before.missing_from(&body).count(), 0);
        assert_eq!(before.missing_from(&after).count(), 1);
        assert_eq!(after.missing_from(&before).count(), 1);
    }
}
