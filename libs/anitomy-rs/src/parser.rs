use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::{combine_tokens, is_dash, opposite_bracket, Token},
    utils::*,
    Options,
};

fn is_token_isolated(tokens: &[Token<'_>], index: usize) -> bool {
    let Some(previous) = find_prev_token(tokens, Some(index), |t| t.is_not_delimiter()) else {
        return false;
    };

    if !tokens[previous].is_bracket() {
        return false;
    }

    let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
        return false;
    };
    tokens[next].is_bracket()
}

fn is_valid_episode_number(s: &str) -> bool {
    !s.is_empty() && s.len() <= 4 && s.bytes().all(|x| x.is_ascii_digit())
}

fn is_japanese_number(ch: char) -> bool {
    matches!(
        ch,
        '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百' | '千'
    )
}

fn is_valid_japanese_episode(s: &str) -> bool {
    if s.is_ascii() {
        is_valid_episode_number(s)
    } else {
        let codepoints = s.chars().count();
        codepoints > 0 && codepoints <= 4 && s.chars().all(is_japanese_number)
    }
}

fn parse_file_extension<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let [previous, last] = last_chunk_mut(tokens)?;
    let is_file_extension = last
        .keyword
        .is_some_and(|x| x.kind == KeywordKind::FileExtension);
    let is_dot = previous.is_delimiter() && previous.value == ".";
    if is_file_extension && is_dot {
        previous.mark_known();
        last.mark_known();
        Some(Element::new(ElementKind::FileExtension, last))
    } else {
        None
    }
}

fn keyword_kind_to_element_kind(keyword: KeywordKind) -> Option<ElementKind> {
    match keyword {
        KeywordKind::AudioChannels => Some(ElementKind::AudioTerm),
        KeywordKind::AudioCodec => Some(ElementKind::AudioTerm),
        KeywordKind::AudioLanguage => Some(ElementKind::AudioTerm),
        KeywordKind::DeviceCompatibility => Some(ElementKind::DeviceCompatibility),
        KeywordKind::EpisodeType => Some(ElementKind::Type),
        KeywordKind::Language => Some(ElementKind::Language),
        KeywordKind::Other => Some(ElementKind::Other),
        KeywordKind::ReleaseGroup => Some(ElementKind::ReleaseGroup),
        KeywordKind::ReleaseInformation => Some(ElementKind::ReleaseInformation),
        KeywordKind::ReleaseVersion => Some(ElementKind::ReleaseVersion),
        KeywordKind::Source => Some(ElementKind::Source),
        KeywordKind::Subtitles => Some(ElementKind::Subtitles),
        KeywordKind::Type => Some(ElementKind::Type),
        KeywordKind::VideoCodec => Some(ElementKind::VideoTerm),
        KeywordKind::VideoColorDepth => Some(ElementKind::VideoTerm),
        KeywordKind::VideoFormat => Some(ElementKind::VideoTerm),
        KeywordKind::VideoFrameRate => Some(ElementKind::VideoTerm),
        KeywordKind::VideoProfile => Some(ElementKind::VideoTerm),
        KeywordKind::VideoQuality => Some(ElementKind::VideoTerm),
        KeywordKind::VideoResolution => Some(ElementKind::VideoResolution),
        _ => None,
    }
}

fn parse_keywords<'a>(tokens: &mut [Token<'a>], options: &Options, results: &mut Vec<Element<'a>>) {
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        let Some(keyword) = token.keyword else {
            continue;
        };

        if keyword.kind == KeywordKind::ReleaseGroup && !options.parse_release_group() {
            continue;
        }
        if keyword.kind == KeywordKind::VideoResolution && !options.parse_video_resolution() {
            continue;
        }

        let Some(element_kind) = keyword_kind_to_element_kind(keyword.kind) else {
            continue;
        };

        if !keyword.is_ambiguous() || token.is_enclosed {
            token.mark_known();
        }

        let value = match keyword.kind {
            KeywordKind::ReleaseVersion => &token.value[1..], // v2 -> 2
            _ => token.value,
        };
        results.push(Element {
            kind: element_kind,
            value: Cow::Borrowed(value),
            position: token.position,
        });
    }
}

