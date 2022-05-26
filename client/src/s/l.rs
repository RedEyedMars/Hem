use crate::a::b::Board;
use crate::a::GameState;
use crate::s::{tokenize, Token};

use core::future::Future;
use std::pin::Pin;
use wasm_bindgen::prelude::*;

pub async fn read_level(level_name: &str, game: &mut GameState) -> Result<(), failure::Error> {
    let mut filename = String::from("./res/levels/");
    filename.push_str(level_name);
    filename.push_str(".lvl");
    read_level_from_string(
        //&crate::s::read_file(filename.as_str()).unwrap().into_bytes(),
        match match (Box::pin(unsafe { crate::s::fetch_level(filename.as_str()) })
            as Pin<Box<dyn Future<Output = Result<JsValue, JsValue>>>>)
            .await
        {
            Ok(response) => Ok(response.as_string()),
            Err(e) => Err(failure::err_msg(format!("Error!{:?}", e))),
        }? {
            Some(s) => Ok(s),
            None => Err(failure::err_msg("No file message: None!")),
        }?,
        game,
    )?;
    Ok(())
}

pub fn read_level_from_string(
    filecontents: String,
    game: &mut GameState,
) -> Result<(), failure::Error> {
    game.setup_boards();
    let mut board_len = 0usize;
    for l in filecontents.lines() {
        board_len = read_level_line(l, board_len, game)?;
    }
    if game.boards().len() > 0 {
        game.set_first_board()
    }
    println!("Complete?");
    Ok(())
}

pub fn read_level_line(
    line: &str,
    size: usize,
    game: &mut GameState,
) -> Result<usize, failure::Error> {
    Ok(match line.chars().nth(0) {
        Some('>') => {
            game.push_board(read_board(&line[1..])?);
            if size == 0usize {
                size
            } else {
                size + 1
            }
        }
        _ => size,
    })
}

pub fn read_board(line: &str) -> Result<Board, failure::Error> {
    let tokens = tokenize(line);
    println!("{:?}", tokens);
    match tokens.as_slice() {
        [Token::String(name)] => Ok(Board::new(crate::a::b::view::View::from(name))),
        _ => Err(failure::err_msg(format!(
            "Unable to parse {} into new Board",
            line
        ))),
    }
}

pub fn save_level(level_name: &str, game: &GameState) /*-> Result<(), failure::Error>*/
{
    let mut tokens = Vec::with_capacity(2048);
    save_board_line(&game.board(), &mut tokens);
    let mut filename = String::from("./res/levels/");
    filename.push_str(level_name);
    filename.push_str(".lvl");
    crate::s::post_level(filename.as_str(), crate::s::extract_file_string(&tokens));
    //crate::s::write_file(filename.as_str(), &tokens)
}
fn save_board_line(board: &Board, tokens: &mut Vec<Vec<Token>>) {
    tokens.push(vec![Token::Varying('>'), Token::String(board.as_text())]);
}
