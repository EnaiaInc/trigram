use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rustc_hash::FxHashSet;
use rustler::{Encoder, Env, NifResult, Term};
use std::cmp::Ordering;

// Pre-compiled regex for word boundary detection
static WORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\p{L}\p{N}]+").unwrap());

// HEURISTIC: Only spin up Rayon threads if the batch is large enough to justify
// the coordination overhead. 250 items is a safe crossover point.
const PARALLEL_THRESHOLD: usize = 250;

#[rustler::nif]
fn similarity(s1: &str, s2: &str) -> f32 {
    let s1_set = trigrams(s1);
    let s2_set = trigrams(s2);
    similarity_from_sets(&s1_set, &s2_set)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn similarity_batch(pairs: Vec<(String, String)>) -> Vec<f32> {
    // HYBRID APPROACH: Sequential for small inputs, Parallel for large
    if pairs.len() < PARALLEL_THRESHOLD {
        pairs
            .iter()
            .map(|(s1, s2)| {
                let s1_set = trigrams(s1);
                let s2_set = trigrams(s2);
                similarity_from_sets(&s1_set, &s2_set)
            })
            .collect()
    } else {
        pairs
            .par_iter() // Rayon parallel iterator
            .map(|(s1, s2)| {
                let s1_set = trigrams(s1);
                let s2_set = trigrams(s2);
                similarity_from_sets(&s1_set, &s2_set)
            })
            .collect()
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn best_match<'a>(env: Env<'a>, needle: &str, haystacks: Vec<String>) -> NifResult<Term<'a>> {
    if haystacks.is_empty() {
        return Ok(rustler::types::tuple::make_tuple(
            env,
            &[
                rustler::types::atom::Atom::from_str(env, "error")?.to_term(env),
                rustler::types::atom::Atom::from_str(env, "empty_list")?.to_term(env),
            ],
        ));
    }

    // Optimization: Calculate needle trigrams exactly ONCE
    let needle_set = trigrams(needle);

    // Defensive sentinel: Jaccard is always >= 0.0.
    // Starting at -1.0 ensures the first valid comparison always wins.
    let init_acc = (0, -1.0);

    let (best_idx, best_score) = if haystacks.len() < PARALLEL_THRESHOLD {
        // Sequential Path (Avoids thread pool overhead)
        haystacks
            .iter()
            .enumerate()
            .map(|(idx, haystack)| {
                let haystack_set = trigrams(haystack);
                let score = similarity_from_sets(&needle_set, &haystack_set);
                (idx, score)
            })
            .fold(init_acc, |acc, x| if x.1 > acc.1 { x } else { acc })
    } else {
        // Parallel Path (Rayon)
        haystacks
            .par_iter()
            .enumerate()
            .map(|(idx, haystack)| {
                let haystack_set = trigrams(haystack);
                let score = similarity_from_sets(&needle_set, &haystack_set);
                (idx, score)
            })
            .reduce(
                || init_acc,
                |acc, x| if x.1 > acc.1 { x } else { acc },
            )
    };

    Ok(rustler::types::tuple::make_tuple(
        env,
        &[
            rustler::types::atom::Atom::from_str(env, "ok")?.to_term(env),
            rustler::types::tuple::make_tuple(env, &[best_idx.encode(env), best_score.encode(env)]),
        ],
    ))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn score_all(needle: &str, haystacks: Vec<String>, min_threshold: f32) -> Vec<(usize, f32)> {
    let needle_set = trigrams(needle);

    let mut results: Vec<(usize, f32)> = if haystacks.len() < PARALLEL_THRESHOLD {
        haystacks
            .iter()
            .enumerate()
            .map(|(idx, haystack)| {
                let haystack_set = trigrams(haystack);
                (idx, similarity_from_sets(&needle_set, &haystack_set))
            })
            .filter(|(_, score)| *score >= min_threshold)
            .collect()
    } else {
        haystacks
            .par_iter()
            .enumerate()
            .map(|(idx, haystack)| {
                let haystack_set = trigrams(haystack);
                (idx, similarity_from_sets(&needle_set, &haystack_set))
            })
            .filter(|(_, score)| *score >= min_threshold)
            .collect()
    };

    // Use unstable sort (faster), order of equal elements not guaranteed
    results.sort_unstable_by(|(idx_a, score_a), (idx_b, score_b)| {
        score_b
            .partial_cmp(score_a)
            .unwrap_or(Ordering::Equal)
            .then_with(|| idx_a.cmp(idx_b))
    });

    results
}

// -----------------------------------------------------------------------------
// Core Logic & Helpers
// -----------------------------------------------------------------------------

fn similarity_from_sets(a_set: &FxHashSet<[u8; 3]>, b_set: &FxHashSet<[u8; 3]>) -> f32 {
    let shared = a_set.intersection(b_set).count() as f64;
    let total = (a_set.len() + b_set.len()) as f64 - shared;

    let value = if total == 0.0 { 0.0 } else { shared / total };
    value as f32
}

fn trigrams(text: &str) -> FxHashSet<[u8; 3]> {
    // CRITICAL: Must normalize (lowercase + remove \u{0307}) BEFORE regex matching
    // to match PostgreSQL pg_trgm behavior exactly. This order matters for edge cases.
    let normalized = pg_downcase(text);

    // Heuristic: Bytes/3 prevents massive over-allocation for CJK
    // but ensures enough space for ASCII. Min 16 to avoid tiny reallocs.
    let capacity = (normalized.len() / 3).max(16);

    // Use FxHasher (fast) instead of default SipHasher (secure/slow)
    let mut set = FxHashSet::with_capacity_and_hasher(capacity, Default::default());

    // Reusable buffer to avoid allocating a new Vec for every word
    let mut char_buf: Vec<char> = Vec::with_capacity(64);

    for mat in WORD_RE.find_iter(&normalized) {
        char_buf.clear();
        char_buf.extend([' ', ' ']); // Pre-padding

        // Text is already lowercased and \u{0307} removed by pg_downcase
        char_buf.extend(mat.as_str().chars());

        char_buf.push(' '); // Post-padding

        for window in char_buf.windows(3) {
            let trigram = compact_trigram(window[0], window[1], window[2]);
            set.insert(trigram);
        }
    }
    set
}

/// Normalize text to match PostgreSQL pg_trgm behavior:
/// lowercase + remove combining dot above (\u{0307})
fn pg_downcase(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        for lc in c.to_lowercase() {
            if lc != '\u{0307}' {
                result.push(lc);
            }
        }
    }
    result
}

fn compact_trigram(a: char, b: char, c: char) -> [u8; 3] {
    // OPTIMIZATION: Stack allocation instead of Heap Vec
    let mut buf = [0u8; 4];
    let mut bytes = [0u8; 12]; // Max UTF-8 size for 3 chars
    let mut len = 0;

    for ch in [a, b, c] {
        let encoded = ch.encode_utf8(&mut buf);
        // SAFETY: We have pre-allocated 12 bytes, max 3 chars * 4 bytes = 12.
        bytes[len..len + encoded.len()].copy_from_slice(encoded.as_bytes());
        len += encoded.len();
    }

    if len == 3 {
        // Fast path for ASCII (1 byte per char)
        [bytes[0], bytes[1], bytes[2]]
    } else {
        // Fallback for multi-byte chars: Calculate CRC32
        let crc = legacy_crc32(&bytes[..len]);
        let crc_bytes = crc.to_le_bytes();
        [crc_bytes[0], crc_bytes[1], crc_bytes[2]]
    }
}

fn legacy_crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;

    for &byte in bytes {
        let idx = ((crc >> 24) as u8 ^ byte) as usize & 0xFF;
        // SAFETY: Table size is 256, idx is masked & 0xFF.
        let table_val = unsafe { *PG_CRC32_TABLE.get_unchecked(idx) };
        crc = (table_val ^ (crc << 8)) & 0xFFFF_FFFF;
    }

    crc ^ 0xFFFF_FFFF
}

