use std::path::Path;

use image::{io::Reader as ImageReader, ImageBuffer, Rgba, RgbaImage};

use crate::lexer::{Position, Token, TokenKind};

#[derive(Debug)]
pub enum ParseError {
    FileNotFound,
    DecodeFailed,
}

#[derive(Debug)]
pub enum EncoderError {
    FileNotFound,
    DecodeFailed,
}

const MINIMUM_PIXEL_DISTANCE: u8 = 10;
const MAXIMUM_PIXEL_DISTANCE: u8 = 18;

#[inline]
fn pixel_distance(a: &image::Rgba<u8>, b: &image::Rgba<u8>) -> Option<u8> {
    let [r1, g1, b1, a1] = a.0;
    let [r2, g2, b2, a2] = b.0;

    let diff = r1.abs_diff(r2);
    if diff < MINIMUM_PIXEL_DISTANCE || diff > MAXIMUM_PIXEL_DISTANCE {
        return None;
    }

    if g1.abs_diff(g2) != diff || b1.abs_diff(b2) != diff || a1.abs_diff(a2) != diff {
        return None;
    }

    Some(diff)
}

fn distance_to_kind(distance: u8) -> Option<TokenKind> {
    match distance - MINIMUM_PIXEL_DISTANCE {
        0 => Some(TokenKind::Increment(0)),
        1 => Some(TokenKind::Decrement(0)),
        2 => Some(TokenKind::Right(0)),
        3 => Some(TokenKind::Left(0)),
        4 => Some(TokenKind::LoopStart),
        5 => Some(TokenKind::LoopEnd),
        6 => Some(TokenKind::PutChar),
        7 => Some(TokenKind::ReadChar),
        8 => Some(TokenKind::EOF),
        _ => None,
    }
}

fn kind_to_distance(kind: &TokenKind) -> u8 {
    MINIMUM_PIXEL_DISTANCE
        + match kind {
            TokenKind::Increment(_) => 0,
            TokenKind::Decrement(_) => 1,
            TokenKind::Right(_) => 2,
            TokenKind::Left(_) => 3,
            TokenKind::LoopStart => 4,
            TokenKind::LoopEnd => 5,
            TokenKind::PutChar => 6,
            TokenKind::ReadChar => 7,
            TokenKind::EOF => 8,
        }
}

#[inline]
fn kind_count(kind: &TokenKind) -> u32 {
    match kind {
        TokenKind::Increment(count)
        | TokenKind::Decrement(count)
        | TokenKind::Right(count)
        | TokenKind::Left(count) => *count as u32,
        _ => 1,
    }
}

#[inline]
fn increase_kind(kind: &mut TokenKind, count: u32) {
    match kind {
        TokenKind::Increment(c) => *c += count as usize,
        TokenKind::Decrement(c) => *c += count as usize,
        TokenKind::Right(c) => *c += count as usize,
        TokenKind::Left(c) => *c += count as usize,
        _ => {}
    }
}

pub fn parse_image(path: &str) -> Result<Vec<Token>, EncoderError> {
    let mut tokens = Vec::new();

    let img = ImageReader::open(path)
        .map_err(|_| EncoderError::FileNotFound)?
        .decode()
        .map_err(|_| EncoderError::DecodeFailed)?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let img: RgbaImage = ImageBuffer::from_vec(width, height, img.into_raw()).unwrap();

    let mut pixels = img.pixels().peekable();
    let mut line = 0;
    let mut column = 0;

    let mut stacking_token: Option<TokenKind> = None;

    while let Some(pixel) = pixels.next() {
        column += 1;

        let next_pixel = match pixels.peek() {
            Some(pixel) => pixel,
            None => break,
        };

        // If we have a distance, we have a token
        if let Some(distance) = pixel_distance(pixel, &next_pixel) {
            let kind = distance_to_kind(distance).expect("corresponding kind");

            // If we have a stacking token, push it
            if let Some(last_kind) = stacking_token {
                let count = kind_count(&last_kind);
                tokens.push(Token::new(
                    last_kind,
                    Position::new(line, column - count - 1),
                ));
            }

            match kind {
                TokenKind::Increment(_)
                | TokenKind::Decrement(_)
                | TokenKind::Right(_)
                | TokenKind::Left(_) => {
                    // Set the stacking token
                    stacking_token = Some(kind);
                }
                _ => {
                    // Push the token
                    if matches!(kind, TokenKind::EOF) {
                        tokens.push(Token::new(kind, Position::new(line, column)));
                        stacking_token = None;
                        break;
                    } else {
                        tokens.push(Token::new(kind, Position::new(line, column)));
                        stacking_token = None;
                    }
                }
            }
        // Otherwise, we have a continuation of the last token
        } else {
            stacking_token = stacking_token.map(|mut kind| {
                increase_kind(&mut kind, 1);
                kind
            });
        }

        if column == width {
            column = 0;
            line += 1;
        }
    }

    // If we have a stacking token, push it
    if let Some(last_kind) = stacking_token {
        let count = kind_count(&last_kind);
        tokens.push(Token::new(
            last_kind,
            Position::new(line, column - count - 1),
        ));
    }

    Ok(tokens)
}

