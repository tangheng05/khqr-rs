//! Phase 7: the binary itself, run end to end.

use std::process::{Command, Output};

const VECTOR: &str = "00020101021229180014jonhsmith@nbcq52045999530311654035005802KH5910Jonh Smith6010PHNOM PENH99170013173949577872263046894";

fn khqr(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_khqr"))
        .args(args)
        .output()
        .expect("the binary must run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("output must be utf-8")
}

#[test]
fn generating_then_verifying_round_trips() {
    let generated = khqr(&[
        "gen",
        "--account",
        "shop@aclb",
        "--name",
        "Shop",
        "--city",
        "Phnom Penh",
        "--amount",
        "5000",
    ]);
    assert!(generated.status.success());

    let printed = stdout(&generated);
    let mut lines = printed.lines();
    let qr = lines.next().expect("the payload is printed first");
    let md5 = lines.next().expect("the handle is printed second");

    assert!(qr.starts_with("000201010212"));
    assert_eq!(md5.split_whitespace().count(), 2);
    assert!(khqr(&["verify", qr]).status.success());
}

#[test]
fn a_static_qr_is_generated_without_an_amount() {
    let generated = khqr(&[
        "gen",
        "--account",
        "shop@aclb",
        "--name",
        "Shop",
        "--city",
        "Phnom Penh",
    ]);

    assert!(stdout(&generated).starts_with("000201010211"));
}

#[test]
fn decoding_prints_the_published_fields() {
    let output = khqr(&["decode", VECTOR]);
    let printed = stdout(&output);

    assert!(output.status.success());
    assert!(printed.contains("bakong account id         jonhsmith@nbcq"));
    assert!(printed.contains("merchant city             PHNOM PENH"));
    assert!(printed.contains("transaction amount        500"));
    assert!(printed.contains("md5                       b1c250304b8594e4c6b53dd44791b57a"));
}

#[test]
fn verifying_a_good_payload_succeeds() {
    let output = khqr(&["verify", VECTOR]);

    assert!(output.status.success());
    assert_eq!(stdout(&output).trim(), "ok");
}

#[test]
fn verifying_a_tampered_payload_fails() {
    let tampered = VECTOR.replacen("5802KH", "5802KM", 1);
    let output = khqr(&["verify", &tampered]);

    assert!(!output.status.success());
}

#[test]
fn an_invalid_account_is_reported() {
    let output = khqr(&[
        "gen",
        "--account",
        "no-bank-here",
        "--name",
        "Shop",
        "--city",
        "Phnom Penh",
    ]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("account id"));
}
