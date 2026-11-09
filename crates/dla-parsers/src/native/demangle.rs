//! Rust symbol demangling: the legacy scheme (`_ZN...17h<16 hex>E`) and the v0 scheme (`_R...`).
//!
//! This is a small own implementation of the rustc-demangle algorithm, without the crate.
//! Output follows the "alternate" form of rustc-demangle: no hash, no instantiating crate.
//!
//! Supported v0 subset: crate roots, nested paths (values, types, closures and shims), inherent
//! and trait impl paths, generic arguments (types, constants, lifetimes are dropped), basic
//! types, references, raw pointers, slices, arrays, tuples, function pointers, `dyn` types and
//! back references. Punycode identifiers are not decoded; the symbol is then not demangled.
//!
//! Closures print as `{closure}` for disambiguator 0 and `{closure#N}` otherwise (legacy
//! symbols keep the `{{closure}}` spelling the compiler wrote).
//!
//! C++ (Itanium) names such as `_ZN3foo3barEv` are left as they are: [`demangle`] returns
//! `None` for them. A legacy Rust symbol always ends right after its closing `E`, which is how
//! the two are told apart.

/// Demangles a Rust symbol. `None` when `symbol` is not a Rust symbol (or is one this
/// implementation cannot read), so a caller keeps the original text.
pub fn demangle(symbol: &str) -> Option<String> {
    if symbol.starts_with("_R") {
        v0(symbol)
    } else if symbol.starts_with("_ZN") || symbol.starts_with("__ZN") {
        legacy(symbol)
    } else {
        None
    }
}

/// [`demangle`], or `symbol` itself when it cannot be demangled.
pub fn demangle_or_raw(symbol: &str) -> String {
    demangle(symbol).unwrap_or_else(|| symbol.to_owned())
}

/// Whether `symbol` looks like a mangled Rust name (either scheme).
pub fn is_rust_symbol(symbol: &str) -> bool {
    demangle(symbol).is_some()
}

// ---------------------------------------------------------------------------------------
// Legacy scheme
// ---------------------------------------------------------------------------------------

fn legacy(symbol: &str) -> Option<String> {
    let mut rest = symbol
        .strip_prefix("_ZN")
        .or_else(|| symbol.strip_prefix("__ZN"))?;
    let mut parts: Vec<&str> = Vec::new();
    loop {
        if let Some(r) = rest.strip_prefix('E') {
            rest = r;
            break;
        }
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || digits > 6 {
            return None;
        }
        let len: usize = rest[..digits].parse().ok()?;
        let end = digits.checked_add(len)?;
        parts.push(rest.get(digits..end)?);
        rest = &rest[end..];
    }
    // Rust symbols end at the `E`; a linker suffix such as `.llvm.123` is allowed.
    if !(rest.is_empty() || rest.starts_with('.')) || parts.is_empty() {
        return None;
    }
    if parts.len() > 1 && parts.last().is_some_and(|p| is_legacy_hash(p)) {
        parts.pop();
    }
    let names: Vec<String> = parts.iter().map(|p| legacy_ident(p)).collect();
    Some(names.join("::"))
}

fn is_legacy_hash(s: &str) -> bool {
    s.len() == 17 && s.starts_with('h') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn legacy_ident(ident: &str) -> String {
    let mut rest = ident
        .strip_prefix('_')
        .filter(|r| r.starts_with('$'))
        .unwrap_or(ident);
    let mut out = String::with_capacity(ident.len());
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("..") {
            out.push_str("::");
            rest = r;
        } else if rest.starts_with('$') {
            let Some(end) = rest[1..].find('$') else {
                out.push_str(rest);
                break;
            };
            let escape = &rest[1..1 + end];
            match legacy_escape(escape) {
                Some(c) => {
                    out.push(c);
                    rest = &rest[end + 2..];
                }
                None => {
                    out.push_str(rest);
                    break;
                }
            }
        } else {
            let stop = rest
                .char_indices()
                .skip(1)
                .find(|&(_, c)| c == '.' || c == '$')
                .map_or(rest.len(), |(i, _)| i);
            out.push_str(&rest[..stop]);
            rest = &rest[stop..];
        }
    }
    out
}

