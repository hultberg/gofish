use std::io;
use std::ops;
use std::collections::HashMap;
use rand::Rng;

struct TerminalInfo {
    height: u16,
    width: u16,
}

#[derive(Eq, Hash, Copy, Clone, PartialEq)]
enum CardRank {
    Heart,
    Diamond,
    Spade,
    Clover,
}

impl CardRank {
    pub fn get_label(&self) -> String {
        match self {
            CardRank::Heart => String::from("♥"),
            CardRank::Diamond => String::from("♦"),
            CardRank::Spade => String::from("♠"),
            CardRank::Clover => String::from("♣"),
        }
    }

    pub fn get_rank_id(&self) -> u8 {
        match self {
            CardRank::Heart => 1,
            CardRank::Diamond => 2,
            CardRank::Spade => 3,
            CardRank::Clover => 4,
        }
    }
}

struct Player {
    name: String,
    is_cpu: bool,
    cards: Vec<Card>,
    books: Vec<Book>,
}

impl Player {
    fn new(name: &String) -> Player {
        Player {
            name: name.clone(),
            is_cpu: false,
            cards: Vec::with_capacity(52),
            books: Vec::with_capacity(13),
        }
    }

    fn new_cpu() -> Player {
        Player {
            name: String::from("Computer"),
            is_cpu: true,
            cards: Vec::with_capacity(52),
            books: Vec::with_capacity(13),
        }
    }

    fn get_card_labels(&self) -> Vec<String> {
        let mut all_cards: Vec<_> = self.cards.to_vec();
        all_cards.sort_by(|a, b| {
            let a_rank_num = a.rank.get_rank_id();
            let b_rank_num = b.rank.get_rank_id();
            return a_rank_num.cmp(&b_rank_num).then_with(|| a.number.cmp(&b.number));
        });
        let all_cards_iterator = all_cards.into_iter();
        all_cards_iterator.map(|card| card.get_label()).collect()
    }
}

struct Book {
    number: u8,
}

#[derive(Copy, Clone)]
struct Card {
    rank: CardRank,
    number: u8,
}

impl Card {
    fn get_label(&self) -> String {
        let num_key = match self.number {
            11 => String::from("J"),
            12 => String::from("Q"),
            13 => String::from("K"),
            14 => String::from("A"),
            _ => self.number.to_string(),
        };

        let mut label = self.rank.get_label().to_string();
        label.push_str(&num_key);
        label
    }
}

struct GameLog {
    status_lines: Vec<String>,
    turn: usize,
}

impl GameLog {
    fn new() -> GameLog {
        GameLog {
            turn: 1,
            status_lines: vec![],
        }
    }

    fn clear(&mut self) {
        self.status_lines.clear();
    }

    fn add_status_line(&mut self, message: String) {
        let mut message_formatted = format!("T{} - ", self.turn);
        message_formatted.push_str(&message);
        self.status_lines.push(message_formatted);
    }
}

