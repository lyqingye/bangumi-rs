use uncased::UncasedStr;

use crate::{
    keyword::{Keyword, KeywordConfig},
    utils::get_pair_mut,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TokenKind {
    OpenBracket,
    CloseBracket,
    Delimiter,
    Keyword,
    Text,
    Number,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) value: &'a str,
    pub(crate) keyword: Option<Keyword>,
    pub(crate) unknown: bool,
    pub(crate) is_enclosed: bool,
    pub(crate) position: usize,
}

impl<'a> Token<'a> {
    pub(crate) const fn open_bracket(value: &'a str) -> Self {
        Self {
            kind: TokenKind::OpenBracket,
            value,
            keyword: None,
            unknown: true,
            is_enclosed: false,
            position: 0,
        }
    }

    pub(crate) const fn close_bracket(value: &'a str) -> Self {
        Self {
            kind: TokenKind::CloseBracket,
            value,
            keyword: None,
            unknown: true,
            is_enclosed: false,
            position: 0,
        }
    }

    pub(crate) const fn delimiter(value: &'a str, is_enclosed: bool) -> Self {
        Self {
            kind: TokenKind::Delimiter,
            value,
            keyword: None,
            unknown: true,
            is_enclosed,
            position: 0,
        }
    }

    pub(crate) const fn text(value: &'a str, kind: TokenKind, is_enclosed: bool) -> Self {
        Self {
            kind,
            value,
            keyword: None,
            unknown: true,
            is_enclosed,
            position: 0,
        }
    }

    pub(crate) const fn from_keyword(value: &'a str, keyword: Keyword, is_enclosed: bool) -> Self {
        Self {
            kind: TokenKind::Keyword,
            value,
            keyword: Some(keyword),
            unknown: true,
            is_enclosed,
            position: 0,
        }
    }

    pub(crate) fn mark_known(&mut self) {
        self.unknown = false;
    }

    fn with_enclosed(self, is_enclosed: bool) -> Self {
        Self {
            is_enclosed,
            ..self
        }
    }

    pub(crate) const fn is_identified(&self) -> bool {
        !self.unknown
    }

    pub(crate) const fn is_free(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Text | TokenKind::Number | TokenKind::Keyword
        ) && self.unknown
    }

    pub(crate) const fn is_open_bracket(&self) -> bool {
        matches!(self.kind, TokenKind::OpenBracket)
    }

    pub(crate) const fn is_closed_bracket(&self) -> bool {
        matches!(self.kind, TokenKind::CloseBracket)
    }

    pub(crate) const fn is_bracket(&self) -> bool {
        matches!(self.kind, TokenKind::OpenBracket | TokenKind::CloseBracket)
    }

    pub(crate) const fn is_delimiter(&self) -> bool {
        matches!(self.kind, TokenKind::Delimiter)
    }

    pub(crate) const fn is_not_delimiter(&self) -> bool {
        !matches!(self.kind, TokenKind::Delimiter)
    }

    pub(crate) fn is_mostly_numbers(&self) -> bool {
        if self.is_number() {
            true
        } else if self.is_text() {
            let codepoints = self.value.chars().count();
            let numbers = self.value.chars().filter(|c| c.is_ascii_digit()).count();
            // Don't allow e.g. S01 to be merged
            numbers * 2 >= codepoints && self.value.chars().next().is_some_and(|s| s != 'S')
        } else {
            false
        }
    }

    // pub(crate) const fn is_keyword(&self) -> bool {
    //     matches!(self.kind, TokenKind::Keyword)
    // }

    pub(crate) const fn is_number(&self) -> bool {
        matches!(self.kind, TokenKind::Number)
    }

    pub(crate) const fn is_text(&self) -> bool {
        matches!(self.kind, TokenKind::Text)
    }
}

const fn is_open_bracket(ch: char) -> bool {
    match ch {
        '(' => true,        // parenthesis
        '[' => true,        // square bracket
        '{' => true,        // curly bracket
        '\u{300C}' => true, // corner bracket
        '\u{300E}' => true, // white corner bracket
        '\u{3010}' => true, // black lenticular bracket
        '\u{FF08}' => true, // fullwidth parenthesis
        '\u{FF3B}' => true, // fullwidth square bracket
        '\u{FF5B}' => true, // fullwidth curly bracket
        _ => false,
    }
}

const fn is_closed_bracket(ch: char) -> bool {
    match ch {
        ')' => true,        // parenthesis
        ']' => true,        // square bracket
        '}' => true,        // curly bracket
        '\u{300D}' => true, // corner bracket
        '\u{300F}' => true, // white corner bracket
        '\u{3011}' => true, // black lenticular bracket
        '\u{FF09}' => true, // fullwidth parenthesis
        '\u{FF3D}' => true, // fullwidth square bracket
        '\u{FF5D}' => true, // fullwidth curly bracket
        _ => false,
    }
}

