mod common;

#[test]
fn scan_report_serializes_as_expected() {
    let report = common::minimal_scan_report();
    let value = serde_json::to_value(&report).unwrap();
    insta::assert_json_snapshot!(value);
}