fn parse_file_checksum<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let (position, token) = tokens.iter_mut().enumerate().rev().find(|(_, t)| {
        t.is_free() && t.value.len() == 8 && t.value.bytes().all(|b| b.is_ascii_hexdigit())
    })?;

    token.mark_known();
    Some(Element {
        kind: ElementKind::FileChecksum,
        value: token.value.into(),
        position,
    })
}

// A video resolution can be in `1080p` or `1920x1080` format
fn is_video_resolution(input: &str) -> bool {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r#"^\d{3,4}(?:[ip]|[xX×]\d{3,4}[ip]?)$"#).unwrap())
        .is_match(input)
}

fn parse_video_resolution<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    let mut found = results
        .iter()
        .any(|e| e.kind == ElementKind::VideoResolution);
    for token in tokens
        .iter_mut()
        .filter(|t| t.is_free() && is_video_resolution(t.value))
    {
        token.mark_known();
        results.push(Element::new(ElementKind::VideoResolution, token));
        found = true;
    }

    if !found {
        // A special case for the 720 and 1080 string
        if let Some(token) = tokens
            .iter_mut()
            .find(|t| t.is_free() && t.is_number() && (t.value == "1080" || t.value == "720"))
        {
            results.push(Element::new(ElementKind::VideoResolution, token));
        }
    }
}

fn is_year(s: &str) -> bool {
    s.parse::<u16>()
        .ok()
        .is_some_and(|x| (1950..=2050).contains(&x))
}

fn is_month(s: &str) -> bool {
    s.parse::<u8>()
        .ok()
        .is_some_and(|month| (1..=12).contains(&month))
}

fn is_day(s: &str) -> bool {
    s.parse::<u8>().ok().is_some_and(|x| (1..=31).contains(&x))
}

fn parse_year<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    // Find a year enclosed by brackets
    if let Some(token) = tokens
        .windows(3)
        .enumerate()
        .find(|(_, x)| {
            x[0].is_open_bracket()
                && x[2].is_closed_bracket()
                && x[1].is_free()
                && x[1].is_number()
                && is_year(x[1].value)
        })
        .map(|(offset, _)| offset + 1)
        .and_then(|idx| tokens.get_mut(idx))
    {
        token.mark_known();
        return Some(Element::new(ElementKind::Year, token));
    }

    // Find a year number that is isolated
    for index in tokens
        .iter()
        .filter(|p| p.is_free() && p.is_number() && !p.is_enclosed && is_year(p.value))
        .map(|p| p.position)
    {
        // Check if it's isolated
        if is_token_isolated(tokens, index) {
            tokens[index].mark_known();
            return Some(Element::new(ElementKind::Year, &tokens[index]));
        }
    }

    None
}

fn parse_date<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    // Parses dates in YYYY.MM.DD format
    // Due to tokenisation this ends up being [YYYY.MM, ., DD]
    let mut iter = windows_mut(tokens);
    while let Some([year_month, delimiter, day]) = iter.next() {
        if !(delimiter.is_delimiter() && delimiter.value.starts_with(['.', '-']) && day.is_number())
        {
            continue;
        }

        let Some((year, month)) = year_month.value.split_once(['.', '-']) else {
            continue;
        };
        if !is_year(year) || !is_month(month) || !is_day(day.value) {
            continue;
        }

        year_month.mark_known();
        delimiter.mark_known();
        day.mark_known();
        return Some(Element {
            kind: ElementKind::Date,
            value: Cow::Owned(format!(
                "{}{}{}",
                year_month.value, delimiter.value, day.value
            )),
            position: year_month.position,
        });
    }

    None
}

