//! A small JVM class-file reader: the constant pool, fields, methods, the `Code`
//! attribute's instructions that matter to the census (`new`, `invoke*`, `putstatic`,
//! `putfield`) and the `LineNumberTable`, plus `SourceFile`.
//!
//! It reads the format of JVMS chapter 4 and nothing else. Anything malformed ends in
//! an `Err` and the caller falls back to source parsing.

use std::collections::HashMap;

/// `ACC_STATIC`.
pub const ACC_STATIC: u16 = 0x0008;
/// `ACC_NATIVE`.
pub const ACC_NATIVE: u16 = 0x0100;
/// `ACC_SYNTHETIC`.
pub const ACC_SYNTHETIC: u16 = 0x1000;
/// `ACC_ENUM`.
pub const ACC_ENUM: u16 = 0x4000;
/// `ACC_PRIVATE`.
pub const ACC_PRIVATE: u16 = 0x0002;
/// `ACC_FINAL`.
pub const ACC_FINAL: u16 = 0x0010;

/// A parsed class file.
#[derive(Debug, Clone, Default)]
pub struct ClassFile {
    /// Internal name (`a/b/Foo`).
    pub name: String,
    /// Super class internal name.
    pub super_name: Option<String>,
    /// Access flags.
    pub access: u16,
    /// `SourceFile` attribute.
    pub source_file: Option<String>,
    /// Fields.
    pub fields: Vec<Field>,
    /// Methods.
    pub methods: Vec<Method>,
}

/// A field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Access flags.
    pub access: u16,
    /// Name.
    pub name: String,
    /// Descriptor.
    pub descriptor: String,
}

/// A method.
#[derive(Debug, Clone)]
pub struct Method {
    /// Access flags.
    pub access: u16,
    /// Name.
    pub name: String,
    /// Descriptor.
    pub descriptor: String,
    /// Relevant instructions, in order.
    pub insns: Vec<Insn>,
    /// `(start_pc, line)` pairs, sorted by pc.
    pub lines: Vec<(u16, u16)>,
}

impl Method {
    /// The source line of the instruction at `pc`.
    pub fn line_at(&self, pc: u32) -> Option<u32> {
        self.lines
            .iter()
            .filter(|(start, _)| u32::from(*start) <= pc)
            .max_by_key(|(start, _)| *start)
            .map(|(_, l)| u32::from(*l))
            .or_else(|| self.lines.first().map(|(_, l)| u32::from(*l)))
    }
}

/// An instruction the census cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insn {
    /// Byte offset in the code array.
    pub pc: u32,
    /// What it does.
    pub op: Op,
}

/// Instruction kinds kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// `new Class`.
    New(String),
    /// A method call: owner internal name, method name, descriptor, whether it has a
    /// receiver (`invokevirtual`, `invokeinterface`, `invokespecial`).
    Invoke(String, String, String, bool),
    /// `putstatic owner.name`.
    PutStatic(String, String),
    /// `putfield owner.name`.
    PutField(String, String),
}

/// Why a class file could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("class file: {0}")]
pub struct ClassError(pub &'static str);

struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ClassError> {
        let end = self.p.checked_add(n).ok_or(ClassError("length overflow"))?;
        let s = self.b.get(self.p..end).ok_or(ClassError("truncated"))?;
        self.p = end;
        Ok(s)
    }
    fn u1(&mut self) -> Result<u8, ClassError> {
        Ok(self.take(1)?[0])
    }
    fn u2(&mut self) -> Result<u16, ClassError> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes([s[0], s[1]]))
    }
    fn u4(&mut self) -> Result<u32, ClassError> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
}

#[derive(Clone)]
enum Cp {
    None,
    Utf8(String),
    Class(u16),
    Ref(u16, u16),
    NameType(u16, u16),
    Other,
}

struct Pool(Vec<Cp>);

impl Pool {
    fn utf8(&self, i: u16) -> Result<&str, ClassError> {
        match self.0.get(usize::from(i)) {
            Some(Cp::Utf8(s)) => Ok(s),
            _ => Err(ClassError("bad utf8 index")),
        }
    }
    fn class(&self, i: u16) -> Result<&str, ClassError> {
        match self.0.get(usize::from(i)) {
            Some(Cp::Class(n)) => self.utf8(*n),
            _ => Err(ClassError("bad class index")),
        }
    }
    fn member(&self, i: u16) -> Result<(String, String, String), ClassError> {
        match self.0.get(usize::from(i)) {
            Some(Cp::Ref(c, nt)) => {
                let owner = self.class(*c)?.to_owned();
                match self.0.get(usize::from(*nt)) {
                    Some(Cp::NameType(n, d)) => {
                        Ok((owner, self.utf8(*n)?.to_owned(), self.utf8(*d)?.to_owned()))
                    }
                    _ => Err(ClassError("bad name-and-type index")),
                }
            }
            _ => Err(ClassError("bad member index")),
        }
    }
}

