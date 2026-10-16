use std::path::{Path, PathBuf};

pub enum Destination<'u> {
    External,
    InvalidPath,
    Local {
        file: PathBuf,
        same_file: bool,
        fragment: Option<&'u str>,
    },
}

pub fn resolve<'u>(url: &'u str, from: &Path) -> Destination<'u> {
    if !is_local(url) {
        return Destination::External;
    }
    let (resource, fragment) = url
        .split_once('#')
        .map_or((url, None), |(resource, fragment)| {
            (resource, Some(fragment))
        });
    let resource = resource
        .split_once('?')
        .map_or(resource, |(resource, _)| resource);
    let Some(decoded) = percent_decode(resource) else {
        return Destination::InvalidPath;
    };
    let same_file = decoded.is_empty();
    let file = if same_file {
        from.to_path_buf()
    } else {
        from.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(decoded)
    };
    Destination::Local {
        file,
        same_file,
        fragment,
    }
}

pub fn is_local(url: &str) -> bool {
    if url.starts_with("//") {
        return false;
    }
    let Some((scheme, _)) = url.split_once(':') else {
        return true;
    };
    !(scheme.starts_with(|character: char| character.is_ascii_alphabetic())
        && scheme.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        }))
}

const HEX_RADIX: u32 = 16;

pub fn percent_decode(value: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = char::from(bytes.next()?).to_digit(HEX_RADIX)?;
            let low = char::from(bytes.next()?).to_digit(HEX_RADIX)?;
            decoded.push(u8::try_from(high * HEX_RADIX + low).ok()?);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded)
        .ok()
        .filter(|value| !value.contains('\0'))
}
