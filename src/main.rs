use std::{
    arch::x86_64,
    fmt::Display,
    fs,
    mem::{ManuallyDrop, MaybeUninit},
};

use voxell_timer::{power_toys::ScopedTimer, time_fn};

use crate::{
    lexer::{Lexer, LexerError, LexerResult},
    source_code::SourceCode,
};

pub mod lexer;
pub mod source_code;
pub mod types;

#[derive(Clone, PartialEq, Eq)]
struct TimerThing {
    i: i32,
    s: String,
}

impl TimerThing {
    pub fn new(i: i32, s: String) -> Self {
        Self { i, s }
    }
}

impl Display for TimerThing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.i, self.s)
    }
}

fn main() {
    let folder = fs::read_dir("progs").unwrap();
    let mut pairs = vec![];
    for entry in folder {
        let entry = entry.unwrap();
        let path = entry.path();

        let mut dat = fs::read_to_string(&path).unwrap().repeat(150000);
        pairs.push((dat, path));
    }

    let mut sum = 0;
    let mut total_source = 0;
    let mut st = ScopedTimer::new(TimerThing::new(0, "main".to_string()));

    let mut f1 = st.fork(TimerThing::new(1, "main".to_string()));
    for (i, (source, path)) in pairs.into_iter().enumerate() {
        let mut f2 = f1.fork(TimerThing::new(
            i as i32,
            format!("file {}, {:.1}MB", path.to_string_lossy(), source.len() as f64 / 1000000.0),
        ));
        let mut lexer = Lexer::new(SourceCode::new(&source));
        let mut val;
        'tokens: loop {
            val = lexer.lex_single_token();
            if val == Err(LexerError::Eof) {
                total_source += source.len();
                break 'tokens;
            }
            match val {
                Ok(_t) => {
                    sum += 1;
                }
                Err(e) => {
                    let (line, col) = lexer.get_line_column();
                    let maybe_lit: LexerResult<&[u8]> = lexer.extract_literal();
                    let start = lexer.start();
                    let index = lexer.index();
                    eprintln!(
                        "lexer error in file {:?} at {}:{} (index {}-{}): {:?}, maybe_lit: {:?}",
                        path, line, col, start, index, e, maybe_lit
                    );
                    total_source += lexer.start();
                    break 'tokens;
                }
            }
        }
        f2.join();
    }
    f1.join();

    // println!(
    //     "Lexed {:.2}M bytes ({:.2}M tokens) in {:?}\n{:.2} MB/s, {:.2} tokens/s",
    //     total_source as f64 / 1000000.0,
    //     sum as f64 / 1000000.0,
    //     dur,
    //     (total_source as f64 / elapsed_secs) / 1_000_000.0,
    //     (150_000_000.0 / elapsed_secs),
    // );

    println!("{}", st.join_and_finish_pretty());
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
