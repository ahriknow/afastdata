//! # afastdata-macro
//!
//! afastdata 序列化框架的 derive 宏，为结构体和枚举自动生成
//! [`AFastSerialize`] 和 [`AFastDeserialize`] trait 的实现。
//!
//! Derive macros for the afastdata serialization framework, automatically generating
//! implementations of [`AFastSerialize`]` and [`AFastDeserialize`] traits for
//! structs and enums.
//!
//! ## 支持的类型 / Supported Types
//!
//! - **命名字段结构体 (Named-field struct)**：逐字段序列化/反序列化
//!   / Field-by-field serialization/deserialization
//! - **元组结构体 (Tuple struct)**：按索引逐字段序列化/反序列化
//!   / Index-based field serialization/deserialization
//! - **单元结构体 (Unit struct)**：生成空实现（不产生任何字节）
//!   / Generates an empty implementation (no bytes produced)
//! - **枚举 (Enum)**：写入 `u32` 变体索引 + 变体字段数据
//!   / Writes a `u32` variant index + variant field data
//!
//! ## 编码格式 / Encoding Format
//!
//! ### 结构体 / Struct
//!
//! 所有字段按声明顺序依次调用 `to_bytes()` / `from_bytes()`，无额外前缀。
//!
//! All fields call `to_bytes()` / `from_bytes()` in declaration order, with no
//! additional prefix.
//!
//! ### 枚举 / Enum
//!
//! | 编码内容 / Content | 类型 / Type | 说明 / Description |
//! |---|---|---|
//! | 变体索引 / Variant index | `u32` little-endian | 从 0 开始递增 / Starts from 0, incrementing |
//! | 变体字段 / Variant fields | 逐字段编码 / Field-wise encoding | 仅非 unit 变体 / Only for non-unit variants |
//!
//! ## 泛型支持 / Generic Support
//!
//! 泛型参数会自动添加 `AFastSerialize` 和（对于反序列化）`AFastDeserialize` trait 约束。
//! 如果泛型参数仅用于某些字段，生成的约束可能过于严格，但这保证了实现的正确性。
//!
//! Generic parameters automatically receive `AFastSerialize` and (for deserialization)
//! `AFastDeserialize` trait bounds. If a generic parameter is only used in certain fields,
//! the generated bounds may be overly strict, but this ensures correctness.
//!
//! ## 示例 / Example
//!
//! ```ignore
//! use afastdata::{AFastSerialize, AFastDeserialize};
//!
//! #[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! #[derive(AFastSerialize, AFastDeserialize, Debug, PartialEq)]
//! enum Shape {
//!     Circle(f64),
//!     Rectangle { width: f64, height: f64 },
//!     Empty,
//! }
//!
//! // 序列化 / Serialize
//! let point = Point { x: 10, y: 20 };
//! let bytes = point.to_bytes();
//!
//! // 反序列化 / Deserialize
//! let (decoded, _) = Point::from_bytes(&bytes).unwrap();
//! assert_eq!(point, decoded);
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

