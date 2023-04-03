use crate::minesweeper::{GameEvent, GameState, Minesweeper, TileModifier, TileState};
use crate::minesweeper::{GameSettings, Tile};
use colored::{ColoredString, Colorize};
use std::error::Error;
use std::time;

pub enum Action {
    Sweep(usize, usize),
    Flag(usize, usize),
    Question(usize, usize),
    Quit,
}

impl Tile {
    fn render(&self) -> ColoredString {
        if let Some(modifier) = self.modifier {
            if modifier == TileModifier::Flagged {
                return "F".bright_red();
            }
        }
        if !self.swept {
            return "?".bright_black();
        }

        match self.state {
            TileState::Zero => " ".black(),
            TileState::One => "1".bright_blue(),
            TileState::Two => "2".green(),
            TileState::Three => "3".bright_red(),
            TileState::Four => "4".blue(),
            TileState::Five => "5".red(),
            TileState::Six => "6".cyan(),
            TileState::Seven => "7".black(),
            TileState::Eight => "8".black(),
            TileState::Mine => "X".red(),
        }
    }
}

fn format_time(duration: time::Duration) -> String {
    format!(
        "Time Elapsed = {} │ ",
        format!(
            "{:0>2}:{:0>2}",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        )
        .bright_yellow(),
    )
}

pub fn begin(start_settings: Option<GameSettings>) {
    let settings = if let Some(s) = start_settings {
        s
    } else {
        if let Ok(a) = get_params() {
            a
        } else {
            GameSettings {
                width: 30,
                height: 16,
                mines: 99,
            }
        }
    };
    let mut game = Minesweeper::new(&settings).unwrap();

    render(&game);

    // Game Loop
    loop {
        while let Some(e) = game.events.next() {
            match e {
                GameEvent::SweepDone
                | GameEvent::FlagTile(_, _, _)
                | GameEvent::RevealMine(_, _, _) => {
                    render(&game);
                }
                _ => (),
            }
        }
        if game.state == GameState::GameOver {
            println!("{}", "Game Over!".red());
            break;
        }
        if game.state == GameState::Victory {
            println!("{}", "You Win!".red());
            break;
        }
        let result = take_input((game.board.width, game.board.height));
        if let Ok(action) = result {
            match action {
                Action::Sweep(x, y) => {
                    game.sweep(x, y);
                }
                Action::Flag(x, y) => {
                    game.flag(x, y);
                }
                Action::Quit => {
                    break;
                }

                _ => (),
            }
        } else if let Err(error) = result {
            println!("{}", error);
        }
    }
}

pub fn render(game: &Minesweeper) {
    print!("{}c", 27 as char);
    let y_max_len = (game.board.height + 1).to_string().len();
    print!("{: ^1$}┃", "", y_max_len);
    for (x, i) in (0..game.board.width).enumerate() {
        print!("{}", x + 1);
        if i < 9 && x != game.board.width - 1 {
            print!(" ")
        }
    }
    if game.board.width < 9 {
        print!(" ")
    }
    println!("{}", "┃".white());

    let bar = format!(
        "{0}{1:━>y_max_len$}{0}{2:━>3$}",
        "━",
        "╋",
        "┫",
        game.board.width * 2
    )
    .white();
    println!("{}", bar);

    for y in 0..game.board.height {
        let mut board_line = String::from("");

        for x in 0..(game.board.width) {
            board_line.push_str(&game.board.tiles[x][y].render().to_string());
            board_line.push_str(" ");
        }
        let line_num = format!("{: ^y_max_len$}", y + 1);
        println!("{}{2}{}{2}", line_num, board_line.on_white(), "┃".white());
    }
    println!(
        "{}",
        format!(
            "{2}{0:━>y_max_len$}{2}{1:━>dim$}",
            "┻",
            "┛",
            "━",
            dim = (game.board.width * 2)
        )
        .white()
    );

    if let Some(start_time) = game.start_time {
        let elapsed = start_time.elapsed();
        print!("{}", format_time(elapsed))
    }
    println!("Commands = x,y: sweep, fx,y: flag, q: quit");
}

pub fn get_params() -> Result<GameSettings, Box<dyn Error>> {
    print!("{}c", 27 as char);
    println!("{}", "Input options:".yellow().bold().underline());
    let mut params: [usize; 3] = [16, 16, 40];
    let default_msg = "Press Enter for default";
    let param_name = ["Width:", "Height:", "Number of Mines:"];
    for (i, _) in param_name.iter().enumerate() {
        loop {
            println!(
                "{} ({}: {})",
                param_name[i].yellow(),
                default_msg,
                params[i]
            );
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line)?;
            if line == String::from("\n") || line == String::from("\r\n") {
                println!("{} {}", "default".italic(), params[i]);
                break;
            }
            if let Ok(a) = line.trim().parse::<usize>() {
                if a == 0 {
                    continue;
                }
                params[i] = a;
                break;
            };
        }
    }
    let options = GameSettings {
        width: params[0],
        height: params[1],
        mines: params[2],
    };
    Ok(options)
}

pub fn take_input(dimensions: (usize, usize)) -> Result<Action, Box<dyn Error>> {
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line)?;

    let first_char = line.chars().next().unwrap();

    if first_char.is_alphabetic() {
        line = line[1..].to_string();
    }

    let mut x = 0;
    let mut y = 0;
    for (i, a) in line.split(",").enumerate() {
        let k = a.trim();
        let coordinate: usize = if let Ok(a) = k.parse() { a } else { 1 };

        if coordinate < 1 {
            return Err("Invalid Location".into());
        }
        if i == 0 {
            x = coordinate - 1
        } else {
            y = coordinate - 1
        }
    }
    if y >= dimensions.1 || x >= dimensions.0 {
        return Err("Invalid Location".into());
    }

    let action: Action = if first_char.is_alphabetic() {
        match first_char {
            'f' => Action::Flag(x, y),
            '?' => Action::Question(x, y),
            'q' => Action::Quit,
            _ => Action::Sweep(x, y),
        }
    } else {
        Action::Sweep(x, y)
    };

    Ok(action)
}