fn legacy_escape(escape: &str) -> Option<char> {
    Some(match escape {
        "SP" => '@',
        "BP" => '*',
        "RF" => '&',
        "LT" => '<',
        "GT" => '>',
        "LP" => '(',
        "RP" => ')',
        "C" => ',',
        _ => {
            let hex = escape.strip_prefix('u')?;
            char::from_u32(u32::from_str_radix(hex, 16).ok()?)?
        }
    })
}

// ---------------------------------------------------------------------------------------
// v0 scheme
// ---------------------------------------------------------------------------------------

const MAX_DEPTH: u32 = 200;
const MAX_OUT: usize = 4096;

fn v0(symbol: &str) -> Option<String> {
    let body = symbol.strip_prefix("_R")?;
    // Drop a linker suffix (`.llvm.123`); v0 itself only uses [A-Za-z0-9_].
    let body = body.split('.').next()?;
    if !body.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return None;
    }
    let mut p = V0 {
        s: body.as_bytes(),
        pos: 0,
        depth: 0,
        on: true,
        out: String::new(),
    };
    p.path(true)?;
    // Optional instantiating crate: parsed, not printed.
    if p.pos < p.s.len() {
        p.on = false;
        p.path(false)?;
        p.on = true;
    }
    if p.pos != p.s.len() {
        return None;
    }
    Some(p.out)
}

struct V0<'a> {
    s: &'a [u8],
    pos: usize,
    depth: u32,
    /// Whether output is produced (off while skipping a part).
    on: bool,
    out: String,
}

