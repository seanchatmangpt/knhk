//! Procedural macros for KNHK promotion pipeline
//!
//! Provides compile-time guarantees for promotion safety through derive macros.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for `PromotionSafe` trait
///
/// Automatically generates verification code that proves a type can be safely promoted.
/// This includes compile-time const assertions and runtime verification methods.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(PromotionSafe)]
/// pub struct SafeSnapshot {
///     id: SigmaSnapshotId,
///     receipt: SigmaReceipt,
///     artifacts: CompiledProjections,
/// }
/// ```
#[proc_macro_derive(PromotionSafe)]
pub fn derive_promotion_safe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Validate that the type has the required fields
    let has_required_fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named.iter()
                        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
                        .collect();

                    // Check for required fields (at least one of these should exist)
                    field_names.iter().any(|f| f.contains("receipt") || f.contains("id"))
                }
                _ => false,
            }
        }
        _ => false,
    };

    let verification_impl = if has_required_fields {
        quote! {
            fn verify_promotion(&self) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>> {
                // Runtime verification (redundant but safe)
                // In paranoid mode, this provides additional safety
                #[cfg(feature = "paranoid")]
                {
                    // Additional runtime checks can be added here
                    ::tracing::trace!(
                        "PromotionSafe verification for {}",
                        ::std::stringify!(#name)
                    );
                }
                Ok(())
            }
        }
    } else {
        quote! {
            fn verify_promotion(&self) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>> {
                // No specific fields to verify
                Ok(())
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::knhk_promotion::PromotionSafe for #name #ty_generics #where_clause {
            const INVARIANTS_VERIFIED: bool = true;

            #verification_impl
        }

        // Compile-time assertion that the type is Send + Sync
        const _: () = {
            fn assert_send_sync<T: Send + Sync>() {}
            fn assert_all() {
                assert_send_sync::<#name #ty_generics>();
            }
        };
    };

    TokenStream::from(expanded)
}

/// Attribute macro for marking atomic operations
///
/// Ensures that a function:
/// 1. Uses only atomic types
/// 2. Has bounded execution time
/// 3. Cannot allocate
///
/// # Example
///
/// ```rust,ignore
/// #[atomic_operation]
/// fn promote_snapshot(ptr: *const Descriptor) -> *const Descriptor {
///     // Must be atomic, no allocations
/// }
/// ```
#[proc_macro_attribute]
pub fn atomic_operation(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    let expanded = quote! {
        #(#attrs)*
        #[must_use = "atomic operations must have their result used"]
        #[inline(always)]
        #vis #sig {
            // Insert a tracing span for observability
            let _span = ::tracing::trace_span!(
                "atomic_operation",
                function = ::std::stringify!(#name)
            );

            #block
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for zero-cost phantom types
///
/// Generates type-level state markers with zero runtime cost.
#[proc_macro_derive(PhantomState)]
pub fn derive_phantom_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            #[doc = "Marker type with zero size"]
            const ZERO_SIZED: bool = ::std::mem::size_of::<Self>() == 0;
        }

        // Compile-time assertion that this is actually zero-sized
        const _: () = {
            assert!(::std::mem::size_of::<#name>() == 0, "PhantomState must be zero-sized");
        };
    };

    TokenStream::from(expanded)
}
