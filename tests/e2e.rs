#![allow(clippy::unwrap_used)]

use std::process::Command;

fn run_command(input_file: &str, args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--", input_file])
        .args(args)
        .output()
        .expect("Failed to execute cargo run")
}

fn assert_failure(output: &std::process::Output) {
    assert!(!output.status.success(), "{output:?}");
}

fn assert_success(output: &std::process::Output) {
    assert!(output.status.success(), "{output:?}");
}

fn verify_output(output: &std::process::Output, num_lines: usize, expected_fields: &[&str]) {
    let output_str = String::from_utf8(output.stdout.clone()).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), num_lines);
    let data_line = lines[1];
    let fields: Vec<&str> = data_line.split(',').collect();
    assert_eq!(fields.len(), expected_fields.len());
    for (field, expected) in fields.iter().zip(expected_fields.iter()) {
        assert_eq!(field, expected, "{field:?} == {expected:?}");
    }
}

#[test]
fn test_expected_piped_run() {
    let output = run_command("sample.csv", &[">", "accounts.csv"]);
    assert_success(&output);
}

#[test]
fn test_wrong_client_format() {
    let output = run_command("./tests/resourses/test_wrong_client_format.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_wrong_transaction_format() {
    let output = run_command("./tests/resourses/test_wrong_transaction_format.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_wrong_amount_format() {
    let output = run_command("./tests/resourses/test_wrong_amount_format.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_negative_amount() {
    let output = run_command("./tests/resourses/test_negative_amount.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_withdrawal_without_deposit() {
    let output = run_command("./tests/resourses/test_withdrawal_without_deposit.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_null_deposit() {
    let output = run_command("./tests/resourses/test_null_deposit.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_zero_deposit() {
    let output = run_command("./tests/resourses/test_zero_deposit.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_basic_deposit_withdrawal() {
    let output = run_command("./tests/resourses/test_basic.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "5", "0", "5", "false"]);
}

#[test]
fn test_shuffled_headers_deposit_withdrawal() {
    let output = run_command("./tests/resourses/test_shuffled_headers.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "5", "0", "5", "false"]);
}

#[test]
fn test_negative_withdrawal() {
    let output = run_command("./tests/resourses/test_negative_withdrawal.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_deposit_with_multiple_withdrawal() {
    let output = run_command("./tests/resourses/test_multiple_withdrawal.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "0", "0", "0", "false"]);
}

#[test]
fn test_deposit_with_same_transaction_withdrawal() {
    let output = run_command("./tests/resourses/test_same_transaction_withdrawal.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_insufficient_funds_deposit_withdrawal() {
    let output = run_command("./tests/resourses/test_insufficient_funds.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_dispute_resolve_workflow() {
    let output = run_command("./tests/resourses/test_dispute_resolve.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "10", "0", "10", "false"]);
}

#[test]
fn test_dispute_with_already_withdrawn_funds() {
    let output = run_command("./tests/resourses/test_dispute_with_already_withdrawn_funds.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "-8.5", "10", "1.5", "false"]);
}

#[test]
fn chargeback_on_a_non_existent_disputed_transaction() {
    let output = run_command(
        "./tests/resourses/test_chargeback_on_a_non_existent_disputed_transaction.csv",
        &[],
    );
    assert_failure(&output);
}

#[test]
fn chargeback_on_a_existent_but_not_disputed_transaction() {
    let output = run_command(
        "./tests/resourses/test_chargeback_on_a_existent_but_not_disputed_transaction.csv",
        &[],
    );
    assert_failure(&output);
}

#[test]
fn test_chargeback_workflow() {
    let output = run_command("./tests/resourses/test_chargeback.csv", &[]);
    assert_success(&output);
    verify_output(&output, 2, &["1", "0", "0", "0", "true"]);
}

#[test]
fn test_malicious_chargeback_workflow() {
    let output = run_command("./tests/resourses/test_malicious_chargeback.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_unexpected_header() {
    let output = run_command("./tests/resourses/test_unexpected_header.csv", &[]);
    assert_failure(&output);
}

#[test]
fn test_two_clients() {
    let output = run_command("./tests/resourses/test_multiple_clients.csv", &[]);
    assert_success(&output);
    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 3);
    let mut client_data = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        client_data.push((fields[0], fields[1], fields[2], fields[3], fields[4]));
    }
    client_data.sort_by_key(|&x| x.0);
    assert_eq!(client_data[0].0, "1"); // client
    assert_eq!(client_data[0].1, "3"); // available
    assert_eq!(client_data[0].2, "0"); // held
    assert_eq!(client_data[0].3, "3"); // total
    assert_eq!(client_data[0].4, "false"); // not locked
    assert_eq!(client_data[1].0, "2"); // client
    assert_eq!(client_data[1].1, "10"); // available
    assert_eq!(client_data[1].2, "0"); // held
    assert_eq!(client_data[1].3, "10"); // total
    assert_eq!(client_data[1].4, "false"); // not locked
}
