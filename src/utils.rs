pub fn to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte > 'a' as u8 && *byte <= 'z' as u8 {
            *byte -= 32;
        }
    }
}