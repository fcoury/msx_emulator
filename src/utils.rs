pub fn hexdump(data: &[u8]) -> String {
    let mut data = data;
    let mut output = String::new();
    let mut address: usize = 0;

    while !data.is_empty() {
        let (chunk, remaining) = data.split_at(std::cmp::min(16, data.len()));

        output.push_str(&format!("{:08X}   ", address));

        for byte in chunk.iter() {
            output.push_str(&format!("{:02X} ", byte));
        }

        for _ in chunk.len()..16 {
            output.push_str("   ");
        }

        output.push_str("  ");

        for ch in chunk.iter() {
            if ch.is_ascii_graphic() {
                output.push(*ch as char);
            } else {
                output.push('.');
            }
            output.push(' ');
        }

        output.push('\n');
        address += chunk.len();
        data = remaining;
    }

    output
}
