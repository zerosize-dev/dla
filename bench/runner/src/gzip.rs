//! A small gzip reader that decodes only the start of a file, enough to check
//! the header of a compressed heap dump without a compression dependency.
//! Written after RFC 1951 (deflate) and RFC 1952 (gzip); the checksum is not verified.

/// The two bytes every gzip file starts with.
pub const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
    nbits: u32,
}

impl Bits<'_> {
    fn need(&mut self, n: u32) -> Result<(), String> {
        while self.nbits < n {
            let Some(b) = self.data.get(self.pos) else {
                return Err("the deflate stream ends early".to_owned());
            };
            self.pos += 1;
            self.bit |= u32::from(*b) << self.nbits;
            self.nbits += 8;
        }
        Ok(())
    }

    fn bits(&mut self, n: u32) -> Result<u32, String> {
        if n == 0 {
            return Ok(0);
        }
        self.need(n)?;
        let v = self.bit & ((1u32 << n) - 1);
        self.bit >>= n;
        self.nbits -= n;
        Ok(v)
    }
}

/// A canonical Huffman code as the count of codes per length and the symbols in code order.
struct Huffman {
    count: [u16; 16],
    symbol: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Huffman, String> {
        let mut count = [0u16; 16];
        for &l in lengths {
            count[usize::from(l)] += 1;
        }
        let mut offs = [0u16; 16];
        for i in 1..15 {
            offs[i + 1] = offs[i] + count[i];
        }
        let mut symbol = vec![0u16; lengths.len()];
        for (s, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbol[usize::from(offs[usize::from(l)])] = s as u16;
                offs[usize::from(l)] += 1;
            }
        }
        count[0] = 0;
        Ok(Huffman { count, symbol })
    }

    fn decode(&self, b: &mut Bits) -> Result<u16, String> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= b.bits(1)? as i32;
            let count = i32::from(self.count[len]);
            if code - count < first {
                return self
                    .symbol
                    .get((index + (code - first)) as usize)
                    .copied()
                    .ok_or_else(|| "invalid Huffman code".to_owned());
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("invalid Huffman code".to_owned())
    }
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn codes(
    b: &mut Bits,
    out: &mut Vec<u8>,
    want: usize,
    lit: &Huffman,
    dist: &Huffman,
) -> Result<(), String> {
    loop {
        if out.len() >= want {
            return Ok(());
        }
        let sym = usize::from(lit.decode(b)?);
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let i = sym - 257;
                let len = usize::from(LEN_BASE[i]) + b.bits(u32::from(LEN_EXTRA[i]))? as usize;
                let d = usize::from(dist.decode(b)?);
                if d >= 30 {
                    return Err("invalid distance code".to_owned());
                }
                let distance =
                    usize::from(DIST_BASE[d]) + b.bits(u32::from(DIST_EXTRA[d]))? as usize;
                if distance > out.len() {
                    return Err("distance reaches before the start of the output".to_owned());
                }
                for _ in 0..len {
                    let c = out[out.len() - distance];
                    out.push(c);
                }
            }
            _ => return Err("invalid literal/length code".to_owned()),
        }
    }
}

