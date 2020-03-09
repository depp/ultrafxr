fn main() {
    const MASK_PLAIN: u8 = 1 << 0;
    const MASK_SINGLE: u8 = 1 << 1;
    const MASK_NOESCAPE: u8 = 1 << 2;
    const MASK_ALL: u8 = MASK_PLAIN | MASK_SINGLE | MASK_NOESCAPE;
    let mut arr: [u8; 128] = [0; 128];
    for c in 32..=126 {
        arr[c] = MASK_SINGLE | MASK_NOESCAPE;
    }
    for c in b'A'..=b'Z' {
        arr[c as usize] = MASK_ALL;
    }
    for c in b'a'..=b'z' {
        arr[c as usize] = MASK_ALL;
    }
    for c in b'0'..=b'9' {
        arr[c as usize] = MASK_ALL;
    }
    for &c in b"%+,-./:=@_".iter() {
        arr[c as usize] = MASK_ALL;
    }
    arr[b'\'' as usize] = MASK_NOESCAPE;
    for &c in b"$\"\\`" {
        arr[c as usize] = MASK_SINGLE;
    }
    for row in arr[..].chunks(16) {
        for &cell in row.iter() {
            print!(" {:2},", cell);
        }
        println!(" //");
    }
}