fn main() {
    // get info in seperate scope.
    let terminal_info = {
        let (terminal_width, terminal_height) = termion::terminal_size().unwrap();

        TerminalInfo {
            width: terminal_width,
            height: terminal_height,
        }
    };

    // clear screen on init.
    println!("{}", termion::clear::All);

    let mut deck = generate_deck();
    assert_eq!(52, deck.len());

    // Welcome:
    println!("Welcome to Go Fish");
    println!("Please enter your name:");

    let player_name = {
        let mut temp_string = String::new();
        io::stdin().read_line(&mut temp_string).expect("Unable to read your name");
        temp_string.trim().to_string()
    };

    // Create player instances.
    let mut player = Player::new(&player_name);
    let mut opponent = Player::new_cpu();

    give_random_card(&mut deck, &mut player, 7);
    give_random_card(&mut deck, &mut opponent, 7);

    let mut game_log = GameLog::new();

    loop {
        // create books.
        check_cards_for_books(&mut game_log, &mut player);
        check_cards_for_books(&mut game_log, &mut opponent);

        let mut game_standings: Vec<&Player> = vec![&player, &opponent];
        game_standings.sort_by(|a, b| {
            return a.books.len().cmp(&b.books.len()).reverse().then_with(|| a.name.cmp(&b.name));
        });

        if let Some(winner) = determine_winner(&game_standings) {
            gameover(&winner);
            break;
        }

        println!("{}{}Go Fish v0.3.0",
            termion::clear::All,
            termion::cursor::Goto(1, 1));

        println!("{}{} cards in deck", termion::cursor::Goto(1, 2), deck.len());

        let player_books_string: String = {
            let player_books_strings: Vec<String> = game_standings.into_iter().map(|this_player| format!("{} ({})", this_player.name, this_player.books.len())).collect();
            format!("Standings: {}", player_books_strings.join(", "))
        };
        println!("{}{}", termion::cursor::Goto(1, 3), player_books_string);

        let player_cards_string = player.get_card_labels().join(", ");
        println!("{}{}",
            termion::cursor::Goto(1, terminal_info.height / 2),
            player_cards_string);

        // print status lines
        for (pos, line) in game_log.status_lines.iter().enumerate() {
            println!("{}{}",
                termion::cursor::Goto(1, terminal_info.height - 2 - (pos as u16)),
                line);
        }

        if !player.is_cpu {
            // clear the last game log
            game_log.clear();
        }

        let (next_player, next_opponent, is_next_turn) = turn(
            &terminal_info,
            &mut game_log,
            &mut deck,
            player,
            opponent
        );

        player = next_player;
        opponent = next_opponent;

        if is_next_turn {
            game_log.add_status_line(String::from("End of turn"));
            game_log.turn += 1;
        }
    }
}

// FIXME: Pass Player structs instead of String
fn determine_winner<'a>(standings: &'a Vec<&Player>) -> Option<&'a Player> {
    let mut total_book_count = 0;

    for &player in standings.iter() {
        total_book_count = total_book_count + player.books.len();
    }

    if total_book_count >= 13 {
        return Some(&standings[0]);
    }

    return None;
}

fn gameover(winner: &Player) {
    println!("{}{}{} has won with {} books", termion::clear::All, termion::cursor::Goto(1, 1), winner.name, winner.books.len());
}

fn turn(
    terminal_info: &TerminalInfo,
    game_log: &mut GameLog,
    mut deck: &mut Vec<Card>,
    mut current_player: Player,
    mut current_opponent: Player,
) -> (Player, Player, bool) {
    let card_face_value: u8 = {
        if current_player.is_cpu {
            let card_value = get_cpu_requesting_card_value(&current_player);
            game_log.add_status_line(format!("Computer requested {}", card_value));
            card_value
        } else {
            get_player_requesting_card_value(&terminal_info)
        }
    };

    if card_face_value < 2 || card_face_value > 14 {
        game_log.add_status_line(String::from("Invalid card face value"));
        return (current_player, current_opponent, false);
    }

    let mut has_card: bool = false;
    for &player_card in current_player.cards.iter() {
        if player_card.number == card_face_value {
            has_card = true;
            break;
        }
    }

    if !has_card && !current_player.is_cpu {
        game_log.add_status_line(String::from("You dont have that card"));
        return (current_player, current_opponent, false);
    }

    let mut found_cards: Vec<Card> = vec![];

    for &other_player_card in current_opponent.cards.iter() {
        if other_player_card.number == card_face_value {
            found_cards.push(other_player_card);
        }
    }

    let found_cards_length = found_cards.len();

    if found_cards_length > 0 {
        for card in found_cards.iter() {
            game_log.add_status_line(String::from(format!("{} -{}-> {}",
                current_opponent.name,
                card.get_label(),
                current_player.name)));
            current_player.cards.push(*card);
        }

        found_cards.clear();

        current_opponent.cards.retain(|&card| card.number != card_face_value);
        return (current_player, current_opponent, false);
    }

    if deck.len() <= 0 {
        return (current_opponent, current_player, true);
    }

    give_random_card(&mut deck, &mut current_player, 1);

    let last_card = current_player.cards.last().unwrap();
    let mut last_card_label = String::new();
    if !current_player.is_cpu {
        last_card_label = last_card.get_label();
    }

    game_log.add_status_line(String::from(format!("Go Fish, deck -{}-> {}",
        last_card_label,
        current_player.name)));

    return (current_opponent, current_player, true);
}

