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
//! ```
//! extern crate afastdata;
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
use syn::{
    Attribute, Data, DeriveInput, Fields, Index, Lit, LitInt, LitStr, Meta, Path, Token, Type,
    TypePath,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// 返回枚举变体标签的类型和字节大小，基于编译时 feature 配置。
///
/// Returns the enum variant tag type and byte size based on compile-time feature config.
fn tag_type() -> (proc_macro2::TokenStream, usize) {
    if cfg!(feature = "tag-u16") {
        (quote! { u16 }, 2)
    } else if cfg!(feature = "tag-u32") {
        (quote! { u32 }, 4)
    } else {
        (quote! { u8 }, 1)
    }
}

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
/// ```text
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastSerialize for MyStruct {
///     fn to_bytes(&self) -> Vec<u8> {
///         let mut bytes = Vec::new();
///         bytes.extend(AFastSerialize::to_bytes(&self.field1));
///         bytes.extend(AFastSerialize::to_bytes(&self.field2));
///         // ... 依次处理每个字段 / remaining fields processed similarly
///         bytes
///     }
/// }
/// ```
///
/// ## 枚举 / Enum
///
/// 先写入 `u8` 变体索引（从 0 开始），再写入变体的字段数据。
/// Unit 变体只写入索引。可通过 feature 切换为 `u16` 或 `u32`。
///
/// Writes a `u8` variant index (starting from 0), then the variant's field data.
/// Unit variants only write the index. Switchable to `u16` or `u32` via features.
///
/// ```text
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastSerialize for MyEnum {
///     fn to_bytes(&self) -> Vec<u8> {
///         let mut bytes = Vec::new();
///         match self {
///             MyEnum::Variant1 => {
///                 bytes.extend(0u8.to_le_bytes());
///             }
///             MyEnum::Variant2(field) => {
///                 bytes.extend(1u8.to_le_bytes());
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
#[proc_macro_derive(AFastSerialize, attributes(afast))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // 为泛型参数添加 AFastSerialize trait 约束
    // Add AFastSerialize trait bounds to generic parameters
    let mut generics_with_bounds = generics.clone();
    for param in &mut generics_with_bounds.params {
        if let syn::GenericParam::Type(ref mut ty) = *param {
            ty.bounds
                .push(syn::parse_quote!(::afastdata::AFastSerialize));
        }
    }
    let (impl_generics, _, _) = generics_with_bounds.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data) => {
            let serialize_body = generate_serialize_fields(&data.fields, quote!(self));
            quote! {
                impl #impl_generics ::afastdata::AFastSerialize for #name #ty_generics {
                    fn to_bytes(&self) -> Vec<u8> {
                        let mut bytes = Vec::new();
                        #(#serialize_body)*
                        bytes
                    }
                }
            }
        }
        Data::Enum(data) => {
            let (tag_ty, _) = tag_type();
            let mut arms = Vec::new();
            for (i, variant) in data.variants.iter().enumerate() {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Unit => {
                        arms.push(quote! {
                            #name::#variant_name => {
                                bytes.extend((#i as #tag_ty).to_le_bytes());
                            }
                        });
                    }
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| {
                                syn::Ident::new(&format!("__f{}", i), variant_name.span())
                            })
                            .collect();
                        let field_patterns = &field_names;
                        let mut serialize_fields = Vec::new();
                        for (i, f) in fields.unnamed.iter().enumerate() {
                            if !has_skip_attr(&f.attrs).0 {
                                let fname = &field_names[i];
                                serialize_fields.push(quote! {
                                    bytes.extend(::afastdata::AFastSerialize::to_bytes(#fname));
                                });
                            }
                        }
                        arms.push(quote! {
                            #name::#variant_name(#(#field_patterns),*) => {
                                bytes.extend((#i as #tag_ty).to_le_bytes());
                                #(#serialize_fields)*
                            }
                        });
                    }
                    Fields::Named(fields) => {
                        let all_field_names: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();
                        let non_skip_names: Vec<_> = fields
                            .named
                            .iter()
                            .filter(|f| !has_skip_attr(&f.attrs).0)
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();
                        let has_skip = non_skip_names.len() < all_field_names.len();
                        let mut serialize_fields = Vec::new();
                        for fname in &non_skip_names {
                            serialize_fields.push(quote! {
                                bytes.extend(::afastdata::AFastSerialize::to_bytes(#fname));
                            });
                        }
                        if has_skip {
                            arms.push(quote! {
                                #name::#variant_name { #(#non_skip_names),*, .. } => {
                                    bytes.extend((#i as #tag_ty).to_le_bytes());
                                    #(#serialize_fields)*
                                }
                            });
                        } else {
                            arms.push(quote! {
                                #name::#variant_name { #(#non_skip_names),* } => {
                                    bytes.extend((#i as #tag_ty).to_le_bytes());
                                    #(#serialize_fields)*
                                }
                            });
                        }
                    }
                }
            }

            quote! {
                impl #impl_generics ::afastdata::AFastSerialize for #name #ty_generics {
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
/// ```text
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastDeserialize for MyStruct {
///     fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
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
/// 先读取 `u8` 变体索引，根据索引匹配对应变体，再反序列化该变体的字段。
/// 可通过 feature 切换为 `u16` 或 `u32`。
///
/// First reads the `u8` variant index, matches the corresponding variant by index,
/// then deserializes the variant's fields.
/// Switchable to `u16` or `u32` via features.
///
/// ```text
/// // 以下为生成代码的示意（非实际代码）
/// // The following is an illustration of generated code (not actual code)
/// impl AFastDeserialize for MyEnum {
///     fn from_bytes(data: &[u8]) -> Result<(Self, usize), Error> {
///         let mut offset: usize = 0;
///         let (__tag, __new_offset) = <u8 as AFastDeserialize>::from_bytes(&data[offset..])?;
///         offset += __new_offset;
///         match __tag as usize {
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
#[proc_macro_derive(AFastDeserialize, attributes(afast))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    // 为泛型参数添加 AFastSerialize + AFastDeserialize trait 约束
    // Add AFastSerialize + AFastDeserialize trait bounds to generic parameters
    let mut generics_with_bounds = generics.clone();
    for param in &mut generics_with_bounds.params {
        if let syn::GenericParam::Type(ref mut ty) = *param {
            ty.bounds
                .push(syn::parse_quote!(::afastdata::AFastSerialize));
            ty.bounds
                .push(syn::parse_quote!(::afastdata::AFastDeserialize));
        }
    }
    let (impl_generics, _, _) = generics_with_bounds.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data) => {
            let (construct, field_desers) =
                generate_deserialize_fields(&data.fields, name, &ty_generics);
            quote! {
                impl #impl_generics ::afastdata::AFastDeserialize for #name #ty_generics {
                    fn from_bytes(data: &[u8]) -> Result<(Self, usize), ::afastdata::Error> {
                        let mut offset: usize = 0;
                        #(#field_desers)*
                        Ok((#construct, offset))
                    }
                }
            }
        }
        Data::Enum(data) => {
            let (tag_ty, _) = tag_type();
            let mut arms = Vec::new();
            for (i, variant) in data.variants.iter().enumerate() {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Unit => {
                        arms.push(quote! {
                            #i => {
                                Ok((#name::#variant_name, offset))
                            }
                        });
                    }
                    Fields::Unnamed(fields) => {
                        let mut field_desers = Vec::new();
                        let mut field_names = Vec::new();
                        for (i, f) in fields.unnamed.iter().enumerate() {
                            let fname = syn::Ident::new(
                                &format!("__f{}", i),
                                variant_name.span(),
                            );
                            let ftype = &f.ty;
                            let (skip, default_fn) = has_skip_attr(&f.attrs);
                            if skip {
                                if let Some(func_name) = default_fn {
                                    match syn::parse_str::<syn::Ident>(&func_name) {
                                        Ok(ident) => {
                                            field_desers.push(quote! {
                                                let #fname: #ftype = #ident();
                                            });
                                        }
                                        Err(_) => {
                                            field_desers.push(quote! {
                                                compile_error!(concat!("invalid function name in skip: ", #func_name));
                                            });
                                        }
                                    }
                                } else {
                                    field_desers.push(quote! {
                                        let #fname: #ftype = <#ftype as ::std::default::Default>::default();
                                    });
                                }
                            } else {
                                let validates = parse_validations(&fname, ftype, &f.attrs);
                                field_desers.push(quote! {
                                    let (__val, __new_offset) = ::afastdata::AFastDeserialize::from_bytes(&data[offset..])?;
                                    let #fname: #ftype = __val;
                                    #(#validates)*
                                    offset += __new_offset;
                                });
                            }
                            field_names.push(fname);
                        }
                        arms.push(quote! {
                            #i => {
                                #(#field_desers)*
                                Ok((#name::#variant_name(#(#field_names),*), offset))
                            }
                        });
                    }
                    Fields::Named(fields) => {
                        let mut field_desers = Vec::new();
                        let mut field_names = Vec::new();
                        for f in &fields.named {
                            let fname = f.ident.as_ref().unwrap();
                            let ftype = &f.ty;
                            let (skip, default_fn) = has_skip_attr(&f.attrs);
                            if skip {
                                if let Some(func_name) = default_fn {
                                    match syn::parse_str::<syn::Ident>(&func_name) {
                                        Ok(ident) => {
                                            field_desers.push(quote! {
                                                let #fname: #ftype = #ident();
                                            });
                                        }
                                        Err(_) => {
                                            field_desers.push(quote! {
                                                compile_error!(concat!("invalid function name in skip: ", #func_name));
                                            });
                                        }
                                    }
                                } else {
                                    field_desers.push(quote! {
                                        let #fname: #ftype = <#ftype as ::std::default::Default>::default();
                                    });
                                }
                            } else {
                                let validates = parse_validations(fname, ftype, &f.attrs);
                                field_desers.push(quote! {
                                    let (__val, __new_offset) = ::afastdata::AFastDeserialize::from_bytes(&data[offset..])?;
                                    let #fname: #ftype = __val;
                                    #(#validates)*
                                    offset += __new_offset;
                                });
                            }
                            field_names.push(fname);
                        }
                        arms.push(quote! {
                            #i => {
                                #(#field_desers)*
                                Ok((#name::#variant_name { #(#field_names),* }, offset))
                            }
                        });
                    }
                }
            }

            quote! {
                impl #impl_generics ::afastdata::AFastDeserialize for #name #ty_generics {
                    fn from_bytes(data: &[u8]) -> Result<(Self, usize), ::afastdata::Error> {
                        let mut offset: usize = 0;
                        let (__tag_bytes, __new_offset) = <#tag_ty as ::afastdata::AFastDeserialize>::from_bytes(&data[offset..])?;
                        offset += __new_offset;
                        match __tag_bytes as usize {
                            #(#arms)*
                            v => Err(::afastdata::Error::deserialize(format!("Unknown variant tag: {} for {}", v, ::std::stringify!(#name)))),
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
            .filter(|f| !has_skip_attr(&f.attrs).0)
            .map(|f| {
                let fname = f.ident.as_ref().unwrap();
                quote! {
                    bytes.extend(::afastdata::AFastSerialize::to_bytes(&#self_prefix.#fname));
                }
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .filter(|(_, f)| !has_skip_attr(&f.attrs).0)
            .map(|(i, _)| {
                let idx = Index::from(i);
                quote! {
                    bytes.extend(::afastdata::AFastSerialize::to_bytes(&#self_prefix.#idx));
                }
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

fn has_skip_attr(attrs: &[Attribute]) -> (bool, Option<String>) {
    for attr in attrs {
        if attr.path().is_ident("afast")
            && let Ok(nested) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        {
            for meta in nested {
                match meta {
                    Meta::Path(path) if path.is_ident("skip") => {
                        return (true, None);
                    }
                    Meta::List(meta_list) if meta_list.path.is_ident("skip") => {
                        if let Ok(lit_str) = syn::parse2::<LitStr>(meta_list.tokens.clone()) {
                            return (true, Some(lit_str.value()));
                        } else {
                            return (true, None);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    (false, None)
}

struct Range {
    int: LitInt,
    _comma1: Token![,],
    code: LitInt,
    _comma2: Token![,],
    msg: LitStr,
}

impl Parse for Range {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Range {
            int: input.parse()?,
            _comma1: input.parse()?,
            code: input.parse()?,
            _comma2: input.parse()?,
            msg: input.parse()?,
        })
    }
}

struct Length {
    min: LitInt,
    _comma1: Token![,],
    max: LitInt,
    _comma2: Token![,],
    code: LitInt,
    _comma3: Token![,],
    msg: LitStr,
}

impl Parse for Length {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Length {
            min: input.parse()?,
            _comma1: input.parse()?,
            max: input.parse()?,
            _comma2: input.parse()?,
            code: input.parse()?,
            _comma3: input.parse()?,
            msg: input.parse()?,
        })
    }
}

#[derive(Clone)]
enum ValidateValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl ValidateValue {
    /// 将值转换为代码生成中使用的 TokenStream
    ///
    /// 例如：
    /// ValidateValue::Int(42) → quote! { 42 }
    /// ValidateValue::Str("hello") → quote! { "hello" }
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            ValidateValue::Int(v) => quote! { #v },

            ValidateValue::Float(v) => {
                // 浮点数通过字符串解析来保持精度
                let v_str = v.to_string();
                v_str.parse().unwrap_or_else(|_| {
                    // 如果字符串解析失败，使用直接值
                    quote! { #v }
                })
            }

            ValidateValue::Bool(v) => quote! { #v },
            ValidateValue::Str(v) => quote! { #v },
        }
    }
}

struct OfValidator {
    allowed_values: Vec<ValidateValue>,
    code: syn::LitInt,
    msg: syn::LitStr,
}

impl Parse for OfValidator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);

        let mut values = Vec::new();

        if !content.is_empty() {
            loop {
                let lit = content.parse::<Lit>()?;

                let value = match lit {
                    Lit::Int(lit_int) => {
                        let int_value: i64 = lit_int.base10_parse()?;
                        ValidateValue::Int(int_value)
                    }

                    Lit::Float(lit_float) => {
                        let float_value: f64 = lit_float.base10_parse()?;
                        ValidateValue::Float(float_value)
                    }

                    Lit::Bool(lit_bool) => ValidateValue::Bool(lit_bool.value),

                    Lit::Str(lit_str) => ValidateValue::Str(lit_str.value()),

                    _ => {
                        return Err(syn::Error::new_spanned(
                            &lit,
                            "unsupported literal type in 'of' validator; \
                             only int, float, bool, and str literals are supported",
                        ));
                    }
                };

                values.push(value);

                if !content.peek(Token![,]) {
                    break;
                }

                content.parse::<Token![,]>()?;

                if content.is_empty() {
                    break;
                }
            }
        }

        input.parse::<Token![,]>()?;

        let code = input.parse::<syn::LitInt>()?;

        input.parse::<Token![,]>()?;

        let msg = input.parse::<syn::LitStr>()?;

        Ok(OfValidator {
            allowed_values: values,
            code,
            msg,
        })
    }
}

/// 解析字段上的 `#[afast(...)]` 校验属性，生成校验代码。
///
/// Parses `#[afast(...)]` validation attributes on a field and generates
/// validation code blocks.
///
/// # 参数 / Parameters
///
/// - `field_name`：字段名 / Field name
/// - `field_type`：字段类型 / Field type
/// - `attrs`：字段的属性列表 / Field attributes
///
/// # 返回值 / Returns
///
/// 返回校验语句的 `TokenStream` 列表。
///
/// Returns a list of validation `TokenStream` blocks.
fn parse_validations(
    field_name: &syn::Ident,
    field_type: &Type,
    attrs: &[Attribute],
) -> Vec<proc_macro2::TokenStream> {
    let mut validates = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("afast") {
            let nested = match attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            {
                Ok(n) => n,
                Err(e) => {
                    validates.push(e.to_compile_error());
                    continue;
                }
            };
            for meta in nested {
                if let Meta::List(meta) = meta {
                        if meta.path.is_ident("gt") {
                            let inner = match meta.parse_args::<Range>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let gt_value = match inner.int.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.int, format!("invalid integer value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            validates.push(quote! {
                                if #field_name <= #gt_value {
                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                }
                            });
                        } else if meta.path.is_ident("gte") {
                            let inner = match meta.parse_args::<Range>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let gt_value = match inner.int.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.int, format!("invalid integer value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            validates.push(quote! {
                                if #field_name < #gt_value {
                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                }
                            });
                        } else if meta.path.is_ident("lt") {
                            let inner = match meta.parse_args::<Range>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let lt_value = match inner.int.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.int, format!("invalid integer value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            validates.push(quote! {
                                if #field_name >= #lt_value {
                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                }
                            });
                        } else if meta.path.is_ident("lte") {
                            let inner = match meta.parse_args::<Range>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let lt_value = match inner.int.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.int, format!("invalid integer value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            validates.push(quote! {
                                if #field_name > #lt_value {
                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                }
                            });
                        } else if meta.path.is_ident("len") {
                            let field_is_option = match field_type {
                                Type::Path(TypePath {
                                    path: Path { segments, .. },
                                    ..
                                }) => {
                                    segments.len() == 1 && segments[0].ident == "Option"
                                }
                                _ => false,
                            };

                            let inner = match meta.parse_args::<Length>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let min_value = match inner.min.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.min, format!("invalid min value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let max_value = match inner.max.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.max, format!("invalid max value: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            if min_value > max_value {
                                validates.push(
                                    syn::Error::new_spanned(
                                        &meta.path,
                                        format!(
                                            "invalid len validation: min ({}) > max ({}) for field `{}`",
                                            min_value, max_value, field_name
                                        ),
                                    )
                                    .to_compile_error(),
                                );
                                continue;
                            }
                            if min_value < 0 && max_value < 0 {
                                validates.push(
                                    syn::Error::new_spanned(
                                        &meta.path,
                                        format!(
                                            "invalid len validation: both min and max are negative for field `{}`",
                                            field_name
                                        ),
                                    )
                                    .to_compile_error(),
                                );
                                continue;
                            } else if min_value < 0 {
                                let max: usize = match max_value.try_into() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        validates.push(
                                            syn::Error::new_spanned(&inner.max, "value too large for usize")
                                                .to_compile_error(),
                                        );
                                        continue;
                                    }
                                };
                                validates.push(quote! {
                                    if #field_name.len() > #max {
                                        return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                    }
                                });
                            } else if max_value < 0 {
                                let min: usize = match min_value.try_into() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        validates.push(
                                            syn::Error::new_spanned(&inner.min, "value too large for usize")
                                                .to_compile_error(),
                                        );
                                        continue;
                                    }
                                };
                                validates.push(quote! {
                                    if #field_name.len() < #min {
                                        return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                    }
                                });
                            } else {
                                let min: usize = match min_value.try_into() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        validates.push(
                                            syn::Error::new_spanned(&inner.min, "value too large for usize")
                                                .to_compile_error(),
                                        );
                                        continue;
                                    }
                                };
                                let max: usize = match max_value.try_into() {
                                    Ok(v) => v,
                                    Err(_) => {
                                        validates.push(
                                            syn::Error::new_spanned(&inner.max, "value too large for usize")
                                                .to_compile_error(),
                                        );
                                        continue;
                                    }
                                };
                                if field_is_option {
                                    validates.push(quote! {
                                        let length = match &#field_name {
                                            Some(s) => {
                                                let __length = s.len();
                                                if __length < #min || __length > #max {
                                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                                }
                                            },
                                            None => {},
                                        };
                                    });
                                } else {
                                    validates.push(quote! {
                                        if #field_name.len() < #min || #field_name.len() > #max {
                                            return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                        }
                                    });
                                }
                            }
                        } else if meta.path.is_ident("of") {
                            let inner = match meta.parse_args::<OfValidator>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let allowed_values = inner.allowed_values.clone();
                            let code = match inner.code.base10_parse::<i64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(&inner.code, format!("invalid error code: {}", e))
                                            .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let err_msg = inner
                                .msg
                                .value()
                                .replace("${field}", &field_name.to_string());
                            let values_tokens: Vec<_> = allowed_values
                                .iter()
                                .map(|v| v.to_token_stream())
                                .collect();
                            validates.push(quote! {
                                if !matches!(#field_name, #(#values_tokens)|*) {
                                    return Err(::afastdata::Error::validate(#code, #err_msg.to_string()));
                                }
                            });
                        } else if meta.path.is_ident("func") {
                            let inner = match meta.parse_args::<LitStr>() {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(e.to_compile_error());
                                    continue;
                                }
                            };
                            let ident = match syn::parse_str::<syn::Ident>(&inner.value()) {
                                Ok(v) => v,
                                Err(e) => {
                                    validates.push(
                                        syn::Error::new_spanned(
                                            &inner,
                                            format!("invalid function name `{}`: {}", inner.value(), e),
                                        )
                                        .to_compile_error(),
                                    );
                                    continue;
                                }
                            };
                            let field = field_name.to_string();
                            validates.push(quote! {
                                match #ident(&#field_name, #field) {
                                    Ok(()) => {},
                                    Err(e) => return Err(e.to_afastdata_error()),
                                }
                            });
                        }
                }
            }
        }
    }
    validates
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
                let ftype = &f.ty;
                field_names.push(fname.clone());

                let validates = parse_validations(fname, ftype, &f.attrs);
                let (skip, default) = has_skip_attr(&f.attrs);
                if skip {
                    if let Some(default) = default {
                        match syn::parse_str::<syn::Ident>(&default) {
                            Ok(ident) => {
                                desers.push(quote! {
                                    let #fname: #ftype = #ident();
                                });
                            }
                            Err(_) => {
                                desers.push(quote! {
                                    compile_error!(concat!("invalid function name in skip: ", #default));
                                });
                            }
                        }
                    } else {
                        desers.push(quote! {
                            let #fname: #ftype = #ftype::default();
                        });
                    }
                } else {
                    desers.push(quote! {
                        let (__val, __new_offset) = ::afastdata::AFastDeserialize::from_bytes(&data[offset..])?;
                        let #fname: #ftype = __val;
                        #(#validates)*
                        offset += __new_offset;
                    });
                }
            }
            let construct = quote! {
                #name #ty_params { #(#field_names),* }
            };
            (construct, desers)
        }
        Fields::Unnamed(unnamed) => {
            let mut desers = Vec::new();
            let mut field_names = Vec::new();
            for (i, f) in unnamed.unnamed.iter().enumerate() {
                let fname = syn::Ident::new(&format!("__f{}", i), name.span());
                let ftype = &f.ty;
                let (skip, default_fn) = has_skip_attr(&f.attrs);
                if skip {
                    if let Some(func_name) = default_fn {
                        match syn::parse_str::<syn::Ident>(&func_name) {
                            Ok(ident) => {
                                desers.push(quote! {
                                    let #fname: #ftype = #ident();
                                });
                            }
                            Err(_) => {
                                desers.push(quote! {
                                    compile_error!(concat!("invalid function name in skip: ", #func_name));
                                });
                            }
                        }
                    } else {
                        desers.push(quote! {
                            let #fname: #ftype = <#ftype as ::std::default::Default>::default();
                        });
                    }
                } else {
                    let validates = parse_validations(&fname, ftype, &f.attrs);
                    desers.push(quote! {
                        let (__val, __new_offset) = ::afastdata::AFastDeserialize::from_bytes(&data[offset..])?;
                        let #fname: #ftype = __val;
                        #(#validates)*
                        offset += __new_offset;
                    });
                }
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
