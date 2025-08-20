use core::error::Error;

pub fn unescape<const X: bool>(input: &str) -> Result<String, Box<dyn Error>> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars
                .next()
                .ok_or("Incomplete escape sequence: trailing backslash")?
            {
                '0' => output.push('\0'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'x' if X => {
                    let h1 = chars
                        .next()
                        .ok_or("Incomplete escape sequence: expected two digits after '\\x'")?;
                    let h2 = chars
                        .next()
                        .ok_or("Incomplete escape sequence: expected two digits after '\\x'")?;
                    let hex_string = format!("{}{}", h1, h2);
                    let byte = u8::from_str_radix(&hex_string, 16)
                        .map_err(|_| format!("Invalid escape sequence: '\\x{}'", hex_string))?;
                    if byte > 0x7F {
                        Err(format!(
                            "Invalid escape sequence: \
                             '\\x{}' is outside the ASCII range (0x00–0x7F)",
                            hex_string
                        ))?;
                    }
                    let ch = byte as char;
                    output.push(ch);
                }
                other => {
                    Err(format!("Unknown escape sequence: '\\{}'", other))?;
                }
            }
        } else {
            output.push(ch);
        }
    }
    Ok(output)
}