/// Decodes modified UTF-8 loosely (NUL and surrogate pairs are not needed here).
fn mutf8(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Parses a class file.
pub fn parse(bytes: &[u8]) -> Result<ClassFile, ClassError> {
    let mut r = Reader { b: bytes, p: 0 };
    if r.u4()? != 0xCAFE_BABE {
        return Err(ClassError("bad magic"));
    }
    r.u2()?;
    r.u2()?;
    let count = usize::from(r.u2()?);
    let mut pool = Pool(vec![Cp::None; count.max(1)]);
    let mut i = 1;
    while i < count {
        let tag = r.u1()?;
        let entry = match tag {
            1 => {
                let n = usize::from(r.u2()?);
                Cp::Utf8(mutf8(r.take(n)?))
            }
            3 | 4 => {
                r.take(4)?;
                Cp::Other
            }
            5 | 6 => {
                r.take(8)?;
                pool.0[i] = Cp::Other;
                i += 1;
                Cp::None
            }
            7 => Cp::Class(r.u2()?),
            8 | 16 | 19 | 20 => {
                r.u2()?;
                Cp::Other
            }
            9..=11 => Cp::Ref(r.u2()?, r.u2()?),
            12 => Cp::NameType(r.u2()?, r.u2()?),
            15 => {
                r.take(3)?;
                Cp::Other
            }
            17 | 18 => {
                r.take(4)?;
                Cp::Other
            }
            _ => return Err(ClassError("unknown constant tag")),
        };
        if i < pool.0.len() {
            pool.0[i] = entry;
        }
        i += 1;
    }
    let access = r.u2()?;
    let this = r.u2()?;
    let sup = r.u2()?;
    let mut cf = ClassFile {
        name: pool.class(this)?.to_owned(),
        super_name: if sup == 0 {
            None
        } else {
            Some(pool.class(sup)?.to_owned())
        },
        access,
        ..ClassFile::default()
    };
    for _ in 0..r.u2()? {
        r.u2()?;
    }
    for _ in 0..r.u2()? {
        let access = r.u2()?;
        let name = pool.utf8(r.u2()?)?.to_owned();
        let descriptor = pool.utf8(r.u2()?)?.to_owned();
        for _ in 0..r.u2()? {
            r.u2()?;
            let len = r.u4()? as usize;
            r.take(len)?;
        }
        cf.fields.push(Field {
            access,
            name,
            descriptor,
        });
    }
    for _ in 0..r.u2()? {
        let access = r.u2()?;
        let name = pool.utf8(r.u2()?)?.to_owned();
        let descriptor = pool.utf8(r.u2()?)?.to_owned();
        let mut m = Method {
            access,
            name,
            descriptor,
            insns: Vec::new(),
            lines: Vec::new(),
        };
        for _ in 0..r.u2()? {
            let an = pool.utf8(r.u2()?)?.to_owned();
            let len = r.u4()? as usize;
            let body = r.take(len)?;
            if an == "Code" {
                parse_code(body, &pool, &mut m)?;
            }
        }
        cf.methods.push(m);
    }
    for _ in 0..r.u2()? {
        let an = pool.utf8(r.u2()?)?.to_owned();
        let len = r.u4()? as usize;
        let body = r.take(len)?;
        if an == "SourceFile" && body.len() >= 2 {
            cf.source_file = Some(
                pool.utf8(u16::from_be_bytes([body[0], body[1]]))?
                    .to_owned(),
            );
        }
    }
    Ok(cf)
}

fn parse_code(body: &[u8], pool: &Pool, m: &mut Method) -> Result<(), ClassError> {
    let mut r = Reader { b: body, p: 0 };
    r.u2()?;
    r.u2()?;
    let len = r.u4()? as usize;
    let code = r.take(len)?;
    for _ in 0..r.u2()? {
        r.take(8)?;
    }
    for _ in 0..r.u2()? {
        let an = pool.utf8(r.u2()?)?;
        let alen = r.u4()? as usize;
        let ab = r.take(alen)?;
        if an == "LineNumberTable" {
            let mut lr = Reader { b: ab, p: 0 };
            for _ in 0..lr.u2()? {
                let pc = lr.u2()?;
                let line = lr.u2()?;
                m.lines.push((pc, line));
            }
        }
    }
    m.lines.sort_unstable();
    decode(code, pool, m)
}

fn be16(code: &[u8], at: usize) -> Result<u16, ClassError> {
    let s = code.get(at..at + 2).ok_or(ClassError("truncated code"))?;
    Ok(u16::from_be_bytes([s[0], s[1]]))
}

fn be32(code: &[u8], at: usize) -> Result<i32, ClassError> {
    let s = code.get(at..at + 4).ok_or(ClassError("truncated code"))?;
    Ok(i32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn decode(code: &[u8], pool: &Pool, m: &mut Method) -> Result<(), ClassError> {
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let at = u32::try_from(pc).unwrap_or(u32::MAX);
        let len: usize = match op {
            0x00..=0x0f => 1,
            0x10 | 0x12 => 2,
            0x11 | 0x13 | 0x14 => 3,
            0x15..=0x19 => 2,
            0x1a..=0x35 => 1,
            0x36..=0x3a => 2,
            0x3b..=0x83 => 1,
            0x84 => 3,
            0x85..=0x98 => 1,
            0x99..=0xa8 => 3,
            0xa9 => 2,
            0xaa => {
                let base = (pc + 4) & !3;
                let low = be32(code, base + 4)?;
                let high = be32(code, base + 8)?;
                let n = usize::try_from(i64::from(high) - i64::from(low) + 1)
                    .map_err(|_| ClassError("bad tableswitch"))?;
                base + 12 + n * 4 - pc
            }
            0xab => {
                let base = (pc + 4) & !3;
                let n = usize::try_from(be32(code, base + 4)?)
                    .map_err(|_| ClassError("bad lookupswitch"))?;
                base + 8 + n * 8 - pc
            }
            0xac..=0xb1 => 1,
            0xb2..=0xb8 => {
                let idx = be16(code, pc + 1)?;
                let (owner, name, desc) = pool.member(idx)?;
                m.insns.push(Insn {
                    pc: at,
                    op: match op {
                        0xb3 => Op::PutStatic(owner, name),
                        0xb5 => Op::PutField(owner, name),
                        0xb6 | 0xb7 => Op::Invoke(owner, name, desc, true),
                        0xb8 => Op::Invoke(owner, name, desc, false),
                        _ => {
                            pc += 3;
                            continue;
                        }
                    },
                });
                3
            }
            0xb9 => {
                let idx = be16(code, pc + 1)?;
                let (owner, name, desc) = pool.member(idx)?;
                m.insns.push(Insn {
                    pc: at,
                    op: Op::Invoke(owner, name, desc, true),
                });
                5
            }
            0xba => 5,
            0xbb => {
                let idx = be16(code, pc + 1)?;
                m.insns.push(Insn {
                    pc: at,
                    op: Op::New(pool.class(idx)?.to_owned()),
                });
                3
            }
            0xbc => 2,
            0xbd => 3,
            0xbe | 0xbf => 1,
            0xc0 | 0xc1 => 3,
            0xc2 | 0xc3 => 1,
            0xc4 => {
                if code.get(pc + 1) == Some(&0x84) {
                    6
                } else {
                    4
                }
            }
            0xc5 => 4,
            0xc6 | 0xc7 => 3,
            0xc8 | 0xc9 => 5,
            _ => return Err(ClassError("unknown opcode")),
        };
        pc += len;
    }
    Ok(())
}

/// Decodes a field descriptor into a readable type (`Ljava/util/Map;` gives
/// `java.util.Map`, `[I` gives `int[]`).
pub fn descriptor_type(d: &str) -> String {
    let dims = d.chars().take_while(|c| *c == '[').count();
    let base = &d[dims..];
    let b = match base.chars().next() {
        Some('B') => "byte".to_owned(),
        Some('C') => "char".to_owned(),
        Some('D') => "double".to_owned(),
        Some('F') => "float".to_owned(),
        Some('I') => "int".to_owned(),
        Some('J') => "long".to_owned(),
        Some('S') => "short".to_owned(),
        Some('Z') => "boolean".to_owned(),
        Some('L') => base
            .trim_start_matches('L')
            .trim_end_matches(';')
            .replace('/', "."),
        _ => base.to_owned(),
    };
    format!("{b}{}", "[]".repeat(dims))
}

/// Simple class name of an internal name (`a/b/Foo$Bar` gives `Foo$Bar`).
pub fn simple_name(internal: &str) -> &str {
    internal.rsplit('/').next().unwrap_or(internal)
}

/// Indexes class files by internal name.
pub fn index(classes: &[ClassFile]) -> HashMap<&str, &ClassFile> {
    classes.iter().map(|c| (c.name.as_str(), c)).collect()
}

#[cfg(test)]
pub(crate) mod testutil {
    //! Builds tiny class files by hand for tests.

    /// Builds a class `name` with the given static field descriptors and a `<clinit>`
    /// that does `new java/lang/Thread; putstatic field0` at line 7.
    pub fn sample(name: &str, field: &str, desc: &str) -> Vec<u8> {
        let mut cp: Vec<Vec<u8>> = Vec::new();
        let utf8 = |s: &str| {
            let mut v = vec![1u8];
            v.extend((s.len() as u16).to_be_bytes());
            v.extend(s.as_bytes());
            v
        };
        let class = |i: u16| {
            let mut v = vec![7u8];
            v.extend(i.to_be_bytes());
            v
        };
        let fieldref = |c: u16, nt: u16| {
            let mut v = vec![9u8];
            v.extend(c.to_be_bytes());
            v.extend(nt.to_be_bytes());
            v
        };
        let nat = |n: u16, d: u16| {
            let mut v = vec![12u8];
            v.extend(n.to_be_bytes());
            v.extend(d.to_be_bytes());
            v
        };
        cp.push(utf8(name)); // 1
        cp.push(class(1)); // 2
        cp.push(utf8("java/lang/Object")); // 3
        cp.push(class(3)); // 4
        cp.push(utf8(field)); // 5
        cp.push(utf8(desc)); // 6
        cp.push(nat(5, 6)); // 7
        cp.push(fieldref(2, 7)); // 8
        cp.push(utf8("java/lang/Thread")); // 9
        cp.push(class(9)); // 10
        cp.push(utf8("<clinit>")); // 11
        cp.push(utf8("()V")); // 12
        cp.push(utf8("Code")); // 13
        cp.push(utf8("LineNumberTable")); // 14
        cp.push(utf8("SourceFile")); // 15
        cp.push(utf8("Foo.java")); // 16
        let mut b = Vec::new();
        b.extend(0xCAFE_BABEu32.to_be_bytes());
        b.extend([0, 0, 0, 52]);
        b.extend((cp.len() as u16 + 1).to_be_bytes());
        for e in &cp {
            b.extend(e);
        }
        b.extend(0x0021u16.to_be_bytes());
        b.extend(2u16.to_be_bytes());
        b.extend(4u16.to_be_bytes());
        b.extend(0u16.to_be_bytes());
        b.extend(1u16.to_be_bytes());
        b.extend(0x000Au16.to_be_bytes());
        b.extend(5u16.to_be_bytes());
        b.extend(6u16.to_be_bytes());
        b.extend(0u16.to_be_bytes());
        b.extend(1u16.to_be_bytes());
        b.extend(0x0008u16.to_be_bytes());
        b.extend(11u16.to_be_bytes());
        b.extend(12u16.to_be_bytes());
        b.extend(1u16.to_be_bytes());
        // Code: new #10; putstatic #8; return
        let code = [0xbbu8, 0, 10, 0xb3, 0, 8, 0xb1];
        let mut line = Vec::new();
        line.extend(1u16.to_be_bytes());
        line.extend(0u16.to_be_bytes());
        line.extend(7u16.to_be_bytes());
        let mut body = Vec::new();
        body.extend(1u16.to_be_bytes());
        body.extend(0u16.to_be_bytes());
        body.extend((code.len() as u32).to_be_bytes());
        body.extend(code);
        body.extend(0u16.to_be_bytes());
        body.extend(1u16.to_be_bytes());
        body.extend(14u16.to_be_bytes());
        body.extend((line.len() as u32).to_be_bytes());
        body.extend(&line);
        b.extend(13u16.to_be_bytes());
        b.extend((body.len() as u32).to_be_bytes());
        b.extend(&body);
        b.extend(1u16.to_be_bytes());
        b.extend(15u16.to_be_bytes());
        b.extend(2u32.to_be_bytes());
        b.extend(16u16.to_be_bytes());
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fields_code_lines_and_source_file() {
        let b = testutil::sample("a/b/Foo", "CACHE", "Ljava/util/Map;");
        let c = parse(&b).unwrap();
        assert_eq!(c.name, "a/b/Foo");
        assert_eq!(c.source_file.as_deref(), Some("Foo.java"));
        assert_eq!(c.fields.len(), 1);
        assert_eq!(c.fields[0].access & ACC_STATIC, ACC_STATIC);
        assert_eq!(descriptor_type(&c.fields[0].descriptor), "java.util.Map");
        let m = &c.methods[0];
        assert_eq!(m.name, "<clinit>");
        assert_eq!(m.insns.len(), 2);
        assert_eq!(m.insns[0].op, Op::New("java/lang/Thread".into()));
        assert_eq!(
            m.insns[1].op,
            Op::PutStatic("a/b/Foo".into(), "CACHE".into())
        );
        assert_eq!(m.line_at(3), Some(7));
    }

    #[test]
    fn rejects_garbage_and_truncation() {
        assert!(parse(b"nope").is_err());
        let b = testutil::sample("a/Foo", "x", "I");
        assert!(parse(&b[..b.len() - 9]).is_err());
    }

    #[test]
    fn descriptors() {
        assert_eq!(descriptor_type("[[I"), "int[][]");
        assert_eq!(descriptor_type("[Ljava/lang/String;"), "java.lang.String[]");
        assert_eq!(simple_name("a/b/Foo$Bar"), "Foo$Bar");
    }
}
