use binread::{io::Cursor, BinRead, BinReaderExt};

#[test]
fn const_generic_test() {
    const IN: [u8; 300] = [6; 300];

    let mut reader = Cursor::new(&IN);
    let out: [u8; 300] = reader.read_be().unwrap();

    assert_eq!(IN, out);
}