pub(crate) const fn opposite_bracket(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),               // parenthesis
        '[' => Some(']'),               // square bracket
        '{' => Some('}'),               // curly bracket
        '\u{300C}' => Some('\u{300D}'), // corner bracket
        '\u{300E}' => Some('\u{300F}'), // white corner bracket
        '\u{3010}' => Some('\u{3011}'), // black lenticular bracket
        '\u{FF08}' => Some('\u{FF09}'), // fullwidth parenthesis
        '\u{FF3B}' => Some('\u{FF3D}'), // fullwidth square bracket
        '\u{FF5B}' => Some('\u{FF5D}'), // fullwidth curly bracket
        ')' => Some('('),               // parenthesis
        ']' => Some('['),               // square bracket
        '}' => Some('{'),               // curly bracket
        '\u{300D}' => Some('\u{300C}'), // corner bracket
        '\u{300F}' => Some('\u{300E}'), // white corner bracket
        '\u{3011}' => Some('\u{3010}'), // black lenticular bracket
        '\u{FF09}' => Some('\u{FF08}'), // fullwidth parenthesis
        '\u{FF3D}' => Some('\u{FF3B}'), // fullwidth square bracket
        '\u{FF5D}' => Some('\u{FF5B}'), // fullwidth curly bracket
        _ => None,
    }
}

const fn is_bracket(ch: char) -> bool {
    is_open_bracket(ch) || is_closed_bracket(ch)
}

pub(crate) const fn is_dash(ch: char) -> bool {
    match ch {
        '-' => true,        // hyphen-minus
        '\u{00AD}' => true, // soft hyphen
        '\u{2010}' => true, // hyphen
        '\u{2011}' => true, // non-breaking hyphen
        '\u{2012}' => true, // figure dash
        '\u{2013}' => true, // en dash
        '\u{2014}' => true, // em dash
        '\u{2015}' => true, // horizontal bar
        _ => false,
    }
}

const fn is_space(ch: char) -> bool {
    match ch {
        ' ' => true,        // space
        '\t' => true,       // character tabulation
        '\u{00A0}' => true, // no-break space
        '\u{200B}' => true, // zero width space
        '\u{3000}' => true, // ideographic space
        _ => false,
    }
}

const fn is_delimiter(ch: char) -> bool {
    match ch {
        '_' => true, // used instead of space
        '.' => true, // used instead of space, problematic (e.g. `AAC2.0.H.264`)
        ',' => true, // used to separate keywords
        '&' => true, // used for episode ranges
        '+' => true, // used in torrent titles
        '|' => true, // used in torrent titles, reserved in Windows
        _ => is_space(ch) || is_dash(ch),
    }
}

const fn is_text(ch: char) -> bool {
    !is_bracket(ch) && !is_delimiter(ch)
}

fn is_keyword_boundary(s: &str) -> bool {
    s.chars().next().map(|ch| !is_text(ch)).unwrap_or(true)
}

#[derive(Debug)]
pub(crate) struct Tokenizer<'a> {
    input: &'a str,
    keywords: &'a KeywordConfig,
}

impl<'a> Tokenizer<'a> {
    pub(crate) fn new(s: &'a str, keywords: &'a KeywordConfig) -> Self {
        Self { input: s, keywords }
    }

    const fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    fn take_keyword(&mut self) -> Option<(&'a str, Keyword)> {
        let mut key = "";
        for (index, ch) in self.input.char_indices() {
            let prefix = &self.input[0..(index + ch.len_utf8())];
            if self.keywords.get_keyword(prefix).is_some() {
                key = prefix;
            }
            if self
                .keywords
                .keywords()
                .keys()
                .filter(|k| k.starts_with(&prefix.to_lowercase()))
                .count()
                > 0
            {
                continue;
            }
            if key.is_empty() {
                return None;
            }
        }

        let n = key.len();
        let keyword = self.keywords.get_keyword(key).cloned()?;
        let rest = &self.input[n..];
        if keyword.is_bounded() && !is_keyword_boundary(rest) {
            // Allow things like "ED2" or "Season2"
            // Negate the condition to return early
            if !(keyword.is_ambiguous()
                && rest
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(true))
            {
                return None;
            }
        }
        let (before, after) = self.input.split_at(n);
        if before.is_empty() {
            None
        } else {
            self.input = after;
            Some((before, keyword))
        }
    }

