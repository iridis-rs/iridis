//! This module contains the macros `Node` and `node(runtime)`.
//! It's used to generate the necessary boilerplate code for creating a node.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, ImplItem, ItemImpl, ReturnType, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Apply this macro to a struct to generate the `C` symbols
/// and the according `tokio::runtime::Runtime`.
#[proc_macro_derive(Node)]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        #[cfg(feature = "cdylib")]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub static IRIDIS_NODE: iridis_node::prelude::DynamicallyLinkedNodeInstance = |inputs, outputs, queries, queryables, configuration| {
            <#name>::new(inputs, outputs, queries, queryables, configuration)
        };

        static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<iridis_node::prelude::thirdparty::tokio::runtime::Runtime> =
            std::sync::LazyLock::new(|| iridis_node::prelude::thirdparty::tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

        fn default_runtime<T: Send + 'static>(
            task: impl Future<Output = T> + Send + 'static,
        ) -> iridis_node::prelude::thirdparty::tokio::task::JoinHandle<T> {
            match iridis_node::prelude::thirdparty::tokio::runtime::Handle::try_current() {
                Ok(handle) => handle.spawn(task),
                Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task)
            }
        }
    };

    TokenStream::from(expanded)
}

struct MacroArgs {
    runtime: String,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut runtime = String::new();

        let vars = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;

        for var in vars {
            if let syn::Meta::NameValue(name_value) = var {
                let name = name_value.path.get_ident().unwrap().to_string();

                if name == "runtime" {
                    if let syn::Expr::Lit(lit) = &name_value.value {
                        if let syn::Lit::Str(lit_str) = &lit.lit {
                            runtime = lit_str.value();
                        }
                    }
                }
            }
        }

        Ok(MacroArgs { runtime })
    }
}

/// Use this macro to mark an `impl` block on a node. This will alter
/// the `new` and `start` methods to return a `tokio::task::JoinHandle` with the provided
/// runtime. The parameter must be a function that takes an `async` closure and returns
/// a `JoinHandle`.
///
/// ```rust,ignore
/// static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
///     std::sync::LazyLock::new(|| {
///         tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
///     });
///
/// fn default_runtime<T: Send + 'static>(
///     task: impl Future<Output = T> + Send + 'static,
/// ) -> tokio::task::JoinHandle<T> {
///     match tokio::runtime::Handle::try_current() {
///         Ok(handle) => handle.spawn(task),
///         Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task),
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    let args = parse_macro_input!(attr as MacroArgs);
    let runtime_tokens = args.runtime.parse::<proc_macro2::TokenStream>().unwrap();

    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item {
            let was_async = method.sig.asyncness.is_some();
            method.sig.asyncness = None;

            let old_block = method.block.clone();

            if was_async {
                let old_return_type = match &method.sig.output {
                    ReturnType::Default => quote! { () },
                    ReturnType::Type(_, ty) => {
                        if method.sig.ident == "new" {
                            quote! { iridis_node::prelude::thirdparty::eyre::Result<Box<dyn iridis_node::prelude::Node>> }
                        } else {
                            quote! { #ty }
                        }
                    }
                };

                method.sig.output = syn::parse_quote! {
                    -> tokio::task::JoinHandle<#old_return_type>
                };

                if method.sig.ident == "new" {
                    method.block = syn::parse_quote! {
                        {
                            #runtime_tokens(async move {
                                #old_block.map(|node| Box::new(node) as Box<dyn iridis_node::prelude::Node>)
                            })
                        }
                    };
                } else {
                    method.block = syn::parse_quote! {
                        {
                            #runtime_tokens(async move {
                                #old_block
                            })
                        }
                    };
                }
            } else {
                panic!("Function is not async");
            }
        }
    }

    quote! {
        #impl_block
    }
    .into()
}