/// 为结构体或枚举生成 [`AFastSerialize`] trait 实现。
///
/// Generates an [`AFastSerialize`] trait implementation for a struct or enum.
///
/// # 生成的代码 / Generated Code
///
/// ## 结构体 / Struct
///
/// 为每个字段调用 `to_bytes()`，并将结果依次追加到字节缓冲区中。
///
/// Calls `to_bytes()` on each field, appending the results to a byte buffer
/// sequentially.
///
/// ```ignore
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastSerialize for MyStruct {
///     fn to_bytes(&self) -> Vec<u8> {
///         let mut bytes = Vec::new();
///         bytes.extend(AFastSerialize::to_bytes(&self.field1));
///         bytes.extend(AFastSerialize::to_bytes(&self.field2));
///         // ...
///         bytes
///     }
/// }
/// ```
///
/// ## 枚举 / Enum
///
/// 先写入 `u32` 变体索引（从 0 开始），再写入变体的字段数据。
/// Unit 变体只写入索引。
///
/// Writes a `u32` variant index (starting from 0), then the variant's field data.
/// Unit variants only write the index.
///
/// ```ignore
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastSerialize for MyEnum {
///     fn to_bytes(&self) -> Vec<u8> {
///         let mut bytes = Vec::new();
///         match self {
///             MyEnum::Variant1 => {
///                 bytes.extend(0u32.to_le_bytes());
///             }
///             MyEnum::Variant2(field) => {
///                 bytes.extend(1u32.to_le_bytes());
///                 bytes.extend(AFastSerialize::to_bytes(field));
///             }
///             // ...
///         }
///         bytes
///     }
/// }
/// ```
///
/// # 泛型 / Generics
///
/// 如果目标类型包含泛型参数，生成的 `impl` 会自动为这些参数添加
/// `AFastSerialize` 约束。
///
/// If the target type contains generic parameters, the generated `impl` automatically
/// adds `AFastSerialize` bounds to those parameters.
///
/// # Panics
///
/// 对 union 类型使用此宏会触发编译 panic。
///
/// Using this macro on a union type will trigger a compile-time panic.
#[proc_macro_derive(AFastSerialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // 为泛型参数添加 AFastSerialize trait 约束
    // Add AFastSerialize trait bounds to generic parameters
    let mut generics_with_bounds = generics.clone();
    for param in &mut generics_with_bounds.params {
        if let syn::GenericParam::Type(ref mut ty) = *param {
            ty.bounds.push(syn::parse_quote!(::afastdata_core::AFastSerialize));
        }
    }
    let (impl_generics, _, _) = generics_with_bounds.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data) => {
            let serialize_body = generate_serialize_fields(&data.fields, quote!(self));
            quote! {
                impl #impl_generics ::afastdata_core::AFastSerialize for #name #ty_generics {
                    fn to_bytes(&self) -> Vec<u8> {
                        let mut bytes = Vec::new();
                        #(#serialize_body)*
                        bytes
                    }
                }
            }
        }
        Data::Enum(data) => {
            let mut arms = Vec::new();
            for (i, variant) in data.variants.iter().enumerate() {
                let variant_name = &variant.ident;
                let tag = i as u32;

                match &variant.fields {
                    Fields::Unit => {
                        // Unit 变体：只写入 u32 索引
                        // Unit variant: write only the u32 index
                        arms.push(quote! {
                            #name::#variant_name => {
                                bytes.extend(#tag.to_le_bytes());
                            }
                        });
                    }
                    Fields::Unnamed(fields) => {
                        // 元组变体：写入 u32 索引 + 逐字段序列化
                        // Tuple variant: write u32 index + field-by-field serialization
                        let field_names: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| {
                                let ident = syn::Ident::new(&format!("__f{}", i), variant_name.span());
                                ident
                            })
                            .collect();
                        let field_patterns = &field_names;
                        let mut serialize_fields = Vec::new();
                        for fname in &field_names {
                            serialize_fields.push(quote! {
                                bytes.extend(::afastdata_core::AFastSerialize::to_bytes(#fname));
                            });
                        }
                        arms.push(quote! {
                            #name::#variant_name(#(#field_patterns),*) => {
                                bytes.extend(#tag.to_le_bytes());
                                #(#serialize_fields)*
                            }
                        });
                    }
                    Fields::Named(fields) => {
                        // 命名字段变体：写入 u32 索引 + 逐字段序列化
                        // Named-field variant: write u32 index + field-by-field serialization
                        let field_names: Vec<_> =
                            fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                        let mut serialize_fields = Vec::new();
                        for fname in &field_names {
                            serialize_fields.push(quote! {
                                bytes.extend(::afastdata_core::AFastSerialize::to_bytes(#fname));
                            });
                        }
                        arms.push(quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                bytes.extend(#tag.to_le_bytes());
                                #(#serialize_fields)*
                            }
                        });
                    }
                }
            }

            quote! {
                impl #impl_generics ::afastdata_core::AFastSerialize for #name #ty_generics {
                    fn to_bytes(&self) -> Vec<u8> {
                        let mut bytes = Vec::new();
                        match self {
                            #(#arms)*
                        }
                        bytes
                    }
                }
            }
        }
        Data::Union(_) => panic!("AFastSerialize does not support unions"),
    };

    TokenStream::from(expanded)
}