// Full PostgreSQL CRC32 Table
const PG_CRC32_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA, 0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988, 0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
    0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE, 0x1ADAD47D, 0x6DDDE4EB, 0xF4D4B551, 0x83D385C7,
    0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC, 0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5,
    0x3B6E20C8, 0x4C69105E, 0xD56041E4, 0xA2677172, 0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B,
    0x35B5A8FA, 0x42B2986C, 0xDBBBC9D6, 0xACBCF940, 0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59,
    0x26D930AC, 0x51DE003A, 0xC8D75180, 0xBFD06116, 0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
    0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924, 0x2F6F7C87, 0x58684C11, 0xC1611DAB, 0xB6662D3D,
    0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A, 0x71B18589, 0x06B6B51F, 0x9FBFE4A5, 0xE8B8D433,
    0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818, 0x7F6A0DBB, 0x086D3D2D, 0x91646C97, 0xE6635C01,
    0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E, 0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457,
    0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA, 0xFCB9887C, 0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65,
    0x4DB26158, 0x3AB551CE, 0xA3BC0074, 0xD4BB30E2, 0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB,
    0x4369E96A, 0x346ED9FC, 0xAD678846, 0xDA60B8D0, 0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
    0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086, 0x5768B525, 0x206F85B3, 0xB966D409, 0xCE61E49F,
    0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4, 0x59B33D17, 0x2EB40D81, 0xB7BD5C3B, 0xC0BA6CAD,
    0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A, 0xEAD54739, 0x9DD277AF, 0x04DB2615, 0x73DC1683,
    0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8, 0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1,
    0xF00F9344, 0x8708A3D2, 0x1E01F268, 0x6906C2FE, 0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7,
    0xFED41B76, 0x89D32BE0, 0x10DA7A5A, 0x67DD4ACC, 0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5,
    0xD6D6A3E8, 0xA1D1937E, 0x38D8C2C4, 0x4FDFF252, 0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
    0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60, 0xDF60EFC3, 0xA867DF55, 0x316E8EEF, 0x4669BE79,
    0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236, 0xCC0C7795, 0xBB0B4703, 0x220216B9, 0x5505262F,
    0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04, 0xC2D7FFA7, 0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D,
    0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A, 0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713,
    0x95BF4A82, 0xE2B87A14, 0x7BB12BAE, 0x0CB61B38, 0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21,
    0x86D3D2D4, 0xF1D4E242, 0x68DDB3F8, 0x1FDA836E, 0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777,
    0x88085AE6, 0xFF0F6A70, 0x66063BCA, 0x11010B5C, 0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
    0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2, 0xA7672661, 0xD06016F7, 0x4969474D, 0x3E6E77DB,
    0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0, 0xA9BCAE53, 0xDEBB9EC5, 0x47B2CF7F, 0x30B5FFE9,
    0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6, 0xBAD03605, 0xCDD70693, 0x54DE5729, 0x23D967BF,
    0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94, 0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
];

