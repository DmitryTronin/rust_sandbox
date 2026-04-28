pub mod rpc {
    pub type RpcResult<T, E = RpcError> = Result<Option<T>, E>;

    #[derive(Debug, Default)]
    pub enum RpcError {
        ConnectionError,
        TimeoutError,
        #[default]
        DataError,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardAnalysis {
    pub total_chars: usize,
    pub total_bytes: usize,
    pub line_count: usize,
    pub invisible_count: usize,
    pub findings: Vec<HiddenCharacter>,
}

impl ClipboardAnalysis {
    pub fn to_json(&self) -> String {
        let findings = self
            .findings
            .iter()
            .map(HiddenCharacter::to_json)
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"total_chars\":{},\"total_bytes\":{},\"line_count\":{},\"invisible_count\":{},\"findings\":[{}]}}",
            self.total_chars, self.total_bytes, self.line_count, self.invisible_count, findings
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenCharacter {
    pub char_index: usize,
    pub byte_index: usize,
    pub line: usize,
    pub column: usize,
    pub code_point: String,
    pub name: &'static str,
    pub category: &'static str,
    pub marker: &'static str,
    pub description: &'static str,
}

impl HiddenCharacter {
    fn to_json(&self) -> String {
        format!(
            "{{\"char_index\":{},\"byte_index\":{},\"line\":{},\"column\":{},\"code_point\":\"{}\",\"name\":\"{}\",\"category\":\"{}\",\"marker\":\"{}\",\"description\":\"{}\"}}",
            self.char_index,
            self.byte_index,
            self.line,
            self.column,
            escape_json(self.code_point.as_str()),
            escape_json(self.name),
            escape_json(self.category),
            escape_json(self.marker),
            escape_json(self.description)
        )
    }
}

pub fn analyze_clipboard_text(input: &str) -> ClipboardAnalysis {
    let mut findings = Vec::new();
    let mut line = 1;
    let mut column = 1;
    let mut char_index = 0;
    let mut chars = input.char_indices().peekable();

    while let Some((byte_index, ch)) = chars.next() {
        if let Some(classification) = classify_hidden_character(ch) {
            findings.push(HiddenCharacter {
                char_index,
                byte_index,
                line,
                column,
                code_point: format!("U+{:04X}", ch as u32),
                name: classification.name,
                category: classification.category,
                marker: classification.marker,
                description: classification.description,
            });
        }

        let next_char = chars.peek().map(|(_, ch)| *ch);
        if ch == '\r' && next_char == Some('\n') {
            chars.next();
            char_index += 1;
            line += 1;
            column = 1;
        } else if is_line_break(ch) {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }

        char_index += 1;
    }

    ClipboardAnalysis {
        total_chars: input.chars().count(),
        total_bytes: input.len(),
        line_count: count_lines(input),
        invisible_count: findings.len(),
        findings,
    }
}

fn count_lines(input: &str) -> usize {
    let mut line_count = 1;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\r' && chars.peek() == Some(&'\n') {
            continue;
        }

        if is_line_break(ch) {
            line_count += 1;
        }
    }

    line_count
}

fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

#[derive(Debug, Clone, Copy)]
struct CharacterClassification {
    name: &'static str,
    category: &'static str,
    marker: &'static str,
    description: &'static str,
}

fn classify_hidden_character(ch: char) -> Option<CharacterClassification> {
    match ch {
        '\u{0000}' => control("NULL", "NUL", "String terminator/control character."),
        '\u{0008}' => control(
            "BACKSPACE",
            "BS",
            "Moves the cursor backward in terminal contexts.",
        ),
        '\u{0009}' => whitespace("CHARACTER TABULATION", "TAB", "Horizontal tab indentation."),
        '\u{000A}' => whitespace("LINE FEED", "LF", "Unix-style line ending."),
        '\u{000B}' => whitespace("LINE TABULATION", "VT", "Vertical tab whitespace."),
        '\u{000C}' => whitespace("FORM FEED", "FF", "Page-break style whitespace."),
        '\u{000D}' => whitespace(
            "CARRIAGE RETURN",
            "CR",
            "Classic Mac or Windows line-ending component.",
        ),
        '\u{001B}' => control("ESCAPE", "ESC", "Begins ANSI terminal escape sequences."),
        '\u{0020}' => None,
        '\u{007F}' => control("DELETE", "DEL", "Delete control character."),
        '\u{0080}'..='\u{009F}' => control("C1 CONTROL", "C1", "Unicode C1 control character."),
        '\u{00A0}' => no_break(
            "NO-BREAK SPACE",
            "NBSP",
            "Looks like a space but prevents line wrapping.",
        ),
        '\u{00AD}' => format_marker(
            "SOFT HYPHEN",
            "SHY",
            "Invisible unless a line breaks at this point.",
        ),
        '\u{034F}' => format_marker(
            "COMBINING GRAPHEME JOINER",
            "CGJ",
            "Affects combining mark ordering and rendering.",
        ),
        '\u{061C}' => bidi(
            "ARABIC LETTER MARK",
            "ALM",
            "Bidirectional text formatting marker.",
        ),
        '\u{115F}' => invisible_letter(
            "HANGUL CHOSEONG FILLER",
            "HCF",
            "Invisible Hangul filler character.",
        ),
        '\u{1160}' => invisible_letter(
            "HANGUL JUNGSEONG FILLER",
            "HJF",
            "Invisible Hangul filler character.",
        ),
        '\u{1680}' => whitespace("OGHAM SPACE MARK", "OSM", "Unicode space separator."),
        '\u{180E}' => format_marker(
            "MONGOLIAN VOWEL SEPARATOR",
            "MVS",
            "Legacy formatting character that may render invisibly.",
        ),
        '\u{2000}' => whitespace("EN QUAD", "ENQ", "Typography-width space."),
        '\u{2001}' => whitespace("EM QUAD", "EMQ", "Typography-width space."),
        '\u{2002}' => whitespace("EN SPACE", "ENSP", "Typography-width space."),
        '\u{2003}' => whitespace("EM SPACE", "EMSP", "Typography-width space."),
        '\u{2004}' => whitespace("THREE-PER-EM SPACE", "3/MSP", "Typography-width space."),
        '\u{2005}' => whitespace("FOUR-PER-EM SPACE", "4/MSP", "Typography-width space."),
        '\u{2006}' => whitespace("SIX-PER-EM SPACE", "6/MSP", "Typography-width space."),
        '\u{2007}' => no_break("FIGURE SPACE", "FIGSP", "Digit-width non-breaking space."),
        '\u{2008}' => whitespace("PUNCTUATION SPACE", "PSP", "Punctuation-width space."),
        '\u{2009}' => whitespace("THIN SPACE", "THSP", "Narrow typography space."),
        '\u{200A}' => whitespace("HAIR SPACE", "HSP", "Very narrow typography space."),
        '\u{200B}' => format_marker("ZERO WIDTH SPACE", "ZWSP", "Invisible break opportunity."),
        '\u{200C}' => format_marker(
            "ZERO WIDTH NON-JOINER",
            "ZWNJ",
            "Prevents adjacent characters from joining.",
        ),
        '\u{200D}' => format_marker(
            "ZERO WIDTH JOINER",
            "ZWJ",
            "Joins adjacent characters; common in emoji sequences.",
        ),
        '\u{200E}' => bidi(
            "LEFT-TO-RIGHT MARK",
            "LRM",
            "Bidirectional text formatting marker.",
        ),
        '\u{200F}' => bidi(
            "RIGHT-TO-LEFT MARK",
            "RLM",
            "Bidirectional text formatting marker.",
        ),
        '\u{2028}' => whitespace("LINE SEPARATOR", "LS", "Unicode line separator."),
        '\u{2029}' => whitespace("PARAGRAPH SEPARATOR", "PS", "Unicode paragraph separator."),
        '\u{202A}' => bidi(
            "LEFT-TO-RIGHT EMBEDDING",
            "LRE",
            "Bidirectional text formatting marker.",
        ),
        '\u{202B}' => bidi(
            "RIGHT-TO-LEFT EMBEDDING",
            "RLE",
            "Bidirectional text formatting marker.",
        ),
        '\u{202C}' => bidi(
            "POP DIRECTIONAL FORMATTING",
            "PDF",
            "Ends bidirectional embedding or override.",
        ),
        '\u{202D}' => bidi(
            "LEFT-TO-RIGHT OVERRIDE",
            "LRO",
            "Forces left-to-right ordering.",
        ),
        '\u{202E}' => bidi(
            "RIGHT-TO-LEFT OVERRIDE",
            "RLO",
            "Forces right-to-left ordering and can disguise text.",
        ),
        '\u{202F}' => no_break(
            "NARROW NO-BREAK SPACE",
            "NNBSP",
            "Narrow non-breaking space.",
        ),
        '\u{205F}' => whitespace(
            "MEDIUM MATHEMATICAL SPACE",
            "MMSP",
            "Mathematical typography space.",
        ),
        '\u{2060}' => no_break(
            "WORD JOINER",
            "WJ",
            "Invisible character that prevents line breaks.",
        ),
        '\u{2061}' => format_marker(
            "FUNCTION APPLICATION",
            "FA",
            "Invisible mathematical formatting operator.",
        ),
        '\u{2062}' => format_marker(
            "INVISIBLE TIMES",
            "IT",
            "Invisible mathematical multiplication operator.",
        ),
        '\u{2063}' => format_marker(
            "INVISIBLE SEPARATOR",
            "IS",
            "Invisible mathematical separator.",
        ),
        '\u{2064}' => format_marker(
            "INVISIBLE PLUS",
            "IP",
            "Invisible mathematical addition operator.",
        ),
        '\u{2066}' => bidi(
            "LEFT-TO-RIGHT ISOLATE",
            "LRI",
            "Bidirectional text isolation marker.",
        ),
        '\u{2067}' => bidi(
            "RIGHT-TO-LEFT ISOLATE",
            "RLI",
            "Bidirectional text isolation marker.",
        ),
        '\u{2068}' => bidi(
            "FIRST STRONG ISOLATE",
            "FSI",
            "Bidirectional text isolation marker.",
        ),
        '\u{2069}' => bidi(
            "POP DIRECTIONAL ISOLATE",
            "PDI",
            "Ends a bidirectional isolate.",
        ),
        '\u{2800}' => whitespace(
            "BRAILLE PATTERN BLANK",
            "BLANK",
            "Blank Braille pattern that appears empty.",
        ),
        '\u{3000}' => whitespace("IDEOGRAPHIC SPACE", "IDSP", "Full-width CJK space."),
        '\u{3164}' => invisible_letter("HANGUL FILLER", "HF", "Invisible Hangul filler character."),
        '\u{FEFF}' => no_break(
            "ZERO WIDTH NO-BREAK SPACE",
            "BOM",
            "Byte order mark; often appears as an invisible prefix.",
        ),
        '\u{FFA0}' => invisible_letter(
            "HALFWIDTH HANGUL FILLER",
            "HHF",
            "Invisible Hangul filler character.",
        ),
        ch if ch.is_control() => control("CONTROL CHARACTER", "CTRL", "Unicode control character."),
        ch if ch.is_whitespace() => whitespace(
            "UNICODE WHITESPACE",
            "WS",
            "Whitespace that may not render like a regular ASCII space.",
        ),
        _ => None,
    }
}

fn control(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "control",
        marker,
        description,
    })
}