// A lot of this tomfoolery is because of mutation
fn inner_parse_season<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let is_season_keyword =
        |token: &Token<'a>| token.keyword.is_some_and(|x| x.kind == KeywordKind::Season);

    let mut iter = windows_mut(tokens);
    while let Some([first, mid, last]) = iter.next() {
        // Check previous token for a number (e.g. 2nd Season)
        if is_season_keyword(last) && mid.is_delimiter() && first.is_free() {
            if let Some(number) = from_ordinal_number(first.value) {
                last.mark_known();
                mid.mark_known();
                first.mark_known();
                return Some(Element {
                    kind: ElementKind::Season,
                    value: number.into(),
                    position: first.position,
                });
            }
        }
        // Check next token for a number (e.g. Season 2, Season II, etc.)
        if is_season_keyword(first) && mid.is_delimiter() && last.is_free() {
            let value = if last.is_number() {
                last.value
            } else {
                match from_roman_number(last.value) {
                    Some(value) => value,
                    None => continue,
                }
            };
            last.mark_known();
            mid.mark_known();
            first.mark_known();
            return Some(Element {
                kind: ElementKind::Season,
                value: value.into(),
                position: last.position,
            });
        }
    }
    None
}

fn parse_season<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    if let Some(result) = inner_parse_season(tokens) {
        results.push(result);
        return;
    }

    // Check other patterns for seasons (e.g. S2, 第2期)
    // Note that sometimes the token can be concatenated (e.g. S01+S02)
    // This concatenation only goes 1 layer deep and in left to right order
    // So S01+S02+S03+S04 becomes [S01+S02, +, S03+S04]
    for token in tokens.iter_mut().filter(|x| x.is_free()) {
        for value in token.value.split(['.', '-', '&', '+', '~']) {
            // S\d{1,2} pattern
            if let Some(suffix) = value.strip_prefix(['S', 's']) {
                if (1..=2).contains(&suffix.len()) && suffix.bytes().all(|x| x.is_ascii_digit()) {
                    token.mark_known();
                    results.push(Element {
                        kind: ElementKind::Season,
                        value: suffix.into(),
                        position: token.position,
                    });
                }
            }
            // 第2期 pattern
            if let Some(prefix) = value.strip_suffix('期') {
                let prefix = prefix.strip_prefix('第').unwrap_or(prefix);
                if (1..=2).contains(&prefix.len()) && prefix.bytes().all(|x| x.is_ascii_digit()) {
                    token.mark_known();
                    results.push(Element {
                        kind: ElementKind::Season,
                        value: prefix.into(),
                        position: token.position,
                    });
                }
            }
        }
    }
}

fn parse_volume<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    // Some files have multiple volume specifiers in the name
    // The index tomfoolery is again because of mutability.
    for index in 0..tokens.len() {
        if !tokens[index]
            .keyword
            .is_some_and(|k| k.kind == KeywordKind::Volume)
        {
            continue;
        }

        let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
            continue;
        };
        if !tokens[next].is_free() {
            continue;
        }

        if parse_multi_episode_range(tokens, next, results, ElementKind::Volume) {
            tokens[index].mark_known();
            tokens[next].mark_known();
            continue;
        }

        let Some((prefix, suffix)) = parse_single_episode(tokens[next].value) else {
            continue;
        };
        results.push(Element {
            kind: ElementKind::Volume,
            value: prefix.into(),
            position: index,
        });
        if !suffix.is_empty() {
            results.push(Element {
                kind: ElementKind::ReleaseVersion,
                value: suffix.into(),
                position: index,
            })
        }
        tokens[index].mark_known();
        tokens[next].mark_known();
    }
    // let (volume, token) = find_pair_mut(
    //     tokens,
    //     |t| t.keyword.is_some_and(|k| k.kind == KeywordKind::Volume),
    //     |t| t.is_not_delimiter(),
    // )?;
    // let (prefix, suffix) = parse_single_episode(token.value)?;
    // if token.is_free() {
    //     token.mark_known();
    //     volume.mark_known();
    //     let first = Element {
    //         kind: ElementKind::Volume,
    //         value: prefix.into(),
    //         position: token.position,
    //     };
    //     let second = if !suffix.is_empty() {
    //         Some(Element {
    //             kind: ElementKind::ReleaseVersion,
    //             value: suffix.into(),
    //             position: token.position,
    //         })
    //     } else {
    //         None
    //     };
    //     Some((first, second))
    // } else {
    //     None
    // }
}

fn episode_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"^(?:E|[Ee][Pp]|Eps)(\d{1,4}(?:\.5)?)(?:[vV](\d))?$"#).unwrap()
    })
}