impl V0<'_> {
    fn emit(&mut self, text: &str) -> Option<()> {
        if self.on {
            self.out.push_str(text);
            if self.out.len() > MAX_OUT {
                return None;
            }
        }
        Some(())
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn eat(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// `_` is 0, otherwise the base-62 digits up to `_` plus one.
    fn base62(&mut self) -> Option<u64> {
        if self.eat(b'_') {
            return Some(0);
        }
        let mut x: u64 = 0;
        while !self.eat(b'_') {
            let d = match self.next()? {
                c @ b'0'..=b'9' => c - b'0',
                c @ b'a'..=b'z' => c - b'a' + 10,
                c @ b'A'..=b'Z' => c - b'A' + 36,
                _ => return None,
            };
            x = x.checked_mul(62)?.checked_add(u64::from(d))?;
        }
        x.checked_add(1)
    }

    fn disambiguator(&mut self) -> Option<u64> {
        if self.eat(b's') {
            self.base62()?.checked_add(1)
        } else {
            Some(0)
        }
    }

    fn ident(&mut self) -> Option<&str> {
        if self.eat(b'u') {
            // Punycode identifiers are not decoded.
            return None;
        }
        let mut len: usize = 0;
        let mut digits = 0;
        // A decimal number is `0` alone or starts with a non-zero digit.
        while let Some(c @ b'0'..=b'9') = self.peek() {
            self.pos += 1;
            digits += 1;
            len = len.checked_mul(10)?.checked_add(usize::from(c - b'0'))?;
            if digits > 6 || c == b'0' && digits == 1 {
                break;
            }
        }
        if digits == 0 {
            return None;
        }
        self.eat(b'_');
        let end = self.pos.checked_add(len)?;
        let bytes = self.s.get(self.pos..end)?;
        self.pos = end;
        std::str::from_utf8(bytes).ok()
    }

    fn backref(&mut self, f: fn(&mut Self) -> Option<()>) -> Option<()> {
        // The `B` is already consumed.
        let start = self.pos - 1;
        let target = usize::try_from(self.base62()?).ok()?;
        if target >= start || self.depth >= MAX_DEPTH {
            return None;
        }
        if !self.on {
            // Nothing is printed, so do not follow it (keeps nested backrefs linear).
            return Some(());
        }
        let saved = self.pos;
        self.pos = target;
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        self.pos = saved;
        r
    }

    fn path(&mut self, in_value: bool) -> Option<()> {
        if self.depth >= MAX_DEPTH {
            return None;
        }
        self.depth += 1;
        let r = self.path_inner(in_value);
        self.depth -= 1;
        r
    }

    fn path_inner(&mut self, in_value: bool) -> Option<()> {
        match self.next()? {
            b'C' => {
                self.disambiguator()?;
                let name = self.ident()?.to_owned();
                self.emit(&name)
            }
            b'N' => {
                let ns = self.next()?;
                if !ns.is_ascii_alphabetic() {
                    return None;
                }
                self.path(in_value)?;
                let dis = self.disambiguator()?;
                let name = self.ident()?.to_owned();
                if ns.is_ascii_uppercase() {
                    self.emit("::{")?;
                    match ns {
                        b'C' => self.emit("closure")?,
                        b'S' => self.emit("shim")?,
                        other => self.emit(&char::from(other).to_string())?,
                    }
                    if !name.is_empty() {
                        self.emit(":")?;
                        self.emit(&name)?;
                    }
                    if dis > 0 {
                        self.emit(&format!("#{dis}"))?;
                    }
                    self.emit("}")
                } else if name.is_empty() {
                    Some(())
                } else {
                    self.emit("::")?;
                    self.emit(&name)
                }
            }
            tag @ (b'M' | b'X' | b'Y') => {
                if tag != b'Y' {
                    // The impl path: parsed and not printed.
                    self.disambiguator()?;
                    let was = std::mem::replace(&mut self.on, false);
                    let r = self.path(false);
                    self.on = was;
                    r?;
                }
                self.emit("<")?;
                self.type_()?;
                if tag != b'M' {
                    self.emit(" as ")?;
                    self.path(false)?;
                }
                self.emit(">")
            }
            b'I' => {
                self.path(in_value)?;
                if in_value {
                    self.emit("::")?;
                }
                self.emit("<")?;
                self.generic_args()?;
                self.emit(">")
            }
            b'B' => self.backref(|s| s.path(false)),
            _ => None,
        }
    }

    /// Generic arguments up to and including the closing `E`. Lifetimes are dropped.
    fn generic_args(&mut self) -> Option<()> {
        let mut first = true;
        while !self.eat(b'E') {
            if self.eat(b'L') {
                self.base62()?;
                continue;
            }
            if !first {
                self.emit(", ")?;
            }
            first = false;
            if self.eat(b'K') {
                self.constant()?;
            } else {
                self.type_()?;
            }
        }
        Some(())
    }

    fn basic_type(tag: u8) -> Option<&'static str> {
        Some(match tag {
            b'a' => "i8",
            b'b' => "bool",
            b'c' => "char",
            b'd' => "f64",
            b'e' => "str",
            b'f' => "f32",
            b'h' => "u8",
            b'i' => "isize",
            b'j' => "usize",
            b'l' => "i32",
            b'm' => "u32",
            b'n' => "i128",
            b'o' => "u128",
            b's' => "i16",
            b't' => "u16",
            b'u' => "()",
            b'v' => "...",
            b'x' => "i64",
            b'y' => "u64",
            b'z' => "!",
            b'p' => "_",
            _ => return None,
        })
    }

    fn type_(&mut self) -> Option<()> {
        if self.depth >= MAX_DEPTH {
            return None;
        }
        self.depth += 1;
        let r = self.type_inner();
        self.depth -= 1;
        r
    }

    fn type_inner(&mut self) -> Option<()> {
        let tag = self.peek()?;
        if let Some(name) = Self::basic_type(tag) {
            self.pos += 1;
            return self.emit(name);
        }
        match tag {
            b'R' | b'Q' => {
                self.pos += 1;
                self.emit(if tag == b'R' { "&" } else { "&mut " })?;
                if self.eat(b'L') {
                    self.base62()?;
                }
                self.type_()
            }
            b'P' | b'O' => {
                self.pos += 1;
                self.emit(if tag == b'P' { "*const " } else { "*mut " })?;
                self.type_()
            }
            b'A' => {
                self.pos += 1;
                self.emit("[")?;
                self.type_()?;
                self.emit("; ")?;
                self.constant()?;
                self.emit("]")
            }
            b'S' => {
                self.pos += 1;
                self.emit("[")?;
                self.type_()?;
                self.emit("]")
            }
            b'T' => {
                self.pos += 1;
                self.emit("(")?;
                let mut count = 0;
                while !self.eat(b'E') {
                    if count > 0 {
                        self.emit(", ")?;
                    }
                    self.type_()?;
                    count += 1;
                }
                if count == 1 {
                    self.emit(",")?;
                }
                self.emit(")")
            }
            b'F' => {
                self.pos += 1;
                self.binder()?;
                if self.eat(b'U') {
                    self.emit("unsafe ")?;
                }
                if self.eat(b'K') {
                    self.emit("extern \"")?;
                    if self.eat(b'C') {
                        self.emit("C")?;
                    } else {
                        let abi = self.ident()?.replace('_', "-");
                        self.emit(&abi)?;
                    }
                    self.emit("\" ")?;
                }
                self.emit("fn(")?;
                let mut first = true;
                while !self.eat(b'E') {
                    if !first {
                        self.emit(", ")?;
                    }
                    first = false;
                    self.type_()?;
                }
                self.emit(")")?;
                if self.eat(b'u') {
                    Some(())
                } else {
                    self.emit(" -> ")?;
                    self.type_()
                }
            }
            b'D' => {
                self.pos += 1;
                self.binder()?;
                self.emit("dyn ")?;
                let mut first = true;
                while !self.eat(b'E') {
                    if !first {
                        self.emit(" + ")?;
                    }
                    first = false;
                    self.dyn_trait()?;
                }
                if self.eat(b'L') {
                    self.base62()?;
                }
                Some(())
            }
            b'B' => {
                self.pos += 1;
                self.backref(|s| s.type_())
            }
            _ => self.path(false),
        }
    }

    /// An optional `G <base-62>` binder (`for<...>`), parsed and not printed.
    fn binder(&mut self) -> Option<()> {
        if self.eat(b'G') {
            self.base62()?;
        }
        Some(())
    }

    fn dyn_trait(&mut self) -> Option<()> {
        // A trait path, whose generic arguments stay open for associated type bindings.
        let mut open = false;
        if self.eat(b'I') {
            self.path(false)?;
            self.emit("<")?;
            self.generic_args()?;
            open = true;
        } else {
            self.path(false)?;
        }
        while self.eat(b'p') {
            if open {
                self.emit(", ")?;
            } else {
                self.emit("<")?;
                open = true;
            }
            let name = self.ident()?.to_owned();
            self.emit(&name)?;
            self.emit(" = ")?;
            self.type_()?;
        }
        if open {
            self.emit(">")?;
        }
        Some(())
    }

    fn constant(&mut self) -> Option<()> {
        if self.eat(b'B') {
            return self.backref(|s| s.constant());
        }
        if self.eat(b'p') {
            return self.emit("_");
        }
        let ty = self.next()?;
        let negative = self.eat(b'n');
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_hexdigit() && b != b'_')
        {
            self.pos += 1;
        }
        let hex = std::str::from_utf8(&self.s[start..self.pos]).ok()?;
        if !self.eat(b'_') || hex.len() > 32 {
            return None;
        }
        let value = if hex.is_empty() {
            0
        } else {
            u128::from_str_radix(hex, 16).ok()?
        };
        match ty {
            b'b' => self.emit(match value {
                0 => "false",
                1 => "true",
                _ => return None,
            }),
            b'c' => {
                let c = char::from_u32(u32::try_from(value).ok()?)?;
                self.emit(&format!("{c:?}"))
            }
            t if Self::basic_type(t).is_some_and(|n| n.starts_with(['i', 'u'])) => {
                if negative {
                    self.emit("-")?;
                }
                self.emit(&value.to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_strips_hash_and_decodes_escapes() {
        assert_eq!(
            demangle("_ZN2dm12with_closure17hbb0adf90573f5eabE").as_deref(),
            Some("dm::with_closure")
        );
        assert_eq!(
            demangle("_ZN36_$LT$dm..Sq$u20$as$u20$dm..Shape$GT$4area17h3d5191619a870e7fE")
                .as_deref(),
            Some("<dm::Sq as dm::Shape>::area")
        );
        assert_eq!(
            demangle("_ZN4core3ptr55drop_glue$LT$dm..Holder$LT$$LP$u8$C$$RF$str$RP$$GT$$GT$17h9473856f0818f1c4E")
                .as_deref(),
            Some("core::ptr::drop_glue<dm::Holder<(u8,&str)>>")
        );
        assert_eq!(
            demangle("_ZN2dm12with_closure28_$u7b$$u7b$closure$u7d$$u7d$17h1d13d59430ab5b69E")
                .as_deref(),
            Some("dm::with_closure::{{closure}}")
        );
    }

    #[test]
    fn legacy_accepts_a_linker_suffix_and_rejects_cpp_names() {
        assert_eq!(
            demangle("_ZN2dm4main17h3e04449de143555eE.llvm.12345").as_deref(),
            Some("dm::main")
        );
        assert_eq!(demangle("_ZN3foo3barEv"), None);
        assert_eq!(demangle("_ZNSt6vectorIiSaIiEE9push_backEOi"), None);
        assert_eq!(demangle("malloc"), None);
        assert_eq!(demangle(""), None);
    }

    #[test]
    fn v0_paths_generics_and_impls() {
        assert_eq!(
            demangle("_RNvNtCsgFX3jaOk65H_5alloc5alloc5alloc").as_deref(),
            Some("alloc::alloc::alloc")
        );
        assert_eq!(
            demangle("_RNvMs0_NtCsgFX3jaOk65H_5alloc5allocNtB5_6Global18alloc_impl_runtime")
                .as_deref(),
            Some("<alloc::alloc::Global>::alloc_impl_runtime")
        );
        assert_eq!(
            demangle(
                "_RNvXs1_NtCsgFX3jaOk65H_5alloc5allocNtB5_6GlobalNtNtCskADBYZwJmSU_4core5alloc9Allocator8allocate"
            )
            .as_deref(),
            Some("<alloc::alloc::Global as core::alloc::Allocator>::allocate")
        );
    }

    #[test]
    fn v0_const_generic_and_instantiating_crate() {
        assert_eq!(
            demangle("_RINvCsklMW56xJ4wR_2dm11generic_arrKj4_EB2_").as_deref(),
            Some("dm::generic_arr::<4>")
        );
    }

    #[test]
    fn v0_closures() {
        assert_eq!(
            demangle("_RNCNvCsinJrbhJGnG5_5rleak4main0B3_").as_deref(),
            Some("rleak::main::{closure}")
        );
        assert_eq!(
            demangle("_RNCNvCsinJrbhJGnG5_5rleak4mains_0B3_").as_deref(),
            Some("rleak::main::{closure#1}")
        );
    }

    #[test]
    fn v0_bad_input_does_not_panic() {
        for bad in [
            "_R",
            "_RN",
            "_RNv",
            "_RNvC",
            "_RNvCs",
            "_RNvC3ab",
            "_RB_",
            "_RBzz_",
            "_RINvC1a1bE",
            "_RNvNvNvNvB_B_B_B_1a",
            "_R\u{e9}",
            "_ZN",
            "_ZN99999999999a",
            "_ZN3abE_",
        ] {
            let _ = demangle(bad);
        }
        assert_eq!(demangle("_RB_"), None);
    }
}
