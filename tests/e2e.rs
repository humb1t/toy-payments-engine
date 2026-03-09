#![allow(clippy::unwrap_used)]

use std::process::Command;

#[test]
fn test_basic_deposit_withdrawal() {
    let input_file = "./tests/resourses/test_basic.csv";
    let output = Command::new("cargo")
        .args(["run", "--", input_file])
        .output()
        .expect("Failed to execute cargo run");
    assert!(output.status.success(), "{output:?}");
    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 2);
    let data_line = lines[1];
    let fields: Vec<&str> = data_line.split(',').collect();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0], "1");
    assert_eq!(fields[1], "5.0");
    assert_eq!(fields[2], "0");
    assert_eq!(fields[3], "5.0");
    assert_eq!(fields[4], "false");
}

#[test]
fn test_dispute_resolve_workflow() {
    let input_file = "./tests/resourses/test_dispute_resolve.csv";
    let output = Command::new("cargo")
        .args(["run", "--", input_file])
        .output()
        .expect("Failed to execute cargo run");
    assert!(output.status.success(), "{output:?}");
    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 2);
    let data_line = lines[1];
    let fields: Vec<&str> = data_line.split(',').collect();
    assert_eq!(fields[0], "1");
    assert_eq!(fields[1], "10.0");
    assert_eq!(fields[2], "0.0");
    assert_eq!(fields[3], "10.0");
    assert_eq!(fields[4], "false");
}

#[test]
fn test_chargeback_workflow() {
    let input_file = "./tests/resourses/test_chargeback.csv";
    let output = Command::new("cargo")
        .args(["run", "--", input_file])
        .output()
        .expect("Failed to execute cargo run");
    assert!(output.status.success(), "{output:?}");
    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 2);
    let data_line = lines[1];
    let fields: Vec<&str> = data_line.split(',').collect();
    assert_eq!(fields[0], "1"); // client
    assert_eq!(fields[1], "0.0"); // available
    assert_eq!(fields[2], "0.0"); // held
    assert_eq!(fields[3], "0.0"); // total 
    assert_eq!(fields[4], "true"); // locked
}

#[test]
fn test_multiple_clients() {
    let input_file = "./tests/resourses/test_multiple_clients.csv";
    let output = Command::new("cargo")
        .args(["run", "--", input_file])
        .output()
        .expect("Failed to execute cargo run");
    assert!(output.status.success(), "{output:?}");
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
    assert_eq!(client_data[0].1, "3.0"); // available 
    assert_eq!(client_data[0].2, "0"); // held
    assert_eq!(client_data[0].3, "3.0"); // total
    assert_eq!(client_data[0].4, "false"); // not locked
    assert_eq!(client_data[1].0, "2"); // client
    assert_eq!(client_data[1].1, "10.0"); // available 
    assert_eq!(client_data[1].2, "0"); // held
    assert_eq!(client_data[1].3, "10.0"); // total
    assert_eq!(client_data[1].4, "false"); // not locked
}