fn season_and_episode_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"^S?(\d{1,2})(?:-S?(\d{1,2}))?(?:x|[ ._-x]?EP?)(\d{1,4})(?:-(?:EP?)?(\d{1,4}))?(?:[vV](\d))?$"#).unwrap())
}

fn number_sign_episode_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"[#＃](\d{1,4})(?:[-~&+](\d{1,4}))?(?:[vV](\d))?"#).unwrap())
}

fn parse_number_in_number_episode<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    // Number comes before another number (e.g. `8 & 10`, `01 of 24`)
    // Once again, a lot of this tom foolery is because of mutability
    for index in 0..tokens.len() {
        {
            let token = &tokens[index];
            if !(token.is_free() && token.is_number()) {
                continue;
            }
        }
        // Skip delimiters but not &
        let Some(middle) = find_next_token(tokens, index, true, |t| {
            t.is_not_delimiter() || t.value == "&"
        }) else {
            continue;
        };
        if tokens[middle].value != "&" && tokens[middle].value != "of" {
            continue;
        }
        if let Some(other_number) = tokens[middle..]
            .iter_mut()
            .skip(1)
            .find(|t| t.is_not_delimiter())
        {
            if !other_number.is_number() {
                continue;
            }
            other_number.mark_known();
            tokens[middle].mark_known();
            tokens[index].mark_known();
            return Some(Element::new(ElementKind::Episode, &tokens[index]));
        }
    }
    None
}

/// Parses numbers in format \d{1,4}(?:[vV]\d)?
///
/// If the second element is not there then an empty string is returned.
fn parse_single_episode(s: &str) -> Option<(&str, &str)> {
    match s.split_once(['v', 'V']) {
        Some((prefix, suffix)) => {
            if is_valid_episode_number(prefix)
                && suffix.len() == 1
                && suffix.as_bytes()[0].is_ascii_digit()
            {
                Some((prefix, suffix))
            } else {
                None
            }
        }
        None if is_valid_episode_number(s) => Some((s, "")),
        _ => None,
    }
}

