//! The gzip reader against files made by the system `gzip`.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use dla_parsers::inflate::{GzipError, gunzip};

/// Deterministic test data: random runs, repeated phrases, zero runs and text.
fn sample(len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len + 4096);
    let mut x: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let phrases: [&[u8]; 4] = [
        b"the quick brown fox jumps over the lazy dog; ",
        b"heap dump record 0x21 instance dump; ",
        &[0u8; 300],
        b"abcabcabcabcabcabcabcabcabcabc",
    ];
    while out.len() < len {
        match next() % 5 {
            0 => {
                let n = (next() % 700) as usize + 1;
                out.extend((0..n).map(|_| next() as u8));
            }
            1 | 2 => {
                let p = phrases[(next() % 4) as usize];
                let k = (next() % 12) as usize + 1;
                for _ in 0..k {
                    out.extend_from_slice(p);
                }
            }
            3 => {
                // A copy from far back, to cross the 32 KiB window edge.
                if out.len() > 40_000 {
                    let from = out.len() - 32_000 + (next() % 500) as usize;
                    let n = (next() % 250) as usize + 3;
                    let chunk: Vec<u8> = out[from..from + n].to_vec();
                    out.extend_from_slice(&chunk);
                }
            }
            _ => out.extend_from_slice(format!("{}:{}\n", next(), next() % 97).as_bytes()),
        }
    }
    out.truncate(len);
    out
}

fn gzip(data: &[u8], level: &str) -> Option<Vec<u8>> {
    let mut child = Command::new("gzip")
        .args([level, "-c"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    let owned = data.to_vec();
    let writer = std::thread::spawn(move || {
        stdin.write_all(&owned).unwrap();
    });
    let out = child.wait_with_output().ok()?;
    writer.join().unwrap();
    out.status.success().then_some(out.stdout)
}

fn un(bytes: &[u8]) -> Result<Vec<u8>, GzipError> {
    let mut out = Vec::new();
    gunzip(bytes, &mut out)?;
    Ok(out)
}

#[test]
fn matches_system_gzip_at_levels_1_6_9() {
    let data = sample(3_000_000);
    for level in ["-1", "-6", "-9"] {
        let Some(gz) = gzip(&data, level) else {
            eprintln!("skipped: gzip not found");
            return;
        };
        assert!(gz.len() < data.len());
        assert!(un(&gz).unwrap() == data, "level {level}");
    }
}

#[test]
fn small_and_edge_inputs() {
    for data in [
        &b""[..],
        b"a",
        b"abc",
        &[0u8; 100_000][..],
        &sample(70_000)[..],
    ] {
        for level in ["-1", "-9"] {
            let Some(gz) = gzip(data, level) else {
                eprintln!("skipped: gzip not found");
                return;
            };
            assert_eq!(un(&gz).unwrap(), data, "len {} level {level}", data.len());
        }
    }
}

#[test]
fn concatenated_members_and_truncation() {
    let a = sample(50_000);
    let b = sample(10_000);
    let (Some(ga), Some(gb)) = (gzip(&a, "-6"), gzip(&b, "-9")) else {
        eprintln!("skipped: gzip not found");
        return;
    };
    let both = [ga.clone(), gb].concat();
    assert_eq!(un(&both).unwrap(), [a, b].concat());
    // Every proper prefix of a member is an error, never a short success.
    for cut in [
        0,
        1,
        9,
        10,
        11,
        ga.len() / 2,
        ga.len() - 9,
        ga.len() - 8,
        ga.len() - 1,
    ] {
        assert!(un(&ga[..cut]).is_err(), "cut at {cut}");
    }
    let mut bad = ga.clone();
    let mid = bad.len() / 2;
    bad[mid] ^= 0x55;
    assert!(un(&bad).is_err());
}