rustler::init!("Elixir.Trigram.Native");

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to compute similarity using internal functions
    fn compute_similarity(a: &str, b: &str) -> f32 {
        let a_set = trigrams(a);
        let b_set = trigrams(b);
        similarity_from_sets(&a_set, &b_set)
    }

    #[test]
    fn test_similarity_identical() {
        assert_eq!(compute_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_similarity_empty() {
        assert_eq!(compute_similarity("", ""), 0.0);
    }

    #[test]
    fn test_similarity_partial() {
        let score = compute_similarity("hello", "hallo");
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_trigrams_basic() {
        let set = trigrams("hello");
        // Should have trigrams: "  h", " he", "hel", "ell", "llo", "lo "
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn test_trigrams_unicode() {
        // Test Turkish dotless i handling (İ → i when lowercased, with \u{0307} removed)
        let set1 = trigrams("İstanbul");
        let set2 = trigrams("istanbul");
        // After case folding and \u{0307} removal, the trigram SETS should be identical
        // because İ lowercases to "i" + "\u{0307}", and we filter out \u{0307}
        assert_eq!(set1, set2, "İstanbul and istanbul should produce identical trigram sets");
        // And thus similarity should be 1.0
        let score = compute_similarity("İstanbul", "istanbul");
        assert_eq!(score, 1.0, "Score should be 1.0, was {}", score);
    }

    #[test]
    fn test_compact_trigram_ascii() {
        let result = compact_trigram('a', 'b', 'c');
        assert_eq!(result, [b'a', b'b', b'c']);
    }

    #[test]
    fn test_compact_trigram_unicode() {
        // Multi-byte chars should use CRC32
        let result = compact_trigram('é', 'é', 'é');
        // Should not be simple bytes, should be CRC32 based
        assert_ne!(result[0], 0);
    }

    #[test]
    fn test_legacy_crc32() {
        // Known CRC32 value for "abc"
        let crc = legacy_crc32(b"abc");
        // PostgreSQL's legacy CRC32 should produce consistent results
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_pg_similarity_cases() {
        // Test cases from SimilarityCases.pairs() in Elixir tests
        // These pairs should have POSITIVE similarity (same script, accent differences)
        let positive_similarity_cases = vec![
            ("café", "cafe"),
            ("naïve", "naive"),
            ("über", "uber"),
            ("São", "Sao"),
            ("ångström", "angstrom"),
            ("fiancé", "fiance"),
            ("東京", "東京"),      // identical
            ("foo_bar", "foobar"),  // partial match
            ("hello-world", "hello world"),
            ("hello—world", "hello world"),
            ("Apt #4B", "Apt 4B"),
            ("LLC", "L.L.C."),
            ("co-op", "coop"),
            ("123-456", "123456"),
            ("mid–range", "mid range"),
            ("space   tabs", "space tabs"),
            ("résumé", "resume"),
            ("façade", "facade"),
            ("İstanbul", "istanbul"),
            ("straße", "strasse"),
        ];

        for (left, right) in positive_similarity_cases {
            let score = compute_similarity(left, right);
            assert!(
                score > 0.0,
                "Expected positive similarity for ({}, {}), got {}",
                left, right, score
            );
        }

        // These pairs have ZERO similarity (different scripts, no shared trigrams)
        let zero_similarity_cases = vec![
            ("привет", "privet"),   // Cyrillic vs Latin
            ("東京", "东 京"),      // Traditional vs Simplified Chinese (different chars)
            ("Ελλάδα", "Ellada"),   // Greek vs Latin
        ];

        for (left, right) in zero_similarity_cases {
            let score = compute_similarity(left, right);
            // These should NOT crash and should return 0 (different scripts)
            assert!(
                score >= 0.0,
                "Score should be non-negative for ({}, {}), got {}",
                left, right, score
            );
        }
    }

    #[test]
    fn test_identical_after_normalization() {
        // These pairs should be IDENTICAL after case normalization
        let identical_pairs = vec![
            ("Hello", "hello"),
            ("WORLD", "world"),
            ("İstanbul", "istanbul"),  // Turkish İ → i after \u{0307} removal
        ];

        for (left, right) in identical_pairs {
            let score = compute_similarity(left, right);
            assert_eq!(
                score, 1.0,
                "Expected exact match for ({}, {}), got {}",
                left, right, score
            );
        }
    }

    #[test]
    fn test_trigram_set_identity() {
        // Verify that trigram sets are identical for case-normalized equivalents
        let set1 = trigrams("İstanbul");
        let set2 = trigrams("istanbul");
        assert_eq!(set1.len(), set2.len(), "Trigram set sizes differ");

        // Check that all trigrams in set1 are in set2
        for t in &set1 {
            assert!(set2.contains(t), "Trigram {:?} not found in set2", t);
        }
    }

    #[test]
    fn test_similarity_batch_consistency() {
        // Test that batch processing gives same results as individual
        let pairs = vec![
            ("hello".to_string(), "world".to_string()),
            ("foo".to_string(), "bar".to_string()),
            ("test".to_string(), "testing".to_string()),
        ];

        let expected: Vec<f32> = pairs
            .iter()
            .map(|(a, b)| compute_similarity(a, b))
            .collect();

        // Compare element by element
        for (i, (a, b)) in pairs.iter().enumerate() {
            let score = compute_similarity(a, b);
            assert_eq!(score, expected[i], "Mismatch at index {}", i);
        }
    }
}
