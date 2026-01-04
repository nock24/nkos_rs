use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::parse::{FieldType, SectorLayout};

pub fn expand_layout(layout: SectorLayout) -> TokenStream {
    // ---- semantic checks ----
    let mut dyn_idxs: Vec<usize> = Vec::new();
    for (i, f) in layout.fields.iter().enumerate() {
        if matches!(f.ty, FieldType::DynArr { .. }) {
            dyn_idxs.push(i);
        }
    }

    if dyn_idxs.len() > 1 {
        return syn::Error::new_spanned(
            &layout.name,
            "only one dynamic array field ([T; len]) is allowed",
        )
        .to_compile_error();
    }

    if let Some(&idx) = dyn_idxs.first() {
        if idx != layout.fields.len() - 1 {
            return syn::Error::new_spanned(
                &layout.fields[idx].name,
                "dynamic array field ([T; len]) must be the last field",
            )
            .to_compile_error();
        }
    }

    let name = &layout.name;
    let vis = &layout.vis;

    // ---- compute offsets + MIN_SIZE ----
    let mut offset_expr: TokenStream = quote!(0usize);
    let mut min_size_expr: TokenStream = quote!(0usize);
    let mut offset_consts: Vec<TokenStream> = Vec::new();

    let mut seen_fields: Vec<Ident> = Vec::new();

    struct DynInfo {
        field_name: Ident,
        elem_ty: Type,
        len_ident: Ident,
    }
    let mut dyn_info: Option<DynInfo> = None;

    for f in &layout.fields {
        let fname = &f.name;
        let off_ident = format_ident!("{}_OFFSET", fname.to_string().to_uppercase());

        // Associated const on the ZST
        offset_consts.push(quote! {
            pub const #off_ident: usize = #offset_expr;
        });

        match &f.ty {
            FieldType::Fixed(ty) => {
                let Some(_bits) = unsigned_bits(ty) else {
                    return syn::Error::new_spanned(
                        ty,
                        "unsupported fixed field type (v1 supports: u8/u16/u32/u64)",
                    )
                    .to_compile_error();
                };

                min_size_expr = quote!(#min_size_expr + core::mem::size_of::<#ty>());
                offset_expr = quote!(#offset_expr + core::mem::size_of::<#ty>());
            }

            FieldType::FixedArr { elem_ty, len } => {
                let Some(_bits) = unsigned_bits(elem_ty) else {
                    return syn::Error::new_spanned(
                        elem_ty,
                        "unsupported array element type (v1 supports: u8/u16/u32/u64)",
                    )
                    .to_compile_error();
                };

                let n = *len;
                min_size_expr =
                    quote!(#min_size_expr + (#n * core::mem::size_of::<#elem_ty>()));
                offset_expr = quote!(#offset_expr + (#n * core::mem::size_of::<#elem_ty>()));
            }

            FieldType::DynArr { elem_ty, len_ident } => {
                let Some(_bits) = unsigned_bits(elem_ty) else {
                    return syn::Error::new_spanned(
                        elem_ty,
                        "unsupported dynamic array element type (v1 supports: u8/u16/u32/u64)",
                    )
                    .to_compile_error();
                };

                if !seen_fields.iter().any(|id| id == len_ident) {
                    return syn::Error::new_spanned(
                        len_ident,
                        "length identifier must refer to a previous field",
                    )
                    .to_compile_error();
                }

                dyn_info = Some(DynInfo {
                    field_name: fname.clone(),
                    elem_ty: elem_ty.clone(),
                    len_ident: len_ident.clone(),
                });

                // dynamic does not contribute to MIN_SIZE and does not advance offset_expr
            }
        }

        seen_fields.push(fname.clone());
    }

    // ---- fixed field + fixed array accessors ----
    let mut accessors: Vec<TokenStream> = Vec::new();

    for f in &layout.fields {
        let fname = &f.name;
        let off_ident = format_ident!("{}_OFFSET", fname.to_string().to_uppercase());
        let off_expr = quote!(Self::#off_ident);

        match &f.ty {
            FieldType::Fixed(ty) => {
                let bits = unsigned_bits(ty).unwrap();
                let getter = fname;
                let setter = format_ident!("set_{}", fname);

                let read = read_int_expr(bits, off_expr.clone());
                let write = write_int_expr(bits, off_expr.clone());

                accessors.push(quote! {
                    pub fn #getter(buf: &[u8]) -> #ty {
                        #read
                    }

                    pub fn #setter(buf: &mut [u8], v: #ty) {
                        #write
                    }
                });
            }

            FieldType::FixedArr { elem_ty, len } => {
                let elem_bits = unsigned_bits(elem_ty).unwrap();
                let elem_size = (elem_bits / 8) as usize;
                let n = *len;

                let len_fn = format_ident!("{}_len", fname);
                let get_fn = format_ident!("{}_get", fname);
                let set_fn = format_ident!("{}_set", fname);
                let read_fn = format_ident!("{}_read", fname);
                let write_fn = format_ident!("{}_write", fname);

                let read_elem =
                    read_int_expr(elem_bits, quote!(Self::#off_ident + i * #elem_size));
                let write_elem =
                    write_int_expr(elem_bits, quote!(Self::#off_ident + i * #elem_size));

                accessors.push(quote! {
                    pub fn #len_fn() -> usize { #n }

                    pub fn #get_fn(buf: &[u8], i: usize) -> #elem_ty {
                        assert!(i < #n, "index out of range");
                        #read_elem
                    }

                    pub fn #set_fn(buf: &mut [u8], i: usize, v: #elem_ty) {
                        assert!(i < #n, "index out of range");
                        #write_elem
                    }

                    pub fn #read_fn(buf: &[u8], dst: &mut [#elem_ty]) {
                        assert!(dst.len() == #n, "wrong output length");
                        for i in 0..#n {
                            dst[i] = Self::#get_fn(buf, i);
                        }
                    }

                    pub fn #write_fn(buf: &mut [u8], src: &[#elem_ty]) {
                        assert!(src.len() == #n, "wrong input length");
                        for i in 0..#n {
                            Self::#set_fn(buf, i, src[i]);
                        }
                    }
                });
            }

            FieldType::DynArr { .. } => {
                // handled below
            }
        }
    }

    // ---- dynamic array accessors + validate ----
    let (dyn_accessors, validate_fn) = if let Some(di) = dyn_info {
        let field_name = di.field_name;
        let elem_ty = di.elem_ty;
        let len_ident = di.len_ident;

        let field_off_ident = format_ident!("{}_OFFSET", field_name.to_string().to_uppercase());
        let len_off_ident = format_ident!("{}_OFFSET", len_ident.to_string().to_uppercase());

        let Some(len_bits) = find_fixed_field_bits(&layout, &len_ident) else {
            return syn::Error::new_spanned(
                len_ident,
                "length field must be a fixed unsigned int: u8/u16/u32/u64",
            )
            .to_compile_error();
        };

        let elem_bits = unsigned_bits(&elem_ty).unwrap();
        let elem_size = (elem_bits / 8) as usize;

        let len_read_expr = read_int_expr(len_bits, quote!(Self::#len_off_ident));

        let len_fn = format_ident!("{}_len", field_name);
        let bytes_fn = format_ident!("{}_bytes", field_name);
        let get_fn = format_ident!("{}_get", field_name);
        let set_fn = format_ident!("{}_set", field_name);
        let read_fn = format_ident!("{}_read", field_name);
        let write_fn = format_ident!("{}_write", field_name);

        let read_elem_expr =
            read_int_expr(elem_bits, quote!(Self::#field_off_ident + i * #elem_size));
        let write_elem_expr =
            write_int_expr(elem_bits, quote!(Self::#field_off_ident + i * #elem_size));

        let dyn_accessors = quote! {
            pub fn #len_fn(buf: &[u8]) -> usize {
                (#len_read_expr) as usize
            }

            pub fn #bytes_fn(buf: &[u8]) -> usize {
                Self::#len_fn(buf) * #elem_size
            }

            pub fn #get_fn(buf: &[u8], i: usize) -> #elem_ty {
                let n = Self::#len_fn(buf);
                assert!(i < n, "index out of range");
                #read_elem_expr
            }

            pub fn #set_fn(buf: &mut [u8], i: usize, v: #elem_ty) {
                let n = Self::#len_fn(buf);
                assert!(i < n, "index out of range");
                #write_elem_expr
            }

            pub fn #read_fn(buf: &[u8], dst: &mut [#elem_ty]) {
                let n = Self::#len_fn(buf);
                assert!(dst.len() == n, "wrong output length");
                for i in 0..n {
                    dst[i] = Self::#get_fn(buf, i);
                }
            }

            pub fn #write_fn(buf: &mut [u8], src: &[#elem_ty]) {
                let n = Self::#len_fn(buf);
                assert!(src.len() == n, "wrong input length");
                for i in 0..n {
                    Self::#set_fn(buf, i, src[i]);
                }
            }
        };

        let validate_fn = quote! {
            pub fn validate(buf: &[u8]) -> bool {
                if buf.len() < Self::MIN_SIZE {
                    return false;
                }
                let n = Self::#len_fn(buf);

                let bytes = match n.checked_mul(#elem_size) {
                    Some(b) => b,
                    None => return false,
                };
                let need = match Self::MIN_SIZE.checked_add(bytes) {
                    Some(t) => t,
                    None => return false,
                };
                need <= buf.len()
            }
        };

        (dyn_accessors, validate_fn)
    } else {
        let validate_fn = quote! {
            pub fn validate(buf: &[u8]) -> bool {
                buf.len() >= Self::MIN_SIZE
            }
        };
        (quote! {}, validate_fn)
    };

    // ✅ ZST + impl, containing *everything*
    quote! {
        #vis struct #name;

        impl #name {
            #(#offset_consts)*

            /// Minimum required buffer length for all fixed-size fields.
            pub const MIN_SIZE: usize = #min_size_expr;

            #(#accessors)*

            #dyn_accessors

            #validate_fn
        }
    }
}

fn ty_path_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            Some(p.path.segments[0].ident.to_string())
        }
        _ => None,
    }
}

fn unsigned_bits(ty: &Type) -> Option<u32> {
    match ty_path_ident(ty)?.as_str() {
        "u8" => Some(8),
        "u16" => Some(16),
        "u32" => Some(32),
        "u64" => Some(64),
        _ => None,
    }
}

fn find_fixed_field_bits(layout: &SectorLayout, name: &Ident) -> Option<u32> {
    let Some(field) = layout.fields.iter().find(|&f| &f.name == name) else {
        return None;
    };
    if let FieldType::Fixed(ty) = &field.ty {
        unsigned_bits(ty)
    } else {
        None
    }
}

// Little-endian reads from `buf` at `off`.
fn read_int_expr(bits: u32, off: TokenStream) -> TokenStream {
    match bits {
        8 => quote! { buf[#off] as u8 },
        16 => quote! {{
            let b = [buf[#off], buf[#off + 1]];
            u16::from_le_bytes(b)
        }},
        32 => quote! {{
            let b = [buf[#off], buf[#off + 1], buf[#off + 2], buf[#off + 3]];
            u32::from_le_bytes(b)
        }},
        64 => quote! {{
            let b = [
                buf[#off], buf[#off + 1], buf[#off + 2], buf[#off + 3],
                buf[#off + 4], buf[#off + 5], buf[#off + 6], buf[#off + 7],
            ];
            u64::from_le_bytes(b)
        }},
        _ => quote! { compile_error!("unsupported int width"); },
    }
}

// Little-endian writes into `buf` at `off` (expects `v` in scope).
fn write_int_expr(bits: u32, off: TokenStream) -> TokenStream {
    match bits {
        8 => quote! { buf[#off] = v as u8; },
        16 => quote! {{
            let b = (v as u16).to_le_bytes();
            buf[#off] = b[0];
            buf[#off + 1] = b[1];
        }},
        32 => quote! {{
            let b = (v as u32).to_le_bytes();
            buf[#off..#off + 4].copy_from_slice(&b);
        }},
        64 => quote! {{
            let b = (v as u64).to_le_bytes();
            buf[#off..#off + 8].copy_from_slice(&b);
        }},
        _ => quote! { compile_error!("unsupported int width"); },
    }
}