fn whitespace(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "whitespace",
        marker,
        description,
    })
}

fn no_break(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "non-breaking",
        marker,
        description,
    })
}

fn format_marker(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "format",
        marker,
        description,
    })
}

fn bidi(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "bidirectional-format",
        marker,
        description,
    })
}

fn invisible_letter(
    name: &'static str,
    marker: &'static str,
    description: &'static str,
) -> Option<CharacterClassification> {
    Some(CharacterClassification {
        name,
        category: "invisible-letter",
        marker,
        description,
    })
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04X}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

pub enum MyEnum<T, E = rpc::RpcError> {
    Ok(rpc::RpcResult<T, E>),
    Err(E),
}

impl<T, E: Default> MyEnum<T, E> {
    pub fn flatten(self) -> Result<T, E> {
        match self {
            MyEnum::Ok(Ok(Some(value))) => Ok(value),
            MyEnum::Ok(Ok(None)) => Err(Default::default()),
            MyEnum::Ok(Err(e)) => Err(e),
            MyEnum::Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_ok_some() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(Some("hello".to_owned())));
        assert!(matches!(val.flatten(), Ok(s) if s == "hello"));
    }

    #[test]
    fn test_flatten_ok_none_returns_default_err() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(None));
        assert!(val.flatten().is_err());
    }

    #[test]
    fn test_flatten_err_propagates() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Err(rpc::RpcError::ConnectionError);
        assert!(matches!(val.flatten(), Err(rpc::RpcError::ConnectionError)));
    }

    #[test]
    fn test_flatten_inner_err_propagates() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Err(rpc::RpcError::TimeoutError));
        assert!(matches!(val.flatten(), Err(rpc::RpcError::TimeoutError)));
    }

    #[test]
    fn test_analyze_clipboard_text_finds_hidden_characters() {
        let analysis = analyze_clipboard_text("a\u{00A0}b\u{200B}c\u{202E}d");

        assert_eq!(analysis.total_chars, 7);
        assert_eq!(analysis.invisible_count, 3);
        assert_eq!(analysis.findings[0].code_point, "U+00A0");
        assert_eq!(analysis.findings[0].name, "NO-BREAK SPACE");
        assert_eq!(analysis.findings[1].name, "ZERO WIDTH SPACE");
        assert_eq!(analysis.findings[2].name, "RIGHT-TO-LEFT OVERRIDE");
    }

    #[test]
    fn test_analyze_clipboard_text_tracks_line_and_column() {
        let analysis = analyze_clipboard_text("first\r\nx\u{200D}y\n");

        assert_eq!(analysis.line_count, 3);
        assert_eq!(analysis.findings.len(), 3);
        assert_eq!(analysis.findings[0].name, "CARRIAGE RETURN");
        assert_eq!(analysis.findings[0].line, 1);
        assert_eq!(analysis.findings[0].column, 6);
        assert_eq!(analysis.findings[1].name, "ZERO WIDTH JOINER");
        assert_eq!(analysis.findings[1].char_index, 8);
        assert_eq!(analysis.findings[1].line, 2);
        assert_eq!(analysis.findings[1].column, 2);
        assert_eq!(analysis.findings[2].name, "LINE FEED");
        assert_eq!(analysis.findings[2].line, 2);
        assert_eq!(analysis.findings[2].column, 4);
    }

    #[test]
    fn test_count_lines_handles_unicode_line_separators() {
        let analysis = analyze_clipboard_text("one\u{2028}two");

        assert_eq!(analysis.line_count, 2);
        assert_eq!(analysis.findings[0].name, "LINE SEPARATOR");
    }

    #[test]
    fn test_analysis_serializes_as_json() {
        let json = analyze_clipboard_text("a\u{2060}b").to_json();

        assert!(json.contains("\"code_point\":\"U+2060\""));
        assert!(json.contains("\"name\":\"WORD JOINER\""));
        assert!(json.contains("\"category\":\"non-breaking\""));
    }
}
