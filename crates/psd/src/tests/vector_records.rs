use super::*;

// Independent wire fixtures: do not use the shared path encoder to test its decoder.
fn record(selector: u16, data: [u8; 24]) -> Vec<u8> {
    let mut out = selector.to_be_bytes().to_vec();
    out.extend_from_slice(&data);
    out
}

fn length(selector: u16, count: u16) -> Vec<u8> {
    let mut data = [0; 24];
    data[..2].copy_from_slice(&count.to_be_bytes());
    record(selector, data)
}

fn knot(selector: u16) -> Vec<u8> {
    let mut data = [0; 24];
    // Stored y then x, including negative and outside-document coordinates.
    for (chunk, value) in data.as_chunks_mut::<4>().0.iter_mut().zip([-(1i32 << 23), 1 << 22, 1 << 24, -(1 << 24), 1 << 23, 1 << 25]) {
        chunk.copy_from_slice(&value.to_be_bytes());
    }
    record(selector, data)
}

fn parse_mask(key: [u8; 4], bytes: Vec<u8>) -> Option<VectorMask> {
    let mut doc = WDoc::new(64, 32);
    doc.layers = vec![WLayer::empty("mask", vec![(key, bytes)])];
    Psd::parse(write(&doc)).unwrap().layers.remove(0).vector_mask
}

fn header(version: u32, flags: u32) -> Vec<u8> {
    [version.to_be_bytes(), flags.to_be_bytes()].concat()
}

#[test]
fn vector_masks_preserve_record_recovery_and_dimension_mapping() {
    for key in [*b"vmsk", *b"vsms"] {
        for version in [3, 99] {
            for tail in 0..26 {
                let mut bytes = header(version, 1 | 2 | 4 | 0x80);
                bytes.extend(knot(1)); // Orphan knots are ignored, not made into a path.
                bytes.extend(length(0, u16::MAX)); // Empty subpath is removed.
                bytes.extend(length(3, 0)); // Counts do not control app grouping.
                for selector in [1, 2, 4, 5] {
                    bytes.extend(knot(selector)); // Keep mixed selectors in the open path.
                }
                bytes.extend(record(0xffff, [0xff; 24]));
                bytes.extend(length(0, 1));
                bytes.extend(knot(4));
                bytes.extend(knot(2)); // More knots than declared still belong to this path.
                bytes.extend(length(3, 1)); // Empty final path is removed too.
                bytes.extend(std::iter::repeat_n(0xff, tail));
                let mask = parse_mask(key, bytes).unwrap();
                assert!(mask.inverted && mask.disabled);
                assert_eq!(mask.subpaths.len(), 2);
                assert!(!mask.subpaths[0].closed);
                assert!(mask.subpaths[1].closed);
                assert_eq!(mask.subpaths[0].knots.len(), 4);
                assert_eq!(mask.subpaths[1].knots.len(), 2);
                for path in mask.subpaths {
                    for points in path.knots {
                        assert_eq!(points, [[16.0, -16.0], [-64.0, 32.0], [128.0, 16.0]]);
                    }
                }
            }
        }
    }
}

#[test]
fn vector_masks_preserve_short_header_and_empty_path_recovery() {
    for key in [*b"vmsk", *b"vsms"] {
        let bytes = header(3, 0);
        for len in 0..8 {
            assert!(parse_mask(key, bytes[..len].to_vec()).is_none());
        }
        for records in [vec![], knot(5), length(3, u16::MAX), record(8, [0xff; 24])] {
            let mut bytes = header(3, 2 | 0x80);
            bytes.extend(records);
            let mask = parse_mask(key, bytes).unwrap();
            assert!(!mask.inverted && !mask.disabled);
            assert!(mask.subpaths.is_empty());
        }
    }
}

#[test]
fn vector_writer_fixed_point_edge_bytes_are_preserved() {
    // Expected signed 8.24 values are literal constants, not the shared encoder.
    for (fraction, fixed) in [
        (0.0, 0i32),
        (-0.0, 0),
        (1.0 / 33_554_432.0, 1),
        (-1.0 / 33_554_432.0, -1),
        (127.999_999_940_395_36, 2_147_483_647),
        (128.0, 2_147_483_647),
        (-128.0, -2_147_483_648),
        (-129.0, -2_147_483_648),
        (f64::MAX, 2_147_483_647),
        (f64::MIN, -2_147_483_648),
        (f64::INFINITY, 2_147_483_647),
        (f64::NEG_INFINITY, -2_147_483_648),
        (f64::NAN, 0),
    ] {
        let block = vector_mask_block(&[vec![[[fraction, fraction]; 3]]], 1, 1, false);
        assert_eq!(block.0, *b"vmsk");
        let mut expected = vec![0, 0, 0, 3, 0, 0, 0, 0, 0, 6];
        expected.extend([0; 24]); // Fill rule.
        expected.extend([0, 0, 0, 1]);
        expected.extend([0; 22]); // Closed length, count one, reserved zeros.
        expected.extend([0, 1]); // Linked closed knot.
        for _ in 0..6 {
            expected.extend(fixed.to_be_bytes());
        }
        assert_eq!(block.1, expected, "{fraction:?}");
    }
}

#[test]
fn vector_writer_normalization_and_y_first_order_are_preserved() {
    let block = vector_mask_block(&[vec![[[2.0, 6.0], [-2.0, 3.0], [4.0, -6.0]]]], 4, 6, true);
    assert_eq!(&block.1[..8], &[0, 0, 0, 3, 0, 0, 0, 1]);
    assert_eq!(&block.1[62..], &[1, 0, 0, 0, 0, 128, 0, 0, 0, 128, 0, 0, 255, 128, 0, 0, 255, 0, 0, 0, 1, 0, 0, 0]);
    let zero = vector_mask_block(&[vec![[[1.0, -1.0], [0.0, -0.0], [-1.0, 1.0]]]], 0, 0, false);
    assert_eq!(&zero.1[62..], &[128, 0, 0, 0, 127, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 127, 255, 255, 255, 128, 0, 0, 0]);
}