    fn take_text(&mut self) -> &'a str {
        if let Some((index, ch)) = self
            .input
            .char_indices()
            .take_while(|&(_, ch)| is_text(ch))
            .last()
        {
            let new_index = index + ch.len_utf8();
            let (before, after) = self.input.split_at(new_index);
            self.input = after;
            before
        } else {
            ""
        }
    }

    fn take_if<F>(&mut self, predicate: F) -> Option<&'a str>
    where
        F: Fn(char) -> bool,
    {
        let ch = self.input.chars().next()?;
        if predicate(ch) {
            let value = &self.input[0..ch.len_utf8()];
            self.input = &self.input[ch.len_utf8()..];
            Some(value)
        } else {
            None
        }
    }

    pub(crate) fn tokens(self) -> Vec<Token<'a>> {
        let original = self.input;
        let mut tokens = self.into_iter().collect::<Vec<_>>();
        let mut start_length = 0;

        // Fix up and combine some tokens (e.g. 1 '.' 2 => '1.2')
        for index in 0..tokens.len() {
            let token_length = tokens[index].value.len();
            let is_dot = tokens[index].value == ".";
            // Combine tokens separated by a delimiter if they're mostly numbers
            // e.g. 009-1 or 01+02
            if tokens[index].is_delimiter()
                && tokens[index].value.starts_with(['.', '-', '&', '+', '~'])
            {
                if let Some((previous, next)) = get_pair_mut(&mut tokens, index - 1, index + 1) {
                    // Handle cases like No.N as well as 1.11
                    if (previous.is_mostly_numbers() && next.is_mostly_numbers())
                        || (is_dot
                            && previous.is_text()
                            && previous.value == UncasedStr::new("No")
                            && next.is_number())
                    {
                        previous.kind = TokenKind::Invalid;
                        next.kind = TokenKind::Invalid;
                        let start = start_length - previous.value.len();
                        let end = start_length + token_length + next.value.len();
                        // Create a new combined token anchored by the middle delimiter
                        tokens[index].kind = TokenKind::Text;
                        tokens[index].value = &original[start..end];
                    }
                }
            }
            start_length += token_length;
        }

        tokens.retain(|t| t.kind != TokenKind::Invalid);
        for (index, token) in tokens.iter_mut().enumerate() {
            token.position = index;
        }
        tokens
    }
}

pub(crate) struct TokenIterator<'a> {
    tokens: Tokenizer<'a>,
    bracket_level: usize,
}

impl<'a> TokenIterator<'a> {
    pub(crate) fn new(tokens: Tokenizer<'a>) -> Self {
        Self {
            tokens,
            bracket_level: 0,
        }
    }
}

impl<'a> IntoIterator for Tokenizer<'a> {
    type Item = Token<'a>;

    type IntoIter = TokenIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator::new(self)
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.is_empty() {
            return None;
        }

        let is_enclosed = self.bracket_level > 0;

        if let Some(value) = self.tokens.take_if(is_open_bracket) {
            self.bracket_level += 1;
            return Some(Token::open_bracket(value).with_enclosed(self.bracket_level >= 2));
        }

        if let Some(value) = self.tokens.take_if(is_closed_bracket) {
            self.bracket_level -= 1;
            return Some(Token::close_bracket(value).with_enclosed(self.bracket_level >= 1));
        }

        if let Some(value) = self.tokens.take_if(is_delimiter) {
            return Some(Token::delimiter(value, is_enclosed));
        }

