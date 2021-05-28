use std::{fmt, ops::Range};

use crate::{Buffer, Error, ErrorKind::*, err::perr, escape::unescape};


/// A byte string or raw byte string literal, e.g. `b"hello"` or `br#"abc"def"#`.
///
/// See [the reference][ref] for more information.
///
/// [ref]: https://doc.rust-lang.org/reference/tokens.html#byte-string-literals
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteStringLit<B: Buffer> {
    /// The raw input.
    raw: B,

    /// The string value (with all escaped unescaped), or `None` if there were
    /// no escapes. In the latter case, `input` is the string value.
    value: Option<Vec<u8>>,

    /// The number of hash signs in case of a raw string literal, or `None` if
    /// it's not a raw string literal.
    num_hashes: Option<u32>,
}

impl<B: Buffer> ByteStringLit<B> {
    /// Parses the input as a (raw) byte string literal. Returns an error if the
    /// input is invalid or represents a different kind of literal.
    pub fn parse(input: B) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(perr(None, Empty));
        }
        if !input.starts_with(r#"b""#) && !input.starts_with("br") {
            return Err(perr(None, InvalidByteStringLiteralStart));
        }

        Self::parse_impl(input)
    }

    /// Returns the string value this literal represents (where all escapes have
    /// been turned into their respective values).
    pub fn value(&self) -> &[u8] {
        self.value.as_deref().unwrap_or(&self.raw.as_bytes()[self.inner_range()])
    }

    /// Like `value` but returns a potentially owned version of the value.
    ///
    /// The return value is either `Cow<'static, [u8]>` if `B = String`, or
    /// `Cow<'a, [u8]>` if `B = &'a str`.
    pub fn into_value(self) -> B::ByteCow {
        let inner_range = self.inner_range();
        let Self { raw, value, .. } = self;
        value.map(B::ByteCow::from).unwrap_or_else(|| raw.cut(inner_range).into_byte_cow())
    }

    /// Returns whether this literal is a raw string literal (starting with
    /// `r`).
    pub fn is_raw_byte_string(&self) -> bool {
        self.num_hashes.is_some()
    }

    /// The range within `self.raw` that excludes the quotes and potential `r#`.
    fn inner_range(&self) -> Range<usize> {
        match self.num_hashes {
            None => 2..self.raw.len() - 1,
            Some(n) => 2 + n as usize + 1..self.raw.len() - n as usize - 1,
        }
    }

    /// Precondition: input has to start with either `b"` or `br`.
    pub(crate) fn parse_impl(input: B) -> Result<Self, Error> {
        if input.starts_with(r"br") {
            // Raw string literal
            let num_hashes = input[2..].bytes().position(|b| b != b'#')
                .ok_or(perr(None, InvalidLiteral))?;

            if input.as_bytes().get(2 + num_hashes) != Some(&b'"') {
                return Err(perr(None, InvalidLiteral));
            }
            let start_inner = 2 + num_hashes + 1;
            let hashes = &input[2..num_hashes + 2];

            let mut closing_quote_pos = None;
            for (i, b) in input[start_inner..].bytes().enumerate() {
                if b == b'"' && input[start_inner + i + 1..].starts_with(hashes) {
                    closing_quote_pos = Some(i + start_inner);
                    break;
                }

                if !b.is_ascii() {
                    return Err(perr(i + start_inner, NonAsciiInByteLiteral));
                }
            }
            let closing_quote_pos = closing_quote_pos
                .ok_or(perr(None, UnterminatedRawString))?;

            if closing_quote_pos + num_hashes != input.len() - 1 {
                return Err(perr(closing_quote_pos + num_hashes + 1..input.len(), UnexpectedChar));
            }

            Ok(Self {
                raw: input,
                value: None,
                num_hashes: Some(num_hashes as u32),
            })
        } else {
            let mut i = 2;
            let mut end_last_escape = 2;
            let mut value = Vec::new();
            while i < input.len() - 1 {
                match input.as_bytes()[i] {
                    b'\\' => {
                        let (b, len) = unescape::<u8>(&input[i..input.len() - 1], i)?;
                        value.extend_from_slice(&input.as_bytes()[end_last_escape..i]);
                        value.push(b);
                        i += len;
                        end_last_escape = i;
                    }
                    b'\r' if input.as_bytes()[i + 1] != b'\n'
                        => return Err(perr(i, IsolatedCr)),
                    b'"' => return Err(perr(i + 1..input.len(), UnexpectedChar)),
                    b if !b.is_ascii()
                        => return Err(perr(i, NonAsciiInByteLiteral)),
                    _ => i += 1,
                }
            }

            if input.as_bytes()[input.len() - 1] != b'"' || input.len() == 2 {
                return Err(perr(None, UnterminatedString));
            }

            // `value` is only empty there was no escape in the input string
            // (with the special case of the input being empty). This means the
            // string value basically equals the input, so we store `None`.
            let value = if value.is_empty() {
                None
            } else {
                // There was an escape in the string, so we need to push the
                // remaining unescaped part of the string still.
                value.extend_from_slice(&input.as_bytes()[end_last_escape..input.len() - 1]);
                Some(value)
            };
            Ok(Self {
                raw: input,
                value,
                num_hashes: None,
            })
        }
    }
}

impl ByteStringLit<&str> {
    /// Makes a copy of the underlying buffer and returns the owned version of
    /// `Self`.
    pub fn into_owned(self) -> ByteStringLit<String> {
        ByteStringLit {
            raw: self.raw.to_owned(),
            value: self.value,
            num_hashes: self.num_hashes,
        }
    }
}

impl<B: Buffer> fmt::Display for ByteStringLit<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.raw)
    }
}


#[cfg(test)]
mod tests;