fn inflate_prefix(data: &[u8], want: usize) -> Result<Vec<u8>, String> {
    let mut b = Bits {
        data,
        pos: 0,
        bit: 0,
        nbits: 0,
    };
    let mut out = Vec::new();
    loop {
        let last = b.bits(1)? == 1;
        match b.bits(2)? {
            0 => {
                b.bit = 0;
                b.nbits = 0;
                let Some(h) = b.data.get(b.pos..b.pos + 4) else {
                    return Err("the stored block header ends early".to_owned());
                };
                let len = usize::from(u16::from_le_bytes([h[0], h[1]]));
                let nlen = u16::from_le_bytes([h[2], h[3]]);
                if nlen != !(len as u16) {
                    return Err("the stored block length check fails".to_owned());
                }
                b.pos += 4;
                let Some(body) = b.data.get(b.pos..b.pos + len) else {
                    return Err("the stored block ends early".to_owned());
                };
                out.extend_from_slice(body);
                b.pos += len;
            }
            1 => {
                let mut l = [8u8; 288];
                l[144..256].fill(9);
                l[256..280].fill(7);
                let lit = Huffman::new(&l)?;
                let dist = Huffman::new(&[5u8; 30])?;
                codes(&mut b, &mut out, want, &lit, &dist)?;
            }
            2 => {
                let nlen = b.bits(5)? as usize + 257;
                let ndist = b.bits(5)? as usize + 1;
                let ncode = b.bits(4)? as usize + 4;
                const ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let mut lengths = [0u8; 320];
                for &o in ORDER.iter().take(ncode) {
                    lengths[o] = b.bits(3)? as u8;
                }
                let code_code = Huffman::new(&lengths[..19])?;
                let mut all = vec![0u8; nlen + ndist];
                let mut i = 0;
                while i < nlen + ndist {
                    let sym = code_code.decode(&mut b)?;
                    if sym < 16 {
                        all[i] = sym as u8;
                        i += 1;
                    } else {
                        let (prev, rep) = match sym {
                            16 => {
                                if i == 0 {
                                    return Err("a repeat code with nothing to repeat".to_owned());
                                }
                                (all[i - 1], 3 + b.bits(2)? as usize)
                            }
                            17 => (0, 3 + b.bits(3)? as usize),
                            _ => (0, 11 + b.bits(7)? as usize),
                        };
                        if i + rep > nlen + ndist {
                            return Err("too many code lengths".to_owned());
                        }
                        all[i..i + rep].fill(prev);
                        i += rep;
                    }
                }
                let lit = Huffman::new(&all[..nlen])?;
                let dist = Huffman::new(&all[nlen..])?;
                codes(&mut b, &mut out, want, &lit, &dist)?;
            }
            _ => return Err("invalid deflate block type".to_owned()),
        }
        if last || out.len() >= want {
            out.truncate(want);
            return Ok(out);
        }
    }
}