        match self.tokens.take_keyword() {
            Some((value, keyword)) => Some(Token::from_keyword(value, keyword, is_enclosed)),
            None => {
                let text = self.tokens.take_text();
                if text.is_empty() {
                    None
                } else if text.as_bytes().iter().all(u8::is_ascii_digit) {
                    Some(Token::text(text, TokenKind::Number, is_enclosed))
                } else {
                    Some(Token::text(text, TokenKind::Text, is_enclosed))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeepDelimiters {
    Yes,
    No,
}

/// Combines and creates a string from the given tokens
pub(crate) fn combine_tokens(tokens: &[Token<'_>], keep: KeepDelimiters) -> String {
    let delimiters = tokens
        .iter()
        .filter(|t| t.is_delimiter())
        .filter_map(|t| t.value.chars().next())
        .collect::<Vec<_>>();
    let has_single_delimiter = delimiters
        .first()
        .map(|f| delimiters.iter().all(|x| x == f))
        .unwrap_or(false);
    let has_spaces = delimiters.iter().copied().any(is_space);
    let has_underscores = delimiters.iter().any(|&x| x == '_');

    let is_transformable_delimiter = |token: &Token<'_>| {
        if keep == KeepDelimiters::Yes {
            return false;
        }
        if token.is_not_delimiter() {
            return false;
        }
        let Some(ch) = token.value.chars().next() else {
            return false;
        };
        if ch == ',' || ch == '&' {
            false
        } else if is_space(ch) || ch == '_' {
            true
        } else if has_spaces || has_underscores {
            false
        } else if ch == '.' {
            true
        } else {
            has_single_delimiter
        }
    };

    // if keep == KeepDelimiters::No {
    //     // Find the last index that isn't a delimiter
    //     // Periods are fine
    //     if let Some((index, _)) = tokens
    //         .iter()
    //         .enumerate()
    //         .rfind(|(_, token)| token.is_not_delimiter())
    //     {
    //         tokens = &tokens[..(index + 1)];
    //     }
    // }

    let mut buffer = String::new();
    for token in tokens {
        if is_transformable_delimiter(token) {
            buffer.push(' ');
        } else {
            buffer.push_str(token.value);
        }
    }

    if keep == KeepDelimiters::No {
        // Trim the string from having delimiters such as dashes or spaces
        let trimmed = buffer.trim_matches(|ch| ch == ' ' || is_dash(ch));
        // Do an in-place trim instead of allocating a new buffer
        let ptr = trimmed.as_ptr();
        let len = trimmed.len();

        // SAFETY: The invariants of the length and values of this ptr are satisfied
        // by the function above.
        // This is nothing more than just a manual memcpy
        unsafe {
            let v = buffer.as_mut_vec();
            std::ptr::copy(ptr, v.as_mut_ptr(), len);
            v.set_len(len);
        }
    }

    buffer
}

#[cfg(test)]
mod tests {
    use crate::keyword::KeywordKind;

    use super::*;

    #[test]
    fn test_tokenizer_toradora() {
        let s = "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD]";
        let keywords = KeywordConfig::default();
        let tokenizer = Tokenizer::new(s, &keywords);
        #[rustfmt::skip]
        let expected = vec![
            Token::open_bracket("["),
            Token::text("TaigaSubs", TokenKind::Text, true),
            Token::close_bracket("]"),
            Token::delimiter("_", false),
            Token::text("Toradora!", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::open_bracket("("),
            Token::text("2008", TokenKind::Number, true),
            Token::close_bracket(")"),
            Token::delimiter("_", false),
            Token::delimiter("-", false),
            Token::delimiter("_", false),
            Token::text("01v2", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::delimiter("-", false),
            Token::delimiter("_", false),
            Token::text("Tiger", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::text("and", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::text("Dragon", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::open_bracket("["),
            Token::text("1280x720", TokenKind::Text, true),
            Token::delimiter("_", true),
            Token::from_keyword("H.264",Keyword::new(KeywordKind::VideoCodec), true),
            Token::delimiter("_", true),
            Token::from_keyword("FLAC", Keyword::new(KeywordKind::AudioCodec), true),
            Token::close_bracket("]"),
            Token::open_bracket("["),
            Token::text("1234ABCD", TokenKind::Text, true),
            Token::close_bracket("]"),
        ];
        for (original, expected) in tokenizer.into_iter().zip(expected.iter()) {
            assert_eq!(original.kind, expected.kind);
            assert_eq!(original.value, expected.value);
            assert_eq!(original.is_enclosed, expected.is_enclosed);
        }
    }

    #[test]
    fn test_tokenizer_evangelion() {
        let s = "Evangelion_1.11_You_Are_(Not)_Alone_(2009)_[1080p,BluRay,x264,DTS-ES]_-_THORA.mkv";
        let keywords = KeywordConfig::default();
        let tokenizer = Tokenizer::new(s, &keywords);
        let expected = vec![
            Token::text("Evangelion", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::text("1.11", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::text("You", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::text("Are", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::open_bracket("("),
            Token::text("Not", TokenKind::Text, true),
            Token::close_bracket(")"),
            Token::delimiter("_", false),
            Token::text("Alone", TokenKind::Text, false),
            Token::delimiter("_", false),
            Token::open_bracket("("),
            Token::text("2009", TokenKind::Number, true),
            Token::close_bracket(")"),
            Token::delimiter("_", false),
            Token::open_bracket("["),
            Token::from_keyword("1080p", Keyword::new(KeywordKind::VideoResolution), true),
            Token::delimiter(",", true),
            Token::from_keyword("BluRay", Keyword::new(KeywordKind::Source), true),
            Token::delimiter(",", true),
            Token::from_keyword("x264", Keyword::new(KeywordKind::VideoCodec), true),
            Token::delimiter(",", true),
            Token::from_keyword("DTS-ES", Keyword::new(KeywordKind::AudioChannels), true),
            Token::close_bracket("]"),
            Token::delimiter("_", false),
            Token::delimiter("-", false),
            Token::delimiter("_", false),
            Token::from_keyword("THORA", Keyword::new(KeywordKind::ReleaseGroup), false),
            Token::delimiter(".", false),
            Token::from_keyword("mkv", Keyword::new(KeywordKind::FileExtension), false),
        ];
        let actual = tokenizer.tokens();
        for (original, expected) in actual.iter().zip(expected.iter()) {
            assert_eq!(original.kind, expected.kind);
            assert_eq!(original.value, expected.value);
            assert_eq!(original.is_enclosed, expected.is_enclosed);
        }
    }
}
