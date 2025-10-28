use voxell_rng::slice_methods::{MultiSelectorImmutOverlap, select_random};

use crate::types::Token;

pub fn source_generator(tokens: usize) -> String {
    let mut rng = voxell_rng::rng::XorShift128::default();
    select_random(MultiSelectorImmutOverlap(tokens), Token::ALL, &mut rng)
        .into_iter()
        .copied()
        .fold(String::new(), |mut acc, tok| {
            acc.push_str(tok.source_repr());
            acc.push(' ');
            acc
        })
}
