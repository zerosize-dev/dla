//! Name normalization for generated JVM classes: Spring CGLIB, Hibernate, ByteBuddy,
//! Mockito and Javassist proxies, and lambda classes map to the user class they were
//! generated from. JDK dynamic proxies (`$ProxyN`) have no user class, so they keep
//! their name and are marked as proxies.
//!
//! The raw name stays available: [`deproxy`] returns both.

use std::borrow::Cow;

/// What kind of generated class a name is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyKind {
    /// `X$$SpringCGLIB$$0`, `X$$EnhancerBySpringCGLIB$$abcd1234`,
    /// `X$$FastClassBySpringCGLIB$$abcd1234`, and plain CGLIB.
    Cglib,
    /// `X$HibernateProxy$abc`.
    Hibernate,
    /// `X$ByteBuddy$abc`.
    ByteBuddy,
    /// `X$MockitoMock$123`.
    Mockito,
    /// `X_$$_jvst1a2_3`.
    Javassist,
    /// `Outer$$Lambda$123/0x0000000800c0a000`, `Outer$$Lambda/0x...`.
    Lambda,
    /// `jdk.proxy2.$Proxy17`, `com.sun.proxy.$Proxy9`: no user class, name kept.
    JdkProxy,
}

impl ProxyKind {
    /// A short word for output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cglib => "cglib",
            Self::Hibernate => "hibernate",
            Self::ByteBuddy => "bytebuddy",
            Self::Mockito => "mockito",
            Self::Javassist => "javassist",
            Self::Lambda => "lambda",
            Self::JdkProxy => "jdk-proxy",
        }
    }
}

/// A class name before and after normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deproxied {
    /// The name as the tool printed it.
    pub raw: String,
    /// The user class name; equal to `raw` when nothing was generated or for a JDK proxy.
    pub name: String,
    /// The kind of generated class, or `None` for an ordinary class.
    pub kind: Option<ProxyKind>,
}

/// Markers that start the generated part of a name, with the kind they stand for. The
/// user class is everything before the first marker. Order does not matter: the earliest
/// position in the name wins.
const MARKERS: [(&str, ProxyKind); 12] = [
    ("$$SpringCGLIB$$", ProxyKind::Cglib),
    ("$$EnhancerBySpringCGLIB$$", ProxyKind::Cglib),
    ("$$FastClassBySpringCGLIB$$", ProxyKind::Cglib),
    ("$$EnhancerByCGLIB$$", ProxyKind::Cglib),
    ("$$FastClassByCGLIB$$", ProxyKind::Cglib),
    ("$$EnhancerByMockitoWithCGLIB$$", ProxyKind::Mockito),
    ("$HibernateProxy$", ProxyKind::Hibernate),
    ("$ByteBuddy$", ProxyKind::ByteBuddy),
    ("$MockitoMock$", ProxyKind::Mockito),
    ("_$$_jvst", ProxyKind::Javassist),
    ("_$$_javassist", ProxyKind::Javassist),
    ("$$Lambda", ProxyKind::Lambda),
];

/// Splits trailing array brackets (`[]`) off a name.
fn split_array(s: &str) -> (&str, &str) {
    let mut base = s;
    while let Some(b) = base.strip_suffix("[]") {
        base = b;
    }
    (base, &s[base.len()..])
}

/// True for a JDK dynamic proxy class name (`jdk.proxy2.$Proxy17`, `$Proxy9`).
fn is_jdk_proxy(base: &str) -> bool {
    let simple = base.rsplit('.').next().unwrap_or(base);
    simple
        .strip_prefix("$Proxy")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Normalizes `raw` and keeps the raw name.
pub fn deproxy(raw: &str) -> Deproxied {
    let (base, dims) = split_array(raw);
    if is_jdk_proxy(base) {
        return Deproxied {
            raw: raw.to_owned(),
            name: raw.to_owned(),
            kind: Some(ProxyKind::JdkProxy),
        };
    }
    let mut best: Option<(usize, ProxyKind)> = None;
    for (m, kind) in MARKERS {
        if let Some(pos) = base.find(m)
            && pos > 0
            && best.is_none_or(|(p, _)| pos < p)
        {
            best = Some((pos, kind));
        }
    }
    match best {
        Some((pos, kind)) => Deproxied {
            raw: raw.to_owned(),
            name: format!("{}{dims}", &base[..pos]),
            kind: Some(kind),
        },
        None => Deproxied {
            raw: raw.to_owned(),
            name: raw.to_owned(),
            kind: None,
        },
    }
}

/// The user class name for `raw`; borrowed when nothing changes.
pub fn normalize(raw: &str) -> Cow<'_, str> {
    let d = deproxy(raw);
    if d.name == raw {
        Cow::Borrowed(raw)
    } else {
        Cow::Owned(d.name)
    }
}

/// True when `raw` names a generated class (proxy or lambda), JDK proxies included.
pub fn is_generated(raw: &str) -> bool {
    deproxy(raw).kind.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(raw: &str) -> String {
        normalize(raw).into_owned()
    }

    #[test]
    fn spring_cglib_names() {
        assert_eq!(
            n("com.acme.OrderService$$SpringCGLIB$$0"),
            "com.acme.OrderService"
        );
        assert_eq!(
            n("com.acme.OrderService$$EnhancerBySpringCGLIB$$abcd1234"),
            "com.acme.OrderService"
        );
        assert_eq!(
            n("com.acme.OrderService$$FastClassBySpringCGLIB$$9f8e7d6c"),
            "com.acme.OrderService"
        );
        assert_eq!(n("a.B$Inner$$SpringCGLIB$$12"), "a.B$Inner");
        assert_eq!(n("a.B$$SpringCGLIB$$0[]"), "a.B[]");
    }

    #[test]
    fn other_proxies() {
        assert_eq!(n("a.User$HibernateProxy$3kzQ1x"), "a.User");
        assert_eq!(n("a.User$ByteBuddy$Zq81"), "a.User");
        assert_eq!(n("a.Repo$MockitoMock$1209334"), "a.Repo");
        assert_eq!(n("a.Bean_$$_jvst1a2_3"), "a.Bean");
    }

    #[test]
    fn lambdas() {
        assert_eq!(n("a.Outer$$Lambda$123/0x0000000800c0a000"), "a.Outer");
        assert_eq!(n("a.Outer$$Lambda/0x0000000800c0a000"), "a.Outer");
        assert_eq!(
            n("a.Outer$Inner$$Lambda$14/0x0000000800c0a000"),
            "a.Outer$Inner"
        );
        // A `$$Lambda` at the very start leaves no class: the name is kept.
        assert_eq!(n("$$Lambda$1"), "$$Lambda$1");
    }

    #[test]
    fn jdk_proxy_is_kept_and_marked() {
        let d = deproxy("jdk.proxy2.$Proxy17");
        assert_eq!(d.name, "jdk.proxy2.$Proxy17");
        assert_eq!(d.kind, Some(ProxyKind::JdkProxy));
        assert!(is_generated("com.sun.proxy.$Proxy9"));
        assert!(!is_generated("a.$ProxyFactory"));
    }

    #[test]
    fn ordinary_names_are_borrowed() {
        assert!(matches!(
            normalize("java.util.HashMap$Node"),
            Cow::Borrowed(_)
        ));
        assert_eq!(deproxy("a.B").kind, None);
        let d = deproxy("a.B$$SpringCGLIB$$0");
        assert_eq!(d.raw, "a.B$$SpringCGLIB$$0");
        assert_eq!(d.kind, Some(ProxyKind::Cglib));
    }
}
