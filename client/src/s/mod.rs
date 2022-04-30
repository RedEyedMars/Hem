pub mod l;

use std::fs::File;
use std::io::{BufReader, Read, Write};
use wasm_bindgen::prelude::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[wasm_bindgen(module = "/src/s/levels.js")]
extern {
    #[wasm_bindgen(catch)]
    pub async fn fetch_level(filename: &str) -> Result<JsValue, JsValue>;
    pub fn post_level(filename: &str, body: String);
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Token {
    Quote(usize, usize), //start,end
    String(String),
    Coordinates(usize, usize, usize), // start comma end
    Coord(i32, i32),
    Varying(char),
    Ruminations(usize, Vec<usize>),
    Runes(Vec<Vec<RuneToken>>),
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum RuneToken {
    Action(char, u8, ConditionRuneToken),
    Manip(char, u8, ConditionRuneToken),
    None,
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ConditionRuneToken {
    Condition(char),
    None,
}
pub enum TokenizingState {
    Root,
    Quoting(usize),                //start
    Coordinating(usize, usize),    // start
    Ruminating(usize, Vec<usize>), // start, rune start/ends
}

pub fn read_file(filename: &str) -> Result<String, failure::Error> {
    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn extract_file_string(tokens: &Vec<Vec<Token>>) -> String {
    let mut content = String::from("");
    for line in tokens.iter() {
        for token in line.iter() {
            match token {
                Token::Quote(_, _) => {}
                Token::String(s) => {
                    content.push('\"');
                    content.push_str(s.as_str());
                    content.push('\"');
                }
                Token::Coordinates(_, _, _) => {}
                Token::Coord(x, y) => {
                    content.push('(');
                    content.push_str(x.to_string().as_str());
                    content.push(',');
                    content.push_str(y.to_string().as_str());
                    content.push(')');
                }
                Token::Varying(c) => {
                    content.push(*c);
                }
                Token::Ruminations(_, _) => {}
                Token::Runes(runes) => {
                    content.push('[');
                    for arrangement in runes.iter() {
                        if arrangement.len() == 0 {
                            continue;
                        }
                        for rune in arrangement.iter() {
                            match rune {
                                RuneToken::Action(c, t, _cond) => {
                                    content.push(*c);
                                    content.push_str(t.to_string().as_str());
                                }
                                RuneToken::Manip(c, t, _cond) => {
                                    content.push(*c);
                                    content.push_str(t.to_string().as_str());
                                }
                                RuneToken::None => {}
                            }
                        }
                        content.push(',');
                    }
                    content.push(']');
                }
            }
        }
        content.push('\n');
    }
    content
}

pub fn write_file(filename: &str, tokens: &Vec<Vec<Token>>) -> Result<(), failure::Error> {
    let mut file = File::create(filename)?;
    file.write_all(extract_file_string(tokens).into_bytes().as_slice())?;
    Ok(())
}

pub fn tokenize(line: &str) -> Vec<Token> {
    let mut r = Vec::with_capacity(4092);

    let mut state = TokenizingState::Root;
    for (i, l) in line.chars().enumerate() {
        match l {
            '\"' => match state {
                TokenizingState::Quoting(start) => {
                    r.push(Token::Quote(start, i));
                    state = TokenizingState::Root;
                }
                _ => {
                    state = TokenizingState::Quoting(i);
                }
            },
            '(' => match state {
                TokenizingState::Root => {
                    state = TokenizingState::Coordinating(i, 0usize);
                }
                TokenizingState::Quoting(_) => {}
                _ => println!("Unexpected quote at {}", i),
            },
            ',' => match state {
                TokenizingState::Coordinating(j, 0usize) => {
                    state = TokenizingState::Coordinating(j, i);
                }
                TokenizingState::Coordinating(_, _) => {}
                TokenizingState::Ruminating(_, ref mut v) => v.push(i),
                _ => {}
            },
            ')' => match state {
                TokenizingState::Coordinating(_, 0usize) => {
                    panic!("Only one coordinate, must have 2!");
                }
                TokenizingState::Coordinating(j, k) => {
                    r.push(Token::Coordinates(j, k, i));
                    state = TokenizingState::Root;
                }
                _ => {}
            },
            '[' => match state {
                TokenizingState::Root => {
                    state = TokenizingState::Ruminating(i, Vec::with_capacity(256))
                }
                _ => {}
            },
            ']' => match state {
                TokenizingState::Ruminating(start, mut v) => {
                    v.push(i);
                    r.push(Token::Ruminations(start, v));
                    state = TokenizingState::Root;
                }
                _ => {}
            },
            c if c as u8 > 32 => match state {
                TokenizingState::Root => r.push(Token::Varying(c)),
                _ => {}
            },
            _ => {}
        }
    }

    let mut result = Vec::with_capacity(4092);
    for t in r {
        match t {
            Token::Quote(start, end) => {
                result.push(Token::String(line[start + 1..end].to_string()))
            }
            Token::Coordinates(start, middle, end) => result.push(Token::Coord(
                line[start + 1..middle].to_string().parse::<i32>().unwrap(),
                line[middle + 1..end].to_string().parse::<i32>().unwrap(),
            )),
            Token::Ruminations(start, v) => {
                if v.len() != 0 {
                    let mut cur = start;
                    let mut next = v.get(0usize).unwrap();
                    let mut i = 0usize;
                    let mut arrangement = Vec::with_capacity(128usize);

                    while cur < *next {
                        let mut runes = Vec::with_capacity(256usize);
                        let mut tok = None;
                        for c in line[cur + 1..*next].chars() {
                            match c {
                                'a' | 'b' | 'g' | 'r' | 'l' | 'p' | 's' | 'd' => {
                                    if let Some(t) = tok {
                                        runes.push(t);
                                    }
                                    tok = Some(RuneToken::Action(c, 1u8, ConditionRuneToken::None));
                                }
                                'n' | 'v' | 'k' | 't' => {
                                    if let Some(t) = tok {
                                        runes.push(t);
                                    }
                                    tok = Some(RuneToken::Manip(c, 1u8, ConditionRuneToken::None));
                                }
                                '0' | '1' | '2' | '3' => {
                                    if let Some(RuneToken::Action(c2, _, cond)) = tok {
                                        tok = Some(RuneToken::Action(
                                            c2,
                                            c.to_string().parse::<u8>().unwrap(),
                                            cond,
                                        ));
                                    } else if let Some(RuneToken::Manip(c2, _, cond)) = tok {
                                        tok = Some(RuneToken::Manip(
                                            c2,
                                            c.to_string().parse::<u8>().unwrap(),
                                            cond,
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Some(t) = tok {
                            runes.push(t);
                        }
                        if runes.len() > 0 {
                            arrangement.push(runes);
                        }
                        cur = *next;
                        if i < v.len() - 1 {
                            i += 1;
                            next = v.get(i).unwrap();
                        }
                    }
                    result.push(Token::Runes(arrangement));
                } else {
                    result.push(Token::Runes(Vec::with_capacity(0)));
                }
            }
            _ => result.push(t),
        }
    }

    result
}
