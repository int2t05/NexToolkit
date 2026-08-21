//! CLI 集成测试:真实调起 nextool 二进制,验证子命令路由、stdin 输入、退出码与错误路径
//! 禁止 mock,真实调用真实数据

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_all_domains() {
    // 顶层 help 应列出七个域
    Command::cargo_bin("nextool")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("encode"))
        .stdout(predicate::str::contains("convert"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("text"))
        .stdout(predicate::str::contains("crypto"))
        .stdout(predicate::str::contains("net-time"));
}

#[test]
fn base64_encode_arg() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "encode", "Hello"])
        .assert()
        .success()
        .stdout("SGVsbG8=\n");
}

#[test]
fn base64_decode_stdin() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "decode"])
        .write_stdin("SGVsbG8=")
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn base64_invalid_nonzero_exit() {
    // 非法输入应非零退出码 + stderr 错误
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "decode", "!!!!"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("错误"));
}

#[test]
fn json_to_yaml_stdin() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["convert", "json-yaml", "to"])
        .write_stdin("{\"a\":1,\"b\":[2,3]}")
        .assert()
        .success()
        .stdout(predicate::str::contains("a: 1"))
        .stdout(predicate::str::contains("- 2"));
}

#[test]
fn numbase_convert() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["convert", "numbase", "16", "10", "ff"])
        .assert()
        .success()
        .stdout("255\n");
}

#[test]
fn hash_sha256() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["generate", "hash", "sha256", "abc"])
        .assert()
        .success()
        .stdout("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n");
}

#[test]
fn case_snake() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["text", "case", "snake", "Hello World"])
        .assert()
        .success()
        .stdout("hello_world\n");
}

#[test]
fn regex_match() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["text", "regex-match", r"\d+", "a12b3"])
        .assert()
        .success()
        .stdout("12\n3\n");
}

#[test]
fn aes_gcm_roundtrip() {
    // 加密后解密应还原明文
    let enc = Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-encrypt", "--password", "pw", "Secret"])
        .assert()
        .success();
    let ciphertext = std::str::from_utf8(&enc.get_output().stdout).unwrap().trim();

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-decrypt", "--password", "pw", ciphertext])
        .assert()
        .success()
        .stdout("Secret\n");
}

#[test]
fn aes_gcm_wrong_password_fails() {
    // 错误口令应解密失败(非零退出)
    let enc = Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-encrypt", "--password", "pw", "Secret"])
        .assert()
        .success();
    let ciphertext = std::str::from_utf8(&enc.get_output().stdout).unwrap().trim();

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-decrypt", "--password", "wrong", ciphertext])
        .assert()
        .failure();
}

#[test]
fn timestamp_roundtrip() {
    // ts -> human -> ts 应一致(Asia/Shanghai +08)
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["net-time", "ts-to-human", "1700000000", "--tz", "Asia/Shanghai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2023-11-15T06:13:20+08:00"));

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["net-time", "ts-from-human", "2023-11-15 06:13:20", "--tz", "Asia/Shanghai"])
        .assert()
        .success()
        .stdout("1700000000\n");
}
