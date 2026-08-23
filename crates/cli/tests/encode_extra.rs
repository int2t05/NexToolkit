//! 编码工具端到端测试:Base32/58/85/Punycode/QP/Morse/Braille/零宽(Base64 等已有在 cli_smoke.rs)
//! 真实调起 nextool 二进制,验证 encode/decode 往返与已知向量。

use assert_cmd::Command;
use predicates::prelude::*;

fn nextool() -> Command {
    Command::cargo_bin("nextool").unwrap()
}

// ---- Base32(RFC 4648)----

#[test]
fn base32_encode_known_vector() {
    nextool()
        .args(["encode", "base32", "encode", "foobar"])
        .assert()
        .success()
        .stdout("MZXW6YTBOI======\n");
}

#[test]
fn base32_roundtrip() {
    let enc = nextool()
        .args(["encode", "base32", "encode", "Hello 世界"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "base32", "decode", s])
        .assert()
        .success()
        .stdout("Hello 世界\n");
}

// ---- Base58(Bitcoin)----

#[test]
fn base58_roundtrip() {
    let enc = nextool()
        .args(["encode", "base58", "encode", "NexToolkit"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "base58", "decode", s])
        .assert()
        .success()
        .stdout("NexToolkit\n");
}

// ---- Base85(Ascii85 Adobe)----

#[test]
fn base85_encode_known_vector() {
    // "Man " → Ascii85 Adobe "<9jqo^>"
    nextool()
        .args(["encode", "base85", "encode", "Man "])
        .assert()
        .success()
        .stdout(predicate::str::contains("9jqo"));
}

#[test]
fn base85_roundtrip() {
    let enc = nextool()
        .args(["encode", "base85", "encode", "Test 123"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "base85", "decode", s])
        .assert()
        .success()
        .stdout("Test 123\n");
}

// ---- Punycode(RFC 3492)----

#[test]
fn punycode_encode_known_vector() {
    // "bücher" → "xn--bcher-kva"
    nextool()
        .args(["encode", "punycode", "encode", "bücher"])
        .assert()
        .success()
        .stdout("xn--bcher-kva\n");
}

#[test]
fn punycode_roundtrip() {
    let enc = nextool()
        .args(["encode", "punycode", "encode", "münchen"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "punycode", "decode", s])
        .assert()
        .success()
        .stdout("münchen\n");
}

// ---- Quoted-Printable(RFC 2045)----

#[test]
fn quoted_printable_encode_ascii_unchanged() {
    // 纯 ASCII 可打印字符应原样输出
    nextool()
        .args(["encode", "quoted-printable", "encode", "Hello"])
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn quoted_printable_roundtrip_unicode() {
    let enc = nextool()
        .args(["encode", "quoted-printable", "encode", "中文测试"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "quoted-printable", "decode", s])
        .assert()
        .success()
        .stdout("中文测试\n");
}

// ---- Morse(国际摩斯码)----

#[test]
fn morse_encode_known() {
    // SOS → ... --- ...
    nextool()
        .args(["encode", "morse", "encode", "SOS"])
        .assert()
        .success()
        .stdout("... --- ...\n");
}

#[test]
fn morse_roundtrip() {
    let enc = nextool()
        .args(["encode", "morse", "encode", "HELLO"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    nextool()
        .args(["encode", "morse", "decode", s])
        .assert()
        .success()
        .stdout("HELLO\n");
}

// ---- Braille(Unicode 6 点)----

#[test]
fn braille_roundtrip() {
    let enc = nextool()
        .args(["encode", "braille", "encode", "abc"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();
    // 产物应含盲文字符(U+2800..=U+283F)
    assert!(s.chars().all(|c| ('\u{2800}'..='\u{283F}').contains(&c)));
    nextool()
        .args(["encode", "braille", "decode", s])
        .assert()
        .success()
        .stdout("abc\n");
}

// ---- 零宽字符隐写 ----

#[test]
fn zero_width_roundtrip() {
    let enc = nextool()
        .args(["encode", "zero-width", "encode", "secret"])
        .assert()
        .success();
    let s = std::str::from_utf8(&enc.get_output().stdout).unwrap();
    // 产物应只含零宽字符(U+200B/U+200C)
    assert!(s
        .chars()
        .all(|c| c == '\u{200B}' || c == '\u{200C}' || c == '\n'));
    let payload = s.trim();
    nextool()
        .args(["encode", "zero-width", "decode", payload])
        .assert()
        .success()
        .stdout("secret\n");
}