/// The first `want` bytes (fewer when the stream is shorter) of the inflated
/// content of a gzip file. `Err` names the problem: no gzip magic, an unsupported
/// method, or a damaged stream.
pub fn gunzip_prefix(data: &[u8], want: usize) -> Result<Vec<u8>, String> {
    if data.len() < 10 || data[..2] != GZIP_MAGIC {
        return Err("does not start with the gzip magic 1f 8b".to_owned());
    }
    if data[2] != 8 {
        return Err(format!("gzip method {} is not deflate", data[2]));
    }
    let flags = data[3];
    let mut pos = 10usize;
    if flags & 4 != 0 {
        let Some(x) = data.get(pos..pos + 2) else {
            return Err("the gzip header ends early".to_owned());
        };
        pos += 2 + usize::from(u16::from_le_bytes([x[0], x[1]]));
    }
    for bit in [8u8, 16] {
        if flags & bit != 0 {
            let Some(n) = data.get(pos..).and_then(|r| r.iter().position(|&c| c == 0)) else {
                return Err("the gzip header ends early".to_owned());
            };
            pos += n + 1;
        }
    }
    if flags & 2 != 0 {
        pos += 2;
    }
    let body = data
        .get(pos..)
        .ok_or_else(|| "the gzip header ends early".to_owned())?;
    inflate_prefix(body, want)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(h: &str) -> Vec<u8> {
        (0..h.len() / 2)
            .map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    // Produced by Python's gzip module (level 9, mtime 0).
    const FIXED: &str =
        "1f8b08000000000002fff3720c73540808f277f3f4715530d433d03364a8a804009bc999d915000000";
    const DYNAMIC: &str = "1f8b08000000000002ff8d9c5d6e1c470c84afe2ab89c8c20e602706b24f3cbd11afa561577dc5d14336927676a69bcd9faa22d75fdf7efc78fb528fe7db97c7cffffefefeef3faf5f9edff4f5fffffe7a7c7fbecd0bdebefffcf67aebfaedcfe57fee363ed2d735fe7addfebaeaf5fb5cc4eb2f7dfd382f797fe475c3df8ffdfa7b8bc7eefadcd0fb7befffbf363c7774adeeba6b7fdc6d18653ef5fded69aec374ef8ffef8c8ebf5f5b06986ebf12d77f57dcf7be873fbdccbf8a39c91adfbf5a1b9caf2c7d78705f2664fab979813fe7e1ee1f9db7968ad6e5b6cebe001e5aeabfb572bbf7e7ffdac91321de7fa5c9fde756eebb878aeea589a98e2a9779991359732df9578aad32fe6a7ea7dc1a7ad879bb81fccddb49e70b0fb797e23730cd79a0e71b8f99f3b1d2ed467b8c0e94eb368923ab63e9fd8c73b258bc59594872b78e63cb6f3fe9c09c6bae74666a05e67f75c63a6c41d2043cf7d5d9edec73a4b0e319abcdd69cf839c7bf5d3d3742b59ed3468870ca3372fad2bd3eabc84525b8195d8c78237a74448a69ad13c0c20f9512e84c0bb0ef3740cadf1ede5542f99a668dea13ae1c39d793aca4c27961dc6093a4c30379b875ab2214602c769ce1c5f3139cc6dcd0742c4cc63a978e285917ea21171f0e971964c1fe4cc7bcd9b27415e447ef719b3ce754a5ac17204a9b352212d3bfeb3188c8f4fd7e14f681d7c72a13ed0a805ad41d8133d13943cb7add0c31f515a97678cc162e82008b19dc54857394354ab0efa9705afe7d4e6db636012323ff7d76ecc88cae8b2d21b5316f454175c6adc13fcba208319b1f0f2ecb9069e5e986835e55e0e3c83f85c2a435da87e85586ebe0a05f154344daa9c51112dc403b1a4f97adc021dde6d7b9ebfc67ddfb9be1535c0d31bec805d428d176b973885e2dd79876397cf2d29cc132fa6a3808cbc56c22629537c98c7428296682c44b8c089012a2c9629c5f1176328443d28e78af3d7761c805b1b4367dcd4492a156d912c122449d9aa0c7c928a4081a060c6340aa011c4a9a140af2a068816c8dbd8b2544a09036a08349131d64df2431bdc666cdfce7ef782108d08d2191519e839bcc2f98aa170b266658c0cbc4c43f0462f7177383f4571ba6b7c89789df730b692e01a057d65bd922b6582f48e1dd9cf68ed26b299c0e321616430903666c580c3e6368cb29b9312398f00cc625fc9a2560b92dd4450a81b203a3e64554492a1731a7de2565ccf106531d6c20d10a0d7f84fd03b200fede5d653a92d62380b68488a58421780c0a0ac931a1c8e0ae442fba4e12fd07782aa51199d3402cd4027e363d8678db9546e8468025411b19959f42356fe90839d010060b2e5a73b54ccb5b1098512b07143053f4aaface4e8f60bd4e7620443794d1238ad59b9dd2d8be3c0518d65b93442e9241e104e44b6ca4d975aa8b2eb65488653018d7aa11b3837942295ea9dac362e1a9912c808044cd105a938693f7147c7c59cdd5ba062583d3e45f70e221fd40cc60273e76b96c4b1ef15d229743af7283a9bd0286622f0d0ee6c65ec6f0a83350844a93b6bcaca701d6939250c18845384453ab617c24441271c66d690e2a9e5c4a14e05b4e77d5d2565ad172437a448b7877bf78f18b5212362e67e7666268c8576e5c40bbbf7bb561eb98031830b2e1720d125cc4e3d56c7f81df094badbe6e276ea80f634779a4aca70c11078609d11543bb4262ca9b88f560da5cb37b159a576bda438769634a92798c61ff66e14d797d407eb5db70be409ea8055f7d0cf0761d248873faa1837563aafa4b16dd223683f4b9816f523092b8aba057aa79dc7362894da33fef7dac456c20d265b1b1e54ee1774eb7950fd082f28d6525b8bb40dea4012e4b31a04dd3a9fc2319dd6734f2fbd0d775f0f00e3a74c63ad8c07bdca2ceec7d128abeaf089374248da77046885020c7d37a0c80e2f153ca85d2cfa8df5d0cc130e117a97db5cc9b484605682ff631d9a8129bb1702080f7fbdd77c2fb16e4fce348e4461d2f72aedd183c583cd925d5b17c4b76336d18a50d849bf358df68e17fee3c4bc168df9dc1469e359447e3c830bd98f0eb36c6e0bd7a0140b905238549f474dbd6b2f573a85d464b346c98d198177a71bf4460d967568a2dd0f65dc82e5ce653c94bcdcf0165082fe4c953108a0fab51ef5dadff0a0082a65989485c6fa338b287e7ccc773417579a3cc2fce30de7e7769a6e0e9545c3a47610ee4cd7a4c67e884dc607d866ec9d9ef811d47d833076b93590712615972d1ad99d7a82a92913c58814f29845761cf3972c35d63dc25ab4682fec56f3faf303b11841a4cef79d0ee70d970a731609fc5292dcd0412519995bdbd8af3f3573b55e8a6e04c0d4a25d1a6ea150d2b829945917add72912260d61bec85c99863269a28c065bfd188c1e44487fd76c80f1ff6db44b4f5cdcdcd16f3d6e7bd8719e2a40f44696d85ce5b8bdb40c5038e601b6bf88e44de089db19951bec0a538b52bb675a85e2c5248b9fb99a2acf45863199e478d6a84ba92f51074fca266119170d332eeb2c17ac9d998e6b901db1078ea2d9c04d9059724385d09b092edb37bca89157995ee3d7be0cd2f1ec4e6a2313b04f9218a7729397972956a09d3ef1c4936bcb786b4ac0d454c935c3e0aef90ac1321eb2b107842f7986ef74f128876b79147a5e47ec3bbcb151a6e513f2a2a9da545cbdf912a72479e2a95704ea5c833538cfced671433bd623e77a3afd6d80102899e7e5805ef2800df4377a199b08f22a213e52661abb3b9ddbfe21c5f0dc92d36dfe06d13ed54fba4d11a329e4198fad87635668026dfefd2d45f64dbd81f4854046d9679c9aa4a2218943129924c6a981340e9db4e54d61debf1fb6d71d1f2d120ecb1fdd9ba2744d16f581225864e29834cc9778ce4bffc000f6d369c29ceb5280e9e1ab981b15a90d5a3ed2579ff39043df8f8a50d5324e93bfa50c53123aaffc0bc9289e4869430000";

    #[test]
    fn decodes_a_fixed_huffman_block() {
        let out = gunzip_prefix(&unhex(FIXED), 100).unwrap();
        assert_eq!(out, b"JAVA PROFILE 1.0.1\0xy");
    }

    #[test]
    fn decodes_a_dynamic_huffman_block_and_stops_at_the_prefix() {
        let all = gunzip_prefix(&unhex(DYNAMIC), usize::MAX).unwrap();
        assert!(
            all.starts_with(b"alpha ") || all.len() > 1000,
            "{}",
            all.len()
        );
        let head = gunzip_prefix(&unhex(DYNAMIC), 17).unwrap();
        assert_eq!(head, all[..17]);
        assert_eq!(17257, all.len());
    }

    #[test]
    fn decodes_a_stored_block_with_a_name_field() {
        let mut g = vec![0x1f, 0x8b, 8, 8, 0, 0, 0, 0, 0, 3];
        g.extend_from_slice(b"dump\0");
        g.extend_from_slice(&[1, 5, 0, 0xfa, 0xff]);
        g.extend_from_slice(b"hello");
        g.extend_from_slice(&[0; 8]);
        assert_eq!(gunzip_prefix(&g, 100).unwrap(), b"hello");
    }

    #[test]
    fn refuses_what_is_not_gzip_or_is_damaged() {
        assert!(
            gunzip_prefix(b"JAVA PROFILE 1.0.2", 4)
                .unwrap_err()
                .contains("magic")
        );
        assert!(gunzip_prefix(&[], 4).is_err());
        let mut g = unhex(FIXED);
        g[2] = 7;
        assert!(gunzip_prefix(&g, 4).unwrap_err().contains("not deflate"));
        let g = unhex(FIXED);
        assert!(gunzip_prefix(&g[..14], 100).is_err());
        let bad = [0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3, 7, 0, 0];
        assert!(gunzip_prefix(&bad, 4).is_err());
    }
}
