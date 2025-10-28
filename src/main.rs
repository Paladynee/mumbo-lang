use voxell_timer::time_fn;

use crate::{
    lexer::{Lexer, LexerError},
    source_code::SourceCode,
};

pub mod lexer;
pub mod source_code;
pub mod test_util;
pub mod types;

fn main() {
    println!("gen source");
    let mut source = test_util::source_generator(150_000_000);
    println!("source: {}, {}", source.len(), source.chars().take(100).collect::<String>());
    println!("lex source");
    let mut lexer = Lexer::new(SourceCode::new(&source));
    let (sum, dur) = time_fn(|| {
        let mut sum = 0;
        loop {
            let next = lexer.lex_single_token();
            match next {
                Ok(tok) => {
                    let _ = std::hint::black_box(tok);
                    sum += 1;
                }
                Err(LexerError::Eof) => break,
                Err(e) => {
                    let (line, col) = lexer.get_line_column();
                    let index = lexer.index();
                    let start = lexer.start();
                    let maybelit: Result<_, _> = lexer.extract_literal();

                    eprintln!(
                        "lexer error at {}:{} (index {}, start {}), literal {:?}: {:?}",
                        line, col, index, start, maybelit, e
                    );
                }
            }
        }
        sum
    });

    let elapsed_secs = dur.as_secs_f64();

    println!(
        "Lexed {} bytes ({} tokens) in {:?}\n{} MB/s, {} tokens/s",
        source.len(),
        sum,
        dur,
        (source.len() as f64 / elapsed_secs) / 1_000_000.0,
        (150_000_000.0 / elapsed_secs),
    );
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, LexerResult};
    use crate::source_code::SourceCode;
    use crate::types::Token;

    #[test]
    fn identifier_test() {
        let source = "
        // ! . = < >> : == let fn return runtime extern enum const compiletime cast mut anymut
        // static struct type union
        // uninit

        let my_custom_function_name1 = fn __secret() -> fn() {
            return runtime {
                extern fn bob() {
                    enum Thingamabob {
                        __variant1,
                        __variant2,
                    };

                    let variant: const u8 = compiletime { __variant1 cast u8 };
                    let v2: mut u8 = 0;
                    v2 = 1;
                    let v3: anymut static u8 = 0;
                }
            };
        };

        struct Lol {
            ty: type,
            un: myunion,
        };

        union myunion {
            num64: u64,
            num32: u32,
        };
        let name: literal = \"quit smoking\";
        let byte: u8 = '5';
        let mynum: mut literal = 1359135;
        let uninitthing: mut u64 = uninit;
        let cond: mut bool = false;
        let cond2: mut bool = true;
        if cond == true {
            mynum = mynum + 1;
        } else {
            mynum = mynum - 1;
        };
        let floatlt = 3.14159;
        ";

        let mut lexer = Lexer::new(SourceCode::new(&source));
        let mut val: LexerResult<Token> = Ok(Token::KwConst);
        loop {
            val = lexer.lex_single_token();
            if val == Err(crate::lexer::LexerError::Eof) {
                break;
            }

            match val {
                Ok(t) => {
                    if t.is_identifier_extractable() {
                        let ident = lexer.extract_literal().unwrap();
                        let str = str::from_utf8(ident).unwrap();
                        print!("{} ", str);
                    } else {
                        print!("{} ", t.source_repr());
                    }
                }
                Err(e) => {
                    panic!("lexer error: {:?}", e);
                }
            }
        }
        println!();
    }
}
