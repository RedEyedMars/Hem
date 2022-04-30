use crate::a::b::Board;
use crate::a::b::Wall;
use crate::a::GameState;
use crate::s::{tokenize, Token};

use core::future::Future;
use packed_simd::{f32x2, u32x2};
use std::pin::Pin;
use wasm_bindgen::prelude::*;

pub async fn read_level(level_name: &str, game: &mut GameState) -> Result<(), failure::Error> {
    let mut filename = String::from("./res/levels/");
    filename.push_str(level_name);
    filename.push_str(".lvl");
    read_level_from_string(
        //&crate::s::read_file(filename.as_str()).unwrap().into_bytes(),
        match match (Box::pin(crate::s::fetch_level(filename.as_str()))
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
    nodejs_helper::console::log(filecontents.as_str());
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
        Some('W') => {
            read_wall(&line[1..], game.get_mut_board(size));
            size
        }
        _ => size,
    })
}

pub fn read_board(line: &str) -> Result<Board, failure::Error> {
    let tokens = tokenize(line);
    println!("{:?}", tokens);
    match tokens.as_slice() {
        [Token::String(name), Token::Coord(width, height)] => Ok(Board::new(
            name.clone(),
            (*width) as usize,
            (*height) as usize,
        )),
        _ => Err(failure::err_msg(format!(
            "Unable to parse {} into new Board",
            line
        ))),
    }
}
pub fn read_wall(line: &str, board: &mut Board) {
    let tokens = tokenize(line);
    match tokens.as_slice() {
        [Token::Coord(x1, y1), Token::Coord(x2, y2)] => {
            if x1 == x2 {
                // vertical
                for y in *y1..y2 + 1 {
                    board
                        .add_wall(u32x2::new(*x1 as u32, y as u32))
                        .expect("Could not add wall, veritical");
                }
            } else {
                // horizontal
                for x in *x1..x2 + 1 {
                    board
                        .add_wall(u32x2::new(x as u32, *y1 as u32))
                        .expect("Could not add wall, horizontal");
                }
            }
        }
        _ => {}
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
    tokens.push(vec![
        Token::Varying('>'),
        Token::String(String::from_utf8(board.name().clone()).expect("Could not parse board name")),
        Token::Coord(board.width() as i32, board.height() as i32),
    ]);
    for (wall, xy) in board.walls().iter() {
        save_wall_line(wall, *xy, tokens);
    }
}

pub fn save_wall_line(_wall: &Wall, xy: f32x2, tokens: &mut Vec<Vec<Token>>) {
    tokens.push(vec![
        Token::Varying('W'),
        Token::Coord(xy.extract(0) as i32, xy.extract(1) as i32),
    ]);
}