/// 为结构体或枚举生成 [`AFastDeserialize`] trait 实现。
///
/// Generates an [`AFastDeserialize`] trait implementation for a struct or enum.
///
/// # 生成的代码 / Generated Code
///
/// ## 结构体 / Struct
///
/// 为每个字段依次调用 `from_bytes()`，并使用偏移量追踪已消耗的字节数，
/// 最后构造结构体实例。
///
/// Calls `from_bytes()` on each field sequentially, using an offset to track
/// consumed bytes, then constructs the struct instance.
///
/// ```ignore
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastDeserialize for MyStruct {
///     fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
///         let mut offset: usize = 0;
///         let (__val, __new_offset) = AFastDeserialize::from_bytes(&data[offset..])?;
///         let field1 = __val;
///         offset += __new_offset;
///         // ... 更多字段 / more fields ...
///         Ok((MyStruct { field1, ... }, offset))
///     }
/// }
/// ```
///
/// ## 枚举 / Enum
///
/// 先读取 `u32` 变体索引，根据索引匹配对应变体，再反序列化该变体的字段。
///
/// First reads the `u32` variant index, matches the corresponding variant by index,
/// then deserializes the variant's fields.
///
/// ```ignore
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastDeserialize for MyEnum {
///     fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
///         let mut offset: usize = 0;
///         let (__tag, __new_offset) = <u32 as AFastDeserialize>::from_bytes(&data[offset..])?;
///         offset += __new_offset;
///         match __tag {
///             0 => Ok((MyEnum::Variant1, offset)),
///             1 => {
///                 let (__val, __new_offset) = AFastDeserialize::from_bytes(&data[offset..])?;
///                 offset += __new_offset;
///                 Ok((MyEnum::Variant2(__val), offset))
///             }
///             v => Err(format!("Unknown variant tag: {} for MyEnum", v)),
///         }
///     }
/// }
/// ```
///
/// # 泛型 / Generics
///
/// 如果目标类型包含泛型参数，生成的 `impl` 会同时添加 `AFastSerialize` 和
/// `AFastDeserialize` 约束。双重约束确保泛型类型在序列化和反序列化两个方向
/// 上都可用。
///
/// If the target type contains generic parameters, the generated `impl` adds both
/// `AFastSerialize` and `AFastDeserialize` bounds. The dual bounds ensure the generic
/// type is available in both serialization and deserialization directions.
///
/// # Panics
///
/// 对 union 类型使用此宏会触发编译 panic。
///
/// Using this macro on a union type will trigger a compile-time panic.
#[proc_macro_derive(AFastDeserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // 为泛型参数添加 AFastSerialize + AFastDeserialize trait 约束
    // Add AFastSerialize + AFastDeserialize trait bounds to generic parameters
    let mut generics_with_bounds = generics.clone();
    for param in &mut generics_with_bounds.params {
        if let syn::GenericParam::Type(ref mut ty) = *param {
            ty.bounds.push(syn::parse_quote!(::afastdata_core::AFastSerialize));
            ty.bounds.push(syn::parse_quote!(::afastdata_core::AFastDeserialize));
        }
    }
    let (impl_generics, _, _) = generics_with_bounds.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data) => {
            let (construct, field_desers) =
                generate_deserialize_fields(&data.fields, &name, &ty_generics);
            quote! {
                impl #impl_generics ::afastdata_core::AFastDeserialize for #name #ty_generics {
                    fn from_bytes(data: &[u8]) -> Result<(Self, usize), ::std::string::String> {
                        let mut offset: usize = 0;
                        #(#field_desers)*
                        Ok((#construct, offset))
                    }
                }
            }
        }
        Data::Enum(data) => {
            let mut arms = Vec::new();
            for (i, variant) in data.variants.iter().enumerate() {
                let variant_name = &variant.ident;
                let tag = i as u32;

                match &variant.fields {
                    Fields::Unit => {
                        // Unit 变体：匹配索引后直接返回
                        // Unit variant: match index and return directly
                        arms.push(quote! {
                            #tag => {
                                Ok((#name::#variant_name, offset))
                            }
                        });
                    }
                    Fields::Unnamed(fields) => {
                        // 元组变体：匹配索引后逐字段反序列化
                        // Tuple variant: match index, then deserialize fields sequentially
                        let mut field_desers = Vec::new();
                        let mut field_names = Vec::new();
                        for _ in &fields.unnamed {
                            let fname = syn::Ident::new(
                                &format!("__f{}", field_names.len()),
                                variant_name.span(),
                            );
                            field_desers.push(quote! {
                                let (__val, __new_offset) = ::afastdata_core::AFastDeserialize::from_bytes(&data[offset..])?;
                                let #fname = __val;
                                offset += __new_offset;
                            });
                            field_names.push(fname);
                        }
                        arms.push(quote! {
                            #tag => {
                                #(#field_desers)*
                                Ok((#name::#variant_name(#(#field_names),*), offset))
                            }
                        });
                    }
                    Fields::Named(fields) => {
                        // 命名字段变体：匹配索引后逐字段反序列化
                        // Named-field variant: match index, then deserialize fields sequentially
                        let mut field_desers = Vec::new();
                        let mut field_names = Vec::new();
                        for f in &fields.named {
                            let fname = f.ident.as_ref().unwrap();
                            field_desers.push(quote! {
                                let (__val, __new_offset) = ::afastdata_core::AFastDeserialize::from_bytes(&data[offset..])?;
                                let #fname = __val;
                                offset += __new_offset;
                            });
                            field_names.push(fname);
                        }
                        arms.push(quote! {
                            #tag => {
                                #(#field_desers)*
                                Ok((#name::#variant_name { #(#field_names),* }, offset))
                            }
                        });
                    }
                }
            }

            quote! {
                impl #impl_generics ::afastdata_core::AFastDeserialize for #name #ty_generics {
                    fn from_bytes(data: &[u8]) -> Result<(Self, usize), ::std::string::String> {
                        let mut offset: usize = 0;
                        // 读取 u32 变体索引
                        // Read the u32 variant index
                        let (__tag_bytes, __new_offset) = <u32 as ::afastdata_core::AFastDeserialize>::from_bytes(&data[offset..])?;
                        offset += __new_offset;
                        match __tag_bytes {
                            #(#arms)*
                            v => Err(::std::format!("Unknown variant tag: {} for {}", v, ::std::stringify!(#name))),
                        }
                    }
                }
            }
        }
        Data::Union(_) => panic!("AFastDeserialize does not support unions"),
    };

    TokenStream::from(expanded)
}

/// 为结构体的字段生成序列化代码。内部辅助函数。
///
/// Generates serialization code for struct fields. Internal helper.
///
/// # 参数 / Parameters
///
/// - `fields`：结构体的字段定义 / The struct's field definitions
/// - `self_prefix`：访问字段时使用的前缀（如 `self` 或变体解构变量）
///   / The prefix used to access fields (e.g., `self` or a variant destructure variable)
///
/// # 返回值 / Returns
///
/// 返回一个 `TokenStream` 列表，每个元素对应一个字段的序列化语句。
///
/// Returns a list of `TokenStream`s, each corresponding to a serialization statement
/// for one field.
///
/// # 生成格式 / Generated Format
///
/// - **命名字段 (Named)**：`bytes.extend(AFastSerialize::to_bytes(&self.field_name));`
/// - **元组字段 (Unnamed)**：`bytes.extend(AFastSerialize::to_bytes(&self.0));`
/// - **单元字段 (Unit)**：不生成任何代码 / Generates no code
fn generate_serialize_fields(
    fields: &Fields,
    self_prefix: proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let fname = f.ident.as_ref().unwrap();
                quote! {
                    bytes.extend(::afastdata_core::AFastSerialize::to_bytes(&#self_prefix.#fname));
                }
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let idx = Index::from(i);
                quote! {
                    bytes.extend(::afastdata_core::AFastSerialize::to_bytes(&#self_prefix.#idx));
                }
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

/// 为结构体的字段生成反序列化代码以及构造表达式。内部辅助函数。
///
/// Generates deserialization code for struct fields along with the construction
/// expression. Internal helper.
///
/// # 参数 / Parameters
///
/// - `fields`：结构体的字段定义 / The struct's field definitions
/// - `name`：结构体类型的标识符 / The struct type's identifier
/// - `ty_generics`：类型的泛型参数（用于构造时的 turbofish 语法）
///   / The type's generic parameters (used for turbofish syntax during construction)
///
/// # 返回值 / Returns
///
/// 返回 `(构造表达式, 反序列化语句列表)`：
/// - 构造表达式：用于创建结构体实例的 `TokenStream`
/// - 反序列化语句：每个字段的 `from_bytes()` 调用和偏移量更新
///
/// Returns `(construction_expression, deserialization_statements)`:
/// - Construction expression: A `TokenStream` for creating the struct instance
/// - Deserialization statements: `from_bytes()` calls and offset updates for each field
///
/// # 泛型构造 / Generic Construction
///
/// 使用 `as_turbofish()` 生成正确的泛型语法。例如 `MyStruct::<T>` 而非
/// `MyStruct <T>`（后者会被解析为比较操作）。
///
/// Uses `as_turbofish()` to generate correct generic syntax. For example,
/// `MyStruct::<T>` instead of `MyStruct <T>` (which would be parsed as a
/// comparison operation).
fn generate_deserialize_fields(
    fields: &Fields,
    name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    // 在表达式上下文中使用 turbofish 语法：Name::<T>
    // In expression context, use turbofish syntax: Name::<T>
    let ty_params = ty_generics.as_turbofish();
    match fields {
        Fields::Named(named) => {
            let mut desers = Vec::new();
            let mut field_names = Vec::new();
            for f in &named.named {
                let fname = f.ident.as_ref().unwrap();
                field_names.push(fname.clone());
                desers.push(quote! {
                    let (__val, __new_offset) = ::afastdata_core::AFastDeserialize::from_bytes(&data[offset..])?;
                    let #fname = __val;
                    offset += __new_offset;
                });
            }
            let construct = quote! {
                #name #ty_params { #(#field_names),* }
            };
            (construct, desers)
        }
        Fields::Unnamed(unnamed) => {
            let mut desers = Vec::new();
            let mut field_names = Vec::new();
            for i in 0..unnamed.unnamed.len() {
                let fname = syn::Ident::new(&format!("__f{}", i), name.span());
                desers.push(quote! {
                    let (__val, __new_offset) = ::afastdata_core::AFastDeserialize::from_bytes(&data[offset..])?;
                    let #fname = __val;
                    offset += __new_offset;
                });
                field_names.push(fname);
            }
            let construct = quote! {
                #name #ty_params ( #(#field_names),* )
            };
            (construct, desers)
        }
        Fields::Unit => {
            let construct = quote! { #name #ty_params };
            (construct, vec![])
        }
    }
}
