use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
};

use quote::ToTokens;

use crate::source_difference::SourceDifference;

#[derive(Default)]
pub(crate) struct RustSurface {
    items: BTreeMap<String, Vec<String>>,
}

impl RustSurface {
    const DATA_DECLARATION: &str = "data_declaration";
    const SERIALIZATION_IMPL: &str = "serialization_impl";
    const OTHER_IMPL: &str = "other_impl";
    const FUNCTION: &str = "function";
    const TRAIT: &str = "trait";
    const OTHER_ITEM: &str = "other_item";

    pub(crate) fn parse(source: &str) -> Result<Self, Box<dyn Error>> {
        let syntax = syn::parse_file(source)?;
        let mut surface = Self::default();

        for attribute in syntax.attrs {
            surface.insert(Self::OTHER_ITEM, attribute.to_token_stream().to_string());
        }

        for item in &syntax.items {
            surface.collect(item, "");
        }

        Ok(surface)
    }

    fn collect(&mut self, item: &syn::Item, module: &str) {
        if let syn::Item::Mod(declaration) = item
            && let Some((_, items)) = &declaration.content
        {
            let scope = format!("{module}{}::", declaration.ident);

            let mut header = declaration.clone();
            header.content = None;

            self.insert(
                Self::OTHER_ITEM,
                format!("{scope} {}", header.to_token_stream(),),
            );

            for child in items {
                self.collect(child, &scope);
            }

            return;
        }

        let category = match item {
            syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Union(_) => {
                Self::DATA_DECLARATION
            }

            syn::Item::Impl(implementation) => {
                let serialization = implementation
                    .trait_
                    .as_ref()
                    .and_then(|(path, _)| path.segments.last())
                    .is_some_and(|segment| {
                        matches!(
                            segment.ident.to_string().as_str(),
                            "Serialize" | "Deserialize" | "Encode" | "Decode" | "BorrowDecode"
                        )
                    });

                if serialization {
                    Self::SERIALIZATION_IMPL
                } else {
                    Self::OTHER_IMPL
                }
            }

            syn::Item::Fn(_) => Self::FUNCTION,
            syn::Item::Trait(_) => Self::TRAIT,
            _ => Self::OTHER_ITEM,
        };

        self.insert(category, format!("{module} {}", item.to_token_stream()));
    }

    fn insert(&mut self, category: &str, value: String) {
        self.items
            .entry(category.to_owned())
            .or_default()
            .push(value);
    }

    pub(crate) fn differences(&self, current: &Self, path: &str) -> Vec<SourceDifference> {
        let categories: BTreeSet<_> = self.items.keys().chain(current.items.keys()).collect();

        let mut differences = Vec::new();

        for category in categories {
            let previous = self
                .items
                .get(category)
                .map(Vec::as_slice)
                .unwrap_or_default();

            let next = current
                .items
                .get(category)
                .map(Vec::as_slice)
                .unwrap_or_default();

            if previous != next {
                differences.push(SourceDifference::new(
                    path.to_owned(),
                    category.clone(),
                    previous.to_vec(),
                    next.to_vec(),
                ));
            }
        }

        differences
    }
}
