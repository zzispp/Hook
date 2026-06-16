#[derive(Clone, Copy)]
pub(super) enum Provider {
    OpenAi,
    Gemini,
    Claude,
}

#[derive(Clone, Copy)]
struct Multipliers {
    word: f64,
    number: f64,
    cjk: f64,
    symbol: f64,
    math_symbol: f64,
    url_delim: f64,
    at_sign: f64,
    emoji: f64,
    newline: f64,
    space: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WordType {
    None,
    Latin,
    Number,
}

pub(in crate::llm_proxy::proxy) fn estimate_text_tokens(model: &str, text: &str) -> i64 {
    if text.is_empty() {
        return 0;
    }
    estimate_token(provider_for_model(model), text)
}

fn provider_for_model(model: &str) -> Provider {
    let lower = model.to_ascii_lowercase();
    if lower.contains("gemini") {
        return Provider::Gemini;
    }
    if lower.contains("claude") {
        return Provider::Claude;
    }
    Provider::OpenAi
}

fn estimate_token(provider: Provider, text: &str) -> i64 {
    let multipliers = multipliers(provider);
    let mut count = 0.0;
    let mut current = WordType::None;
    for char in text.chars() {
        if char.is_whitespace() {
            current = WordType::None;
            count += if matches!(char, '\n' | '\t') {
                multipliers.newline
            } else {
                multipliers.space
            };
            continue;
        }
        if is_cjk(char) {
            current = WordType::None;
            count += multipliers.cjk;
            continue;
        }
        if is_emoji(char) {
            current = WordType::None;
            count += multipliers.emoji;
            continue;
        }
        if char.is_alphanumeric() {
            let next = if char.is_numeric() { WordType::Number } else { WordType::Latin };
            if current == WordType::None || current != next {
                count += if next == WordType::Number { multipliers.number } else { multipliers.word };
                current = next;
            }
            continue;
        }
        current = WordType::None;
        count += symbol_weight(char, multipliers);
    }
    count.ceil() as i64
}

fn symbol_weight(char: char, multipliers: Multipliers) -> f64 {
    if is_math_symbol(char) {
        return multipliers.math_symbol;
    }
    if char == '@' {
        return multipliers.at_sign;
    }
    if is_url_delim(char) {
        return multipliers.url_delim;
    }
    multipliers.symbol
}

fn multipliers(provider: Provider) -> Multipliers {
    match provider {
        Provider::Gemini => Multipliers {
            word: 1.15,
            number: 2.8,
            cjk: 0.68,
            symbol: 0.38,
            math_symbol: 1.05,
            url_delim: 1.2,
            at_sign: 2.5,
            emoji: 1.08,
            newline: 1.15,
            space: 0.2,
        },
        Provider::Claude => Multipliers {
            word: 1.13,
            number: 1.63,
            cjk: 1.21,
            symbol: 0.4,
            math_symbol: 4.52,
            url_delim: 1.26,
            at_sign: 2.82,
            emoji: 2.6,
            newline: 0.89,
            space: 0.39,
        },
        Provider::OpenAi => Multipliers {
            word: 1.02,
            number: 1.55,
            cjk: 0.85,
            symbol: 0.4,
            math_symbol: 2.68,
            url_delim: 1.0,
            at_sign: 2.0,
            emoji: 2.12,
            newline: 0.5,
            space: 0.42,
        },
    }
}

fn is_cjk(char: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&char) || ('\u{3040}'..='\u{30ff}').contains(&char) || ('\u{ac00}'..='\u{d7a3}').contains(&char)
}

fn is_emoji(char: char) -> bool {
    ('\u{1f300}'..='\u{1f9ff}').contains(&char)
        || ('\u{2600}'..='\u{26ff}').contains(&char)
        || ('\u{2700}'..='\u{27bf}').contains(&char)
        || ('\u{1fa00}'..='\u{1faff}').contains(&char)
}

fn is_math_symbol(char: char) -> bool {
    const MATH_SYMBOLS: &str = "∑∫∂√∞≤≥≠≈±×÷∈∉∋∌⊂⊃⊆⊇∪∩∧∨¬∀∃∄∅∆∇∝∟∠∡∢°′″‴⁺⁻⁼⁽⁾ⁿ₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎²³¹⁴⁵⁶⁷⁸⁹⁰";
    MATH_SYMBOLS.contains(char)
        || ('\u{2200}'..='\u{22ff}').contains(&char)
        || ('\u{2a00}'..='\u{2aff}').contains(&char)
        || ('\u{1d400}'..='\u{1d7ff}').contains(&char)
}

fn is_url_delim(char: char) -> bool {
    matches!(char, '/' | ':' | '?' | '&' | '=' | ';' | '#' | '%')
}

#[cfg(test)]
mod tests {
    use super::estimate_text_tokens;

    #[test]
    fn counts_cjk_and_latin_text() {
        assert!(estimate_text_tokens("gpt-5.5", "hello 世界") > 0);
    }
}
