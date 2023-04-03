mod cli;
mod minesweeper;
use crate::minesweeper::GameSettings;
use clap::Parser;

/// Minesweeper
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Size of game board
    #[arg(
        long,
        short,
        value_parser = parse_dimensions
    )]
    dimensions: Option<[usize; 2]>,
    /// Width of game board
    #[arg(
        long,
        requires("height"),
        requires("mines"),
        conflicts_with = "dimensions"
    )]
    width: Option<usize>,
    /// Height of game board
    #[arg(
        long,
        requires("width"),
        requires("mines"),
        conflicts_with = "dimensions"
    )]
    height: Option<usize>,
    /// Number of mines
    #[arg(short, long)]
    mines: Option<usize>,
}

fn main() {
    let args = Args::parse();
    fn get_settings(args: &Args) -> Option<GameSettings> {
        if args.mines.is_none() {
            return None;
        }
        let [width, height] = if let (Some(width), Some(height)) = (args.height, args.width) {
            [width, height]
        } else if let Some(dimensions) = args.dimensions {
            dimensions
        } else {
            return None;
        };
        Some(GameSettings {
            width,
            height,
            mines: args.mines.unwrap(),
        })
    }
    let settings = get_settings(&args);
    cli::begin(settings);
}

fn parse_dimensions(s: &str) -> Result<[usize; 2], String> {
    let mut dimension: [usize; 2] = [0, 0];
    let mut atr_itr = s.split('x');
    for a in dimension.iter_mut() {
        if let Some(str) = atr_itr.next() {
            if let Ok(num) = str.parse::<usize>() {
                *a = num;
            } else {
                return Err("Each dimension must be a number.".into());
            }
        }
    }
    if dimension[0] == 0 || dimension[1] == 0 {
        return Err("Two dimensions separated by an `x` are required.".into());
    }

    Ok(dimension)
}
