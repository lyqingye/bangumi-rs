pub(crate) mod element;
pub(crate) mod keyword;
pub(crate) mod options;
pub(crate) mod parser;
pub(crate) mod tokenizer;
pub(crate) mod utils;

pub use element::{Element, ElementKind, ElementObject, OwnedElementObject};
pub use keyword::KeywordConfig;
pub use options::Options;

/// Parses a string into its element components with the given options.
///
/// If no options are meant to be passed, use [`parse`] instead which
/// will use the default options.
///
/// For best results, the string should be in composed form (NFC/NFKC)
/// for the tokenizer to work properly.
pub fn parse_with_options_and_config<'a>(
    input: &'a str,
    options: Options,
    config: &'a KeywordConfig,
) -> Vec<Element<'a>> {
    let tokens = tokenizer::Tokenizer::new(input, config).tokens();
    parser::parse_with_options(tokens, options)
}
