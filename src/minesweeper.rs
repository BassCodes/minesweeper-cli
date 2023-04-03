use rand::Rng;

use std::collections::VecDeque;
use std::error::Error;
use std::time;
use std::vec;

const SCAN: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const SAFE_ZONE: [(isize, isize); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
#[derive(Copy, Clone, PartialEq)]

pub enum TileState {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Mine,
}
#[derive(Copy, Clone, PartialEq)]
pub enum TileModifier {
    Flagged,
    _Unsure,
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub state: TileState,
    pub modifier: Option<TileModifier>,
    pub swept: bool,
    safe: bool,
}

#[derive(PartialEq)]
pub enum GameState {
    Empty,
    Playing,
    GameOver,
    Victory,
}
pub enum GameEvent {
    RevealMine(usize, usize, Tile),
    RevealTile(usize, usize, Tile),
    FlagTile(usize, usize, Tile),
    SweepDone,
    SweepBegin,
    InitDone,
    FlagAllMines,
    GameStart,
    GameEnd(GameBoard),
}
pub struct GameSettings {
    pub width: usize,
    pub height: usize,
    pub mines: usize,
}

pub struct Minesweeper {
    pub board: GameBoard,
    pub state: GameState,
    pub start_time: Option<time::Instant>,
    pub events: Events,
}
pub struct Events {
    events: Vec<GameEvent>,
}

impl Events {
    fn add(&mut self, event: GameEvent) {
        self.events.push(event);
    }
    pub fn next(&mut self) -> Option<GameEvent> {
        if self.events.len() > 0 {
            self.events.pop()
        } else {
            None
        }
    }
}
#[derive(Clone)]
pub struct GameBoard {
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    pub flags: usize,
    valid_flags: usize,
}

impl Minesweeper {
    pub fn new(settings: &GameSettings) -> Result<Minesweeper, Box<dyn Error>> {
        if settings.width == 0 || settings.height == 0 {
            return Err("Can't make game board with zero length dimension".into());
        };
        if (settings.width * settings.height) as isize - 9 < settings.mines as isize {
            // Can't have more mines than tiles on the game board. Else, the
            // loop that places mines on board will never complete.
            return Err("Not enough space for mines".into());
        }

        let board = GameBoard {
            tiles: vec![
                vec![
                    Tile {
                        state: TileState::Zero,
                        modifier: None,
                        swept: false,
                        safe: false,
                    };
                    settings.height
                ];
                settings.width
            ],
            width: settings.width,
            height: settings.height,
            mines: settings.mines,
            flags: 0,
            valid_flags: 0,
        };
        let game = Self {
            board,
            state: GameState::Empty,
            start_time: None,
            events: Events { events: vec![] },
        };

        Ok(game)
    }
    fn generate(&mut self, avoid_x: usize, avoid_y: usize) {
        self.events.add(GameEvent::GameStart);

        let width = self.board.width;
        let height = self.board.height;
        // Make list of all safe positions which are actually on board,
        //removing all which are before, or past the bounds of the board.
        let mut valid_safezone: Vec<(isize, isize)> = SAFE_ZONE.to_vec();
        valid_safezone.retain_mut(|pos: &mut (isize, isize)| {
            let adjusted_x = pos.0 + avoid_x as isize;
            let adjusted_y = pos.1 + avoid_y as isize;
            if adjusted_x >= 0
                && adjusted_y >= 0
                && adjusted_x < width as isize
                && adjusted_y < height as isize
            {
                pos.0 = adjusted_x;
                pos.1 = adjusted_y;
                return true;
            }
            false
        });
        for safe_pos in valid_safezone.iter() {
            let safe_x = safe_pos.0 as usize;
            let safe_y = safe_pos.1 as usize;
            self.board.tiles[safe_x][safe_y].safe = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        let mut rng = rand::thread_rng();

        let mut i = 0;

        while i != self.board.mines {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);

            let mut tile = &mut self.board.tiles[x][y];

            if tile.state == TileState::Mine || tile.safe == true {
                continue;
            }

            tile.state = TileState::Mine;

            i += 1;
        }

        for x in 0..width {
            for y in 0..height {
                if self.board.tiles[x][y].state == TileState::Mine {
                    for &scan_location in SCAN.iter() {
                        let new_x = x as isize + scan_location.0;
                        let new_y = y as isize + scan_location.1;
                        if new_x >= width as isize
                            || new_x < 0
                            || new_y >= height as isize
                            || new_y < 0
                        {
                            continue;
                        }
                        let new_x = new_x as usize;
                        let new_y = new_y as usize;
                        if self.board.tiles[new_x][new_y].state != TileState::Mine {
                            self.board.tiles[new_x][new_y].state =
                                match self.board.tiles[new_x][new_y].state {
                                    TileState::Zero => TileState::One,
                                    TileState::One => TileState::Two,
                                    TileState::Two => TileState::Three,
                                    TileState::Three => TileState::Four,
                                    TileState::Four => TileState::Five,
                                    TileState::Five => TileState::Six,
                                    TileState::Six => TileState::Seven,
                                    TileState::Seven => TileState::Eight,
                                    _ => TileState::Eight,
                                }
                        }
                    }
                }
            }
        }
        self.start_time = Some(time::Instant::now());
        self.state = GameState::Playing;
        self.events.add(GameEvent::InitDone);
    }
    pub fn sweep(&mut self, x: usize, y: usize) {
        if GameState::Empty == self.state {
            self.generate(x, y);
        }
        if self.state != GameState::Playing {
            return;
        }
        let &tile = &self.board.tiles[x][y];
        if let Some(_) = tile.modifier {
            return;
        }
        self.board.tiles[x][y].swept = true;
        self.events
            .add(GameEvent::RevealTile(x, y, self.board.tiles[x][y].clone()));

        if tile.state == TileState::Mine {
            self.events.add(GameEvent::RevealMine(x, y, tile.clone()));

            self.events.add(GameEvent::GameEnd(self.board.clone()));
            self.state = GameState::GameOver;
            return;
        };
        self.events.add(GameEvent::SweepBegin);

        let mut scan_list = VecDeque::from([(x, y)]);
        while scan_list.len() > 0 {
            for &scan_location in SCAN.iter() {
                let old_tile = self.board.tiles[scan_list[0].0][scan_list[0].1];
                if old_tile.state != TileState::Zero {
                    continue;
                }
                let x = scan_list[0].0 as isize + scan_location.0;
                let y = scan_list[0].1 as isize + scan_location.1;

                if x >= self.board.width as isize
                    || y >= self.board.height as isize
                    || x < 0
                    || y < 0
                {
                    continue;
                }
                let y = y as usize;
                let x = x as usize;
                let tile = self.board.tiles[x][y];
                if tile.swept {
                    continue;
                }
                scan_list.push_back((x, y));

                self.board.tiles[x][y].swept = true;
                self.events
                    .add(GameEvent::RevealTile(x, y, self.board.tiles[x][y].clone()));
            }
            scan_list.pop_front();
        }
        self.events.add(GameEvent::SweepDone);
    }
    pub fn flag(&mut self, x: usize, y: usize) {
        if self.state != GameState::Playing || self.board.tiles[x][y].swept {
            return;
        }

        let mut tile = &mut self.board.tiles[x][y];
        if let Some(TileModifier::Flagged) = tile.modifier {
            tile.modifier = None;
            if tile.state == TileState::Mine {
                self.board.valid_flags -= 1;
            }
            self.board.flags -= 1;
            self.events.add(GameEvent::FlagTile(x, y, tile.clone()));
        } else if tile.modifier.is_none() {
            tile.modifier = Some(TileModifier::Flagged);
            self.board.flags += 1;
            if tile.state == TileState::Mine {
                self.board.valid_flags += 1;
            }
            self.events.add(GameEvent::FlagTile(x, y, tile.clone()));

            if self.board.valid_flags == self.board.mines {
                self.events.add(GameEvent::FlagAllMines);
                self.state = GameState::Victory;
                return;
            }
        }
    }
}