fn parse_multi_episode_range<'a>(
    tokens: &mut [Token<'a>],
    index: usize,
    results: &mut Vec<Element<'a>>,
    kind: ElementKind,
) -> bool {
    if let Some((first, last)) = tokens[index].value.split_once(['-', '~', '&', '+']) {
        let token = &mut tokens[index];
        if let Some(((lower, low_version), (upper, up_version))) =
            parse_single_episode(first).zip(parse_single_episode(last))
        {
            match lower.parse::<u16>().ok().zip(upper.parse::<u16>().ok()) {
                // Avoid matching 000-1, 5-2, etc.
                Some((x, y)) if x < y => {
                    results.push(Element {
                        kind,
                        value: lower.into(),
                        position: token.position,
                    });
                    token.mark_known();
                    if !low_version.is_empty() {
                        results.push(Element {
                            kind: ElementKind::ReleaseVersion,
                            value: low_version.into(),
                            position: token.position,
                        });
                    }
                    results.push(Element {
                        kind,
                        value: upper.into(),
                        position: token.position,
                    });
                    if !up_version.is_empty() {
                        results.push(Element {
                            kind: ElementKind::ReleaseVersion,
                            value: up_version.into(),
                            position: token.position,
                        });
                    }
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

fn parse_episode<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>, kind: ElementKind) {
    let is_regular_episode = kind == ElementKind::Episode;
    // While it's tempting to combine these for loops into one, it's imperative that
    // these are separated out since this allows the set patterns to go through
    // *the entire string* before choosing a less favourable outcome

    // For example: Nier:Automata Ver1.1a - 01
    // In here, the 01 is more important than the 1a, despite coming in later.

    // Parse episode prefixes (e.g. E1, EP1, Episode 1, etc)
    for index in 0..tokens.len() {
        if !tokens[index].is_free() {
            continue;
        }

        let is_keyword = tokens[index]
            .keyword
            .is_some_and(|k| k.kind == KeywordKind::Episode);
        if is_keyword {
            if let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) {
                if tokens[next].is_free() && tokens[next].is_mostly_numbers() {
                    if parse_multi_episode_range(tokens, next, results, kind) {
                        tokens[index].mark_known();
                        return;
                    }

                    if tokens[next].is_number() {
                        tokens[index].mark_known();
                        tokens[next].mark_known();
                        results.push(Element::new(kind, &tokens[next]));
                        return;
                    }
                }
            }
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(m) = episode_prefix_regex().captures(token.value) {
            results.push(Element {
                kind,
                value: m.get(1).unwrap().as_str().into(),
                position: token.position,
            });
            token.mark_known();
            if let Some(inner) = m.get(2) {
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            return;
        }
    }

    if let Some(number) = parse_number_in_number_episode(tokens) {
        results.push(number);
        return;
    }

    // Single episode (e.g. 01v2)
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some((prefix, suffix)) = parse_single_episode(token.value) {
            if !suffix.is_empty() {
                token.mark_known();
                results.push(Element {
                    kind,
                    value: prefix.into(),
                    position: token.position,
                });
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: suffix.into(),
                    position: token.position,
                });
                return;
            }
        }
    }

    // Multi-episode, (e.g. 01-02, 03-05v2)
    for index in 0..tokens.len() {
        if tokens[index].is_free() && parse_multi_episode_range(tokens, index, results, kind) {
            return;
        }
    }

    // Season and episode (e.g. `2x01`, `S01E03`, `S01-02xE001-150`)
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(captures) = season_and_episode_regex().captures(token.value) {
            if captures[1].parse::<u8>().unwrap_or_default() != 0 {
                results.push(Element {
                    kind: ElementKind::Season,
                    value: captures.get(1).unwrap().as_str().into(),
                    position: token.position,
                });
                token.mark_known();
                if let Some(inner) = captures.get(2) {
                    results.push(Element {
                        kind: ElementKind::Season,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }

                results.push(Element {
                    kind,
                    value: captures.get(3).unwrap().as_str().into(),
                    position: token.position,
                });
                if let Some(inner) = captures.get(4) {
                    results.push(Element {
                        kind,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }
                if let Some(inner) = captures.get(5) {
                    results.push(Element {
                        kind: ElementKind::ReleaseVersion,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }
                return;
            }
        }
    }

    // Type and episode (e.g. `ED1`, `OP4a`, `OVA2`)
    if let Some((_, token)) = find_pair_mut(
        tokens,
        |t| {
            t.keyword.is_some_and(|x| x.kind == KeywordKind::Type)
                && !t.value.eq_ignore_ascii_case("movie")
        },
        |t| t.is_not_delimiter(),
    ) {
        if token.is_free() && token.is_number() {
            token.mark_known();
            results.push(Element::new(kind, token));
            return;
        }
    }

    // Number sign, e.g. #01 or #02-03v2
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(captures) = number_sign_episode_regex().captures(token.value) {
            token.mark_known();
            results.push(Element {
                kind,
                value: captures.get(1).unwrap().as_str().into(),
                position: token.position,
            });
            if let Some(inner) = captures.get(2) {
                results.push(Element {
                    kind,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            if let Some(inner) = captures.get(3) {
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            return;
        }
    }
    // Japanese counter (e.g. `第01話`)
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(prefix) = token.value.strip_suffix('話') {
            let prefix = prefix.strip_prefix('第').unwrap_or(prefix);
            if is_valid_japanese_episode(prefix) {
                token.mark_known();
                results.push(Element {
                    kind,
                    value: prefix.into(),
                    position: token.position,
                });
                return;
            }
        }
    }

    // Equivalent numbers (e.g. `01 (176)`, `29 (04)`)
    if is_regular_episode {
        for index in tokens
            .iter()
            .filter(|t| t.is_free() && t.is_number())
            .map(|t| t.position)
        {
            if is_token_isolated(tokens, index) || !is_valid_episode_number(tokens[index].value) {
                continue;
            }

            // At this point we're here:
            // `01 (176)`
            //  ^^

            // Find first enclosed non-delimiter token
            // e.g. `01 (176)`
            //           ^^^
            let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
                continue;
            };
            if !tokens[next].is_bracket() {
                continue;
            }

            let Some(next) = find_next_token(tokens, next, true, |t| t.is_not_delimiter()) else {
                continue;
            };
            // Validate that the actual token is valid for insertion
            // i.e. a free episode number that is isolated
            let is_valid = {
                let token = &tokens[next];
                token.is_free()
                    && token.is_number()
                    && is_valid_episode_number(token.value)
                    && is_token_isolated(tokens, next)
            };
            if !is_valid {
                continue;
            }

            // At this point `index` represents the first number and `next` is the second number
            let Some((first, second)) = tokens[index]
                .value
                .parse::<u16>()
                .ok()
                .zip(tokens[next].value.parse::<u16>().ok())
            else {
                continue;
            };

            let (a, b) = if first > second {
                (ElementKind::EpisodeAlt, ElementKind::Episode)
            } else {
                (ElementKind::Episode, ElementKind::EpisodeAlt)
            };

            tokens[next].mark_known();
            tokens[index].mark_known();
            results.push(Element::new(b, &tokens[next]));
            results.push(Element::new(a, &tokens[index]));
            return;
        }
    }

    // Separated number (e.g. ` - 08`)
    // Have to do manual loop due to mutability
    for index in 0..tokens.len() {
        let is_valid = {
            let token = &tokens[index];
            token.is_delimiter() && token.value.chars().next().is_some_and(is_dash)
        };
        if !is_valid {
            continue;
        }

        if let Some(token) = tokens.iter_mut().skip(index).find(|x| x.is_not_delimiter()) {
            if token.is_number() && token.is_free() {
                token.mark_known();
                results.push(Element::new(kind, token));
                tokens[index].mark_known();
                return;
            }
        }
    }

    // Fractional episode (e.g. `07.5`)
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some((first, second)) = token.value.split_once('.') {
            // We don't allow any fractional part other than `.5`, because there are cases
            // where such a number is a part of the title (e.g. `Evangelion: 1.11`,
            // `Tokyo Magnitude 8.0`) or a keyword (e.g. `5.1`).
            if second == "5" && is_valid_episode_number(first) {
                token.mark_known();
                results.push(Element::new(kind, token));
                return;
            }
        }
    }

    // Isolated number (e.g. [12], (2006), etc.)
    {
        let mut iter = windows_mut(tokens);
        while let Some([first, middle, last]) = iter.next() {
            if first.is_open_bracket()
                && last.is_closed_bracket()
                && middle.is_free()
                && middle.is_number()
            {
                results.push(Element::new(kind, middle));
                middle.mark_known();
                return;
            }
        }
    }

    // Partial episode (e.g. `4a`, `111C`)
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(prefix) = token.value.strip_suffix(['A', 'B', 'C', 'a', 'b', 'c']) {
            if is_valid_episode_number(prefix) {
                token.mark_known();
                results.push(Element::new(kind, token));
                return;
            }
        }
    }

    // Last number
    // Get all the free number tokens available:
    // is_enclosed: At this point an enclosed number is not the episode number
    for index in (0..tokens.len())
        .skip(1)
        .filter(|&i| tokens[i].is_free() && tokens[i].is_number() && !tokens[i].is_enclosed)
    {
        // Ignore if it's the first non-enclosed and non-delimiter token
        if tokens[..index]
            .iter()
            .all(|t| t.is_enclosed || t.is_delimiter())
        {
            continue;
        }

        // Ignore if the previous token is "movie" or "part"
        let previous = find_prev_token(tokens, Some(index), |t| t.is_not_delimiter());
        if let Some(idx) = previous {
            let prev = &tokens[idx];
            if prev.is_free()
                && (prev.value.eq_ignore_ascii_case("movie")
                    || prev.value.eq_ignore_ascii_case("part"))
            {
                continue;
            }
        }

        // At this point this is probably the valid number
        let token = &mut tokens[index];
        token.mark_known();
        results.push(Element::new(kind, token));
        break;
    }
}

fn find_prev_token<F>(
    tokens: &[Token<'_>],
    position: Option<usize>,
    mut predicate: F,
) -> Option<usize>
where
    F: FnMut(&Token<'_>) -> bool,
{
    let index = position.unwrap_or(tokens.len());
    tokens[..index]
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, t)| predicate(t).then_some(idx))
}

fn find_next_token<F>(tokens: &[Token<'_>], index: usize, skip: bool, predicate: F) -> Option<usize>
where
    F: FnMut(&Token<'_>) -> bool,
{
    let offset = if skip { index + 1 } else { index };
    tokens[offset..]
        .iter()
        .position(predicate)
        .map(|idx| idx + offset)
}

fn find_title<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    // Find the first free unenclosed range
    // e.g. `[Group] Title - Episode [Info]`
    //               ^-------^
    let mut first = tokens.iter().position(|t| t.is_free() && !t.is_enclosed);
    let mut last =
        first.and_then(|index| find_next_token(tokens, index, true, |t| t.is_identified()));
    // Fall back to the second enclosed range (assuming the first one is for release group)
    // e.g. `[Group][Title][Info]`
    //               ^----^
    if first.is_none() {
        // Get the opposite bracket that was matched with the open bracket
        // This is mainly for cases where a parentheses is within the title,
        // e.g. [Evangelion 3.0 You Can (Not) Redo]
        if let Some((opposite, index)) =
            find_pair_mut(tokens, |t| t.is_closed_bracket(), |t| t.is_open_bracket()).and_then(
                |(_, open)| {
                    open.value
                        .chars()
                        .next()
                        .and_then(opposite_bracket)
                        .zip(Some(open.position))
                },
            )
        {
            first = find_next_token(tokens, index, false, |t| t.is_free());
            last = first.and_then(|idx| {
                find_next_token(tokens, idx, true, |t| {
                    t.is_bracket() && t.value.starts_with(opposite)
                })
            });
        }
    }

    // Allow filenames without a title
    let index = first?;

    // Prevent titles with mismatched brackets
    // e.g. `Title (`      -> `Title `
    // e.g. `Title [Info ` -> `Title `
    // Get the count + last index of the bracket
    let slice = &tokens[index..last.unwrap_or(tokens.len())];
    let (count, last_index) = slice
        .iter()
        .enumerate()
        .filter_map(|(index, token)| token.is_open_bracket().then_some(index))
        .fold((0, 0), |acc, x| (acc.0 + 1, x));
    if count != 0 {
        let closed_count = slice.iter().filter(|t| t.is_closed_bracket()).count();
        if closed_count != count {
            last = Some(last_index + index);
        }
    }

    // Prevent titles ending with brackets (except parentheses)
    // e.g. `Title [Group]` -> `Title `
    // e.g. `Title (TV)`    -> *no change*
    if let Some(idx) = find_prev_token(tokens, last, |t| t.is_not_delimiter()) {
        let token = &tokens[idx];
        if token.is_closed_bracket() && token.value != ")" {
            if let Some(new_last) = find_prev_token(tokens, Some(idx), |t| t.is_open_bracket()) {
                last = Some(new_last)
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

fn parse_title<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_title(tokens)?;
    let value = combine_tokens(range, crate::tokenizer::KeepDelimiters::No);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::Title,
            value: value.into(),
            position,
        })
    }
}

fn get_last_index_for_release_group(tokens: &[Token<'_>], first: Option<usize>) -> Option<usize> {
    let other_bracket = find_prev_token(tokens, first, |t| !t.is_enclosed && t.is_open_bracket())
        .and_then(|i| tokens[i].value.chars().next().and_then(opposite_bracket));

    first.and_then(|index| match other_bracket {
        Some(bracket) => find_next_token(tokens, index, true, |t| {
            t.is_closed_bracket() && t.value.starts_with(bracket)
        }),
        None => find_next_token(tokens, index, true, |t| t.is_closed_bracket()),
    })
}

fn find_release_group<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    // Find the first enclosed unidentified range
    // e.g. `[Group] Title - Episode [Info]`
    //        ^----^
    let mut first = tokens
        .iter()
        .position(|t| t.is_enclosed && !t.is_identified());

    let mut last = get_last_index_for_release_group(tokens, first);

    // Skip brackets if they have taken tokens and move on to the next pair of brackets instead
    while let Some((start, end)) = first.zip(last) {
        if start > tokens.len() || end > tokens.len() {
            break;
        }

        if tokens[start..end].iter().all(|t| !t.is_identified()) {
            break;
        }

        first = find_next_token(tokens, end, true, |t| t.is_enclosed && t.is_free());
        last = get_last_index_for_release_group(tokens, first);
    }

    // Fall back to the last token before file extension
    // e.g. `Title.Episode.Info-Group.mkv`
    //                          ^----^
    if first.is_none() {
        if let Some(idx) = find_prev_token(tokens, Some(tokens.len()), |t| {
            t.is_free() && t.is_not_delimiter()
        }) {
            let token = &tokens[idx];
            if token.is_free()
                && idx != 0
                && tokens
                    .get(idx - 1)
                    .is_some_and(|t| t.is_delimiter() && t.value == "-")
            {
                first = Some(idx);
                last = Some(idx + 1);
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

fn parse_release_group<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_release_group(tokens)?;
    let value = combine_tokens(range, crate::tokenizer::KeepDelimiters::Yes);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::ReleaseGroup,
            value: value.into(),
            position,
        })
    }
}

fn find_episode_title<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    // Find the first free unenclosed range
    // e.g. `[Group] Title - Episode - Episode Title [Info]`
    //                                 ^-------------^
    let mut first = tokens.iter().position(|t| t.is_free() && !t.is_enclosed);
    let mut last = first.and_then(|index| {
        find_next_token(tokens, index, false, |t| {
            t.is_open_bracket() || t.is_identified()
        })
    });

    // Fall back to the first free range in corner brackets
    // e.g. `[Group] Title - Episode 「Episode Title」`
    //                                ^------------^
    if first.is_none() {
        first = tokens
            .iter()
            .position(|t| t.is_open_bracket() && t.value == "「")
            .map(|idx| idx + 1);
        last = first.and_then(|index| {
            find_next_token(tokens, index, false, |t| {
                t.is_closed_bracket() && t.value == "」"
            })
        });
        match last {
            None => return None,
            Some(last) => {
                if tokens[first.unwrap_or_default()..last]
                    .iter()
                    .any(|t| t.is_identified())
                {
                    return None;
                }
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

fn parse_episode_title<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_episode_title(tokens)?;
    let value = combine_tokens(range, crate::tokenizer::KeepDelimiters::No);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::EpisodeTitle,
            value: value.into(),
            position,
        })
    }
}

pub(crate) fn parse_with_options(mut tokens: Vec<Token<'_>>, options: Options) -> Vec<Element<'_>> {
    let mut results = Vec::new();
    if options.parse_file_extension() {
        if let Some(el) = parse_file_extension(&mut tokens) {
            results.push(el);
        }
    }

    parse_keywords(&mut tokens, &options, &mut results);

    if options.parse_file_checksum() {
        if let Some(el) = parse_file_checksum(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_video_resolution() {
        parse_video_resolution(&mut tokens, &mut results);
    }

    if options.parse_date() {
        if let Some(el) = parse_date(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_year() {
        if let Some(el) = parse_year(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_season() {
        parse_season(&mut tokens, &mut results);
    }

    if options.parse_episode() {
        parse_volume(&mut tokens, &mut results);
        parse_episode(&mut tokens, &mut results, ElementKind::Episode);
    }

    if options.parse_title() {
        if let Some(title) = parse_title(&mut tokens) {
            results.push(title);
        }
    }

    if options.parse_release_group() && !results.iter().any(|e| e.kind == ElementKind::ReleaseGroup)
    {
        if let Some(group) = parse_release_group(&mut tokens) {
            results.push(group);
        }
    }

    let has_episode = results.iter().any(|e| e.kind == ElementKind::Episode);

    if has_episode {
        if options.parse_episode_title() {
            if let Some(title) = parse_episode_title(&mut tokens) {
                results.push(title);
            }
        }

        if options.parse_episode() {
            parse_episode(&mut tokens, &mut results, ElementKind::EpisodeAlt)
        }
    }

    results.sort_by_key(|e| e.position);
    results
}