#[inline]
fn encoded_pixel(distance: u8, values: &[u8; 4]) -> [u8; 4] {
    let [r, g, b, a] = values;

    let r = if r.saturating_add(distance) == u8::MAX {
        r - distance
    } else {
        r + distance
    };

    let g = if g.saturating_add(distance) == u8::MAX {
        g - distance
    } else {
        g + distance
    };

    let b = if b.saturating_add(distance) == u8::MAX {
        b - distance
    } else {
        b + distance
    };

    let a = if a.saturating_add(distance) == u8::MAX {
        a - distance
    } else {
        a + distance
    };

    [r, g, b, a]
}

pub fn encode_image(path: &str, tokens: Vec<Token>) -> Result<(), EncoderError> {
    let mut tokens = tokens.into_iter();

    let img = ImageReader::open(path)
        .map_err(|_| EncoderError::FileNotFound)?
        .decode()
        .map_err(|_| EncoderError::DecodeFailed)?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let mut pixels: Vec<Rgba<u8>> = img.pixels().map(|pixel| pixel.clone()).collect();

    let mut i = 0;
    while i < pixels.len() {
        let pixel = pixels[i];
        let next_pixel = match pixels.get_mut(i + 1) {
            Some(pixel) => pixel,
            None => break,
        };

        let token = match tokens.next() {
            Some(token) => token,
            None => break,
        };

        match token.kind() {
            &TokenKind::Increment(amount)
            | &TokenKind::Decrement(amount)
            | &TokenKind::Left(amount)
            | &TokenKind::Right(amount) => {
                let distance = kind_to_distance(token.kind());
                let encoded = encoded_pixel(distance, &pixel.0);

                next_pixel.0 = encoded;

                for _ in 0..amount {
                    if i + 1 >= pixels.len() {
                        break;
                    }
                    let pixel = pixels[i + 1];

                    let next_pixel = match pixels.get_mut(i + 2) {
                        Some(pixel) => pixel,
                        None => break,
                    };

                    if let Some(_) = pixel_distance(&pixel, &next_pixel) {
                        // We don't want this, modify first pixel so that change is not the same
                        let r = next_pixel.0[0];
                        next_pixel.0[0] = if r > u8::MAX / 2 { r - 1 } else { r + 1 };
                    }

                    i += 1;
                }
            }
            TokenKind::LoopStart
            | TokenKind::LoopEnd
            | TokenKind::PutChar
            | TokenKind::ReadChar
            | TokenKind::EOF => {
                let distance = kind_to_distance(token.kind());
                next_pixel.0 = encoded_pixel(distance, &pixel.0);
            }
        }

        i += 1;
    }

    let buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = pixels[(y * width + x) as usize];
        image::Rgba(pixel.0)
    });

    let image_path = Path::new(path);
    let new_path = format!(
        "encoded-{}",
        image_path.file_name().unwrap().to_str().unwrap()
    );
    let new_path = std::path::Path::new(new_path.as_str());
    buffer.save(new_path).unwrap();

    Ok(())
}

pub fn read_n_pixels(path: &str, n: usize) {
    let img = ImageReader::open(path)
        .map_err(|_| EncoderError::FileNotFound)
        .unwrap()
        .decode()
        .map_err(|_| EncoderError::DecodeFailed)
        .unwrap()
        .to_rgba8();

    let pixels: Vec<Rgba<u8>> = img.pixels().map(|pixel| pixel.clone()).collect();

    for i in 0..n {
        println!("{:?}", pixels[i]);
    }
}