fn check_cards_for_books(game_log: &mut GameLog, current_player: &mut Player) {
    let mut cards_count: HashMap<u8, u8> = HashMap::new();

    for &card in current_player.cards.iter() {
        *cards_count.entry(card.number).or_insert(0) += 1;

        if let Some(x) = cards_count.get(&card.number) {
            if *x >= 4 {
                // create a book of this.
                current_player.books.push(Book {
                    number: card.number,
                });

                game_log.add_status_line(String::from(format!("{} collected a book of {}", current_player.name, card.number)));
            }
        };
    }

    cards_count.retain(|_k, val| *val >= 4);

    if cards_count.len() > 0 {
        // Returning false means the card gets removed.

        current_player.cards.retain(|&card| {
            if cards_count.contains_key(&card.number) {
                let num_cards = *cards_count.get(&card.number).unwrap();
                return num_cards < 4; // Below four is allowed.
            }

            return true;
        });
    }
}

fn get_player_requesting_card_value(
    terminal_info: &TerminalInfo
) -> u8 {
    println!("{}Enter a card face value to request (0-10, Jack, Queen, King, Ace):",
        termion::cursor::Goto(1, terminal_info.height - 1));
    print!("{}>>> ",
        termion::cursor::Goto(1, terminal_info.height));

    let mut command = String::new();
    io::stdin().read_line(&mut command).expect("Unable to read line");
    let command = command.trim();
    let command_as_int = match command.parse() {
        Ok(num) => num,
        _ => 0,
    };

    // match as int?
    match command_as_int {
        2 ... 10 => return command_as_int,
        _ => (),
    }

    let command_lowercase = &*command.to_lowercase();

    return match command_lowercase {
        "j" | "jack" => 11,
        "q" | "queen" => 12,
        "k" | "king" => 13,
        "a" | "ace" => 14,
        // "?hand" => 60,
        // "?deck_len" => 61,
        // "?books" => 62,
        _ => 0,
    };
}

fn get_cpu_requesting_card_value(player: &Player) -> u8 {
    let mut cards_count: HashMap<u8, u8> = HashMap::new();

    for card in player.cards.iter() {
        *cards_count.entry(card.number).or_insert(0) += 1;
    }

    let count_vec: Vec<_> = cards_count.into_iter().collect();
    if count_vec.len() > 0 {
        let count_value = count_vec[count_vec.len() - 1].0;

        return count_value;
    }

    return 2;
}

fn give_random_card(
    deck: &mut Vec<Card>,
    player: &mut Player,
    num_cards: u8
) {
    // this is just a reference and is cached in each thread:
    let mut rng = rand::thread_rng();

    let range = ops::RangeInclusive::new(1, num_cards);
    for _num in range {
        let rand_index = rng.gen_range(0, deck.len());
        let card = deck.get(rand_index).unwrap();
        player.cards.push(card.clone());
        deck.remove(rand_index);
    }
}

fn generate_deck() -> Vec<Card> {
    let all_ranks: [CardRank; 4] = [CardRank::Heart, CardRank::Diamond, CardRank::Spade, CardRank::Clover];

    // init the deck
    let mut deck: Vec<Card> = vec![];

    for &rank in all_ranks.iter() {
        for number in 2..15 {
            let card = Card {
                rank: rank.clone(),
                number,
            };
            deck.push(card);
        }
    }

    deck
}
