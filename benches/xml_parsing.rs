//! Performance benchmarks for XML parsing and serialization.
//!
//! This module measures the performance of quick-xml
//! for parsing and serialization operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use xmile::xml::XmileFile;

fn parse_xml(xml: &str) -> Result<XmileFile, xmile::xml::ParseError> {
    XmileFile::from_str(xml)
}

fn serialize_xml(file: &XmileFile) -> Result<String, xmile::xml::serialize::SerializeError> {
    file.to_xml()
}

fn bench_parse_simple(c: &mut Criterion) {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test Vendor</vendor>
        <product version="1.0">Test Product</product>
    </header>
    <sim_specs>
        <start>0.0</start>
        <stop>10.0</stop>
    </sim_specs>
</xmile>"#;

    c.bench_function("parse_simple", |b| b.iter(|| parse_xml(black_box(xml))));
}

fn bench_parse_teacup(c: &mut Criterion) {
    let xml = include_str!("../data/examples/teacup.xmile");

    c.bench_function("parse_teacup", |b| b.iter(|| parse_xml(black_box(xml))));
}

fn bench_serialize_simple(c: &mut Criterion) {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test Vendor</vendor>
        <product version="1.0">Test Product</product>
    </header>
    <sim_specs>
        <start>0.0</start>
        <stop>10.0</stop>
    </sim_specs>
</xmile>"#;

    let file = parse_xml(xml).expect("Failed to parse test file");

    c.bench_function("serialize_simple", |b| {
        b.iter(|| serialize_xml(black_box(&file)))
    });
}

fn bench_serialize_teacup(c: &mut Criterion) {
    let xml = include_str!("../data/examples/teacup.xmile");
    let file = parse_xml(xml).expect("Failed to parse teacup file");

    c.bench_function("serialize_teacup", |b| {
        b.iter(|| serialize_xml(black_box(&file)))
    });
}

fn bench_round_trip(c: &mut Criterion) {
    let xml = include_str!("../data/examples/teacup.xmile");

    c.bench_function("round_trip_teacup", |b| {
        b.iter(|| {
            let file = parse_xml(black_box(xml)).expect("Failed to parse");
            let serialized = serialize_xml(&file).expect("Failed to serialize");
            let _file2 = parse_xml(&serialized).expect("Failed to re-parse");
        })
    });
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_teacup,
    bench_serialize_simple,
    bench_serialize_teacup,
    bench_round_trip
);
criterion_main!(benches);
