/// Declarative macro for one-step typed extraction from `ParseResult`.
///
/// Returns `Result<Struct, OptionError>` where the struct has a `pub` field
/// for each declared field.
///
/// # Supported field types
///
/// | Syntax | Behavior |
/// |--------|-----------|
/// | `name: bool` | Flag — calls `get_flag()` |
/// | `name: Option<T>` | Optional — calls `get_option()` + parse |
/// | `name: Vec<T>` | Multi-value — calls `get_option_values()` + parse each |
/// | `name: T as "cli-name" = expr` | Custom name with default |
/// | `name: T as "cli-name"` | Custom name, required — uses literal as option name |
/// | `name: T = expr` | Default — calls `get_option_or_default::<T>()` |
/// | `name: T` | Required — calls `get_option_required::<T>()` |
/// | `name: T as @pos` | Positional by index, required |
/// | `name: Option<T> as @pos` | Positional by index, optional (`None` when absent) |
/// | `name: Vec<T> as @pos` | Remaining positionals from current index onward |
/// | `name: T as @pos = expr` | Positional by index with default when absent |
///
/// Field names have underscores converted to hyphens for the CLI lookup
/// (e.g. `listen_port` → `"listen-port"`). Use a string literal to override
/// the name mapping (e.g. `port: u16 as "listen-port"`).
///
/// # Positional field ordering
///
/// Positional indices are assigned sequentially in declaration order among
/// `@pos` fields (non-positional fields do not consume indices). The
/// recommended declaration order is:
///
/// 1. Required positionals (`T as @pos`)
/// 2. Optional / default positionals (`Option<T> as @pos`, `T as @pos = expr`)
/// 3. Remaining positionals (`Vec<T> as @pos`)
///
/// `Vec<T> as @pos` consumes **all** remaining positionals from the current
/// index onward and must be the last positional field. Any `@pos` field
/// declared after it will always see an out-of-bounds index.
#[macro_export]
macro_rules! extract {
    // ── entry point: collect fields via TT munching ────────────────
    ($result:expr, { $($body:tt)* }) => {{
        let __res = &$result;
        (|| {
            #[allow(unused_mut, unused_variables)]
            let mut __pos_idx: usize = 0;
            #[allow(unused_variables)]
            let __positionals = __res.get_positionals();
            $crate::extract!(@normalize __res, __pos_idx, __positionals,
                { $($body)* }
            )
        })()
    }};

    // ── @normalize: always append trailing comma, then dispatch ────
    (@normalize $res:expr, $pos_idx:ident, $positionals:ident,
        { $($body:tt)* }
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            { /* bindings */ }
            { /* struct fields */ }
            { /* field names */ }
            $($body)* ,
        )
    };

    // ── TT muncher: flag field (bool) ──────────────────────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : bool , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: bool = {
                    let __name = stringify!($field_name).replace('_', "-");
                    $res.get_flag(&__name)
                };
            }
            {
                $($struct_fields)*
                pub $field_name : bool,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: optional positional (Option<T> as @pos) ────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : Option < $inner:ty > as @ pos , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: Option<$inner> = {
                    let __name = stringify!($field_name);
                    if $pos_idx < $positionals.len() {
                        let __val = $positionals[$pos_idx].parse::<$inner>().map_err(|e| $crate::OptionError::ParseFailed {
                            option: __name.to_string(),
                            message: e.to_string(),
                        })?;
                        $pos_idx += 1;
                        Some(__val)
                    } else {
                        None
                    }
                };
            }
            {
                $($struct_fields)*
                pub $field_name : Option<$inner>,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: Option<T> field ────────────────────────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : Option < $inner:ty > , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: Option<$inner> = {
                    let __name = stringify!($field_name).replace('_', "-");
                    match $res.get_option(&__name) {
                        Some(v) => Some(v.parse::<$inner>().map_err(|e| $crate::OptionError::ParseFailed {
                            option: __name.clone(),
                            message: e.to_string(),
                        })?),
                        None => None,
                    }
                };
            }
            {
                $($struct_fields)*
                pub $field_name : Option<$inner>,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: remaining positionals (Vec<T> as @pos) ─────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : Vec < $inner:ty > as @ pos , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: Vec<$inner> = {
                    let __name = stringify!($field_name);
                    let __slice = &$positionals[$pos_idx..];
                    let __parsed: Vec<$inner> = __slice.iter().map(|v| v.parse::<$inner>()).collect::<Result<Vec<$inner>, _>>().map_err(|e| $crate::OptionError::ParseFailed {
                        option: __name.to_string(),
                        message: e.to_string(),
                    })?;
                    $pos_idx = $positionals.len();
                    __parsed
                };
            }
            {
                $($struct_fields)*
                pub $field_name : Vec<$inner>,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: Vec<T> field ───────────────────────────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : Vec < $inner:ty > , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: Vec<$inner> = {
                    let __name = stringify!($field_name).replace('_', "-");
                    let __raw = $res.get_option_values(&__name);
                    __raw.iter().map(|v| v.parse::<$inner>()).collect::<Result<Vec<$inner>, _>>().map_err(|e| $crate::OptionError::ParseFailed {
                        option: __name.clone(),
                        message: e.to_string(),
                    })?
                };
            }
            {
                $($struct_fields)*
                pub $field_name : Vec<$inner>,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: default positional (T as @pos = expr) ──────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty as @ pos = $default:expr , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name = stringify!($field_name);
                    if $pos_idx < $positionals.len() {
                        let __val = $positionals[$pos_idx].parse::<$ty>().map_err(|e| $crate::OptionError::ParseFailed {
                            option: __name.to_string(),
                            message: e.to_string(),
                        })?;
                        $pos_idx += 1;
                        __val
                    } else {
                        $default
                    }
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: required positional (T as @pos) ────────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty as @ pos , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name = stringify!($field_name);
                    if $pos_idx < $positionals.len() {
                        let __val = $positionals[$pos_idx].parse::<$ty>().map_err(|e| $crate::OptionError::ParseFailed {
                            option: __name.to_string(),
                            message: e.to_string(),
                        })?;
                        $pos_idx += 1;
                        __val
                    } else {
                        return Err($crate::OptionError::Missing { option: __name.to_string() });
                    }
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: custom name with default (T as "name" = expr) ──
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty as $custom_name:literal = $default:expr , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name: &str = $custom_name;
                    $res.get_option_or_default::<$ty>(__name, $default)?
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: custom name required (T as "name") ─────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty as $custom_name:literal , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name: &str = $custom_name;
                    $res.get_option_required::<$ty>(__name)?
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: default field (T = expr) ───────────────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty = $default:expr , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name = stringify!($field_name).replace('_', "-");
                    $res.get_option_or_default::<$ty>(&__name, $default)?
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: required field (fallthrough type) ──────────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $field_name:ident : $ty:ty , $($rest:tt)*
    ) => {
        $crate::extract!(@munch $res, $pos_idx, $positionals,
            {
                $($bindings)*
                let $field_name: $ty = {
                    let __name = stringify!($field_name).replace('_', "-");
                    $res.get_option_required::<$ty>(&__name)?
                };
            }
            {
                $($struct_fields)*
                pub $field_name : $ty,
            }
            { $($names)* $field_name }
            $($rest)*
        )
    };

    // ── TT muncher: base case — all fields consumed, emit struct ───
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
    ) => {{
        $($bindings)*

        #[derive(Debug)]
        struct __ExtractedFields {
            $($struct_fields)*
        }

        Ok::<__ExtractedFields, $crate::OptionError>(__ExtractedFields {
            $($names,)*
        })
    }};

    // ── TT muncher: base case — trailing comma from normalize ──────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        ,
    ) => {{
        $($bindings)*

        #[derive(Debug)]
        struct __ExtractedFields {
            $($struct_fields)*
        }

        Ok::<__ExtractedFields, $crate::OptionError>(__ExtractedFields {
            $($names,)*
        })
    }};

    // ── TT muncher: catch-all — unsupported field syntax ───────────
    (@munch $res:expr, $pos_idx:ident, $positionals:ident,
        { $($bindings:tt)* }
        { $($struct_fields:tt)* }
        { $($names:ident)* }
        $($rest:tt)*
    ) => {
        compile_error!(concat!(
            "extract!: unsupported field syntax. Check the field starting near: ",
            stringify!($($rest)*)
        ))
    };
}
