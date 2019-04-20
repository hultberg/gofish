use std::io;
use std::ops;
use std::collections::HashMap;
use rand::Rng;
use std::char;

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
            _ => String::from("LOL"),
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
        let mut labels = vec![];
        
        for &card in self.cards.iter() {
            labels.push(card.get_label());
        }
        
        labels
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
    
    // clear again for our main output.
    println!("{}", termion::clear::All);
    
    // Create player instances.
    let mut player = Player::new(&player_name);
    let mut opponent = Player::new_cpu();
    
    give_random_card(&mut deck, &mut player, 7);
    give_random_card(&mut deck, &mut opponent, 7);
    
    let mut current_status_lines: Vec<String> = vec![];
    
    loop {
        // create books.
        check_cards_for_books(&mut current_status_lines, &mut player);
        check_cards_for_books(&mut current_status_lines, &mut opponent);
        
        println!("{}{}Go Fish v0.1.0",
            termion::clear::All,
            termion::cursor::Goto(1, 1));
            
        let game_standings: Vec<(&String, usize)> = {
            let all_players: Vec<&Player> = vec![&player, &opponent];
            
            let mut book_counts: HashMap<&String, usize> = HashMap::new();
            for &player in all_players.iter() {
                book_counts.insert(&player.name, player.books.len());
            }
            let mut book_counts: Vec<_> = book_counts.into_iter().collect();
            book_counts.sort_by(|a, b| a.1.cmp(&b.1).reverse());
            book_counts
        };
        
        if let Some(winner) = determine_winner(&game_standings) {
            gameover(&winner);
            break;
        }
            
        let player_books_string: String = {
            let mut player_books_strings: Vec<String> = vec![];
            for (this_player, num_books) in game_standings.into_iter() {
                player_books_strings.push(format!("{}({})", this_player, num_books));
            }
            
            let mut player_books_string = String::from("Books: ");
            player_books_string.push_str(&player_books_strings.join(", "));
            player_books_string
        };
        println!("{}{}", termion::cursor::Goto(1, 2), player_books_string);
            
        let player_cards_string = player.get_card_labels().join(", ");
        println!("{}{}",
            termion::cursor::Goto(1, terminal_info.height / 2),
            player_cards_string);
            
        // print status lines
        for (pos, line) in current_status_lines.iter().enumerate() {
            println!("{}{}", 
                termion::cursor::Goto(1, terminal_info.height - 2 - (pos as u16)),
                line);
        }
        
        // clear the status lines
        current_status_lines.clear();
    
        let (next_player, next_opponent) = turn(
            &terminal_info,
            &mut current_status_lines,
            &mut deck,
            player,
            opponent
        );
        
        player = next_player;
        opponent = next_opponent;
    }
}

// FIXME: Pass Player structs instead of String
fn determine_winner<'a>(standings: &'a Vec<(&String, usize)>) -> Option<&'a String> {
    let mut total_book_count = 0;
    
    for (_player_name, book_count) in standings.iter() {
        total_book_count = total_book_count + *book_count;
    }
    
    if total_book_count >= 13 {
        return Some(&standings[0].0);
    }
    
    return None;
}

fn gameover(winner: &String) {
    println!("{}{} has won", termion::clear::All, winner);
}

fn turn(
    terminal_info: &TerminalInfo,
    current_status_lines: &mut Vec<String>,
    mut deck: &mut Vec<Card>,
    mut current_player: Player,
    mut current_opponent: Player,
) -> (Player, Player) {
    let card_face_value: u8 = {
        if current_player.is_cpu {
            let card_value = get_cpu_requesting_card_value(&current_player);
            current_status_lines.push(format!("Computer requested {}", card_value));
            card_value
        } else {
            get_player_requesting_card_value(&terminal_info)
        }
    };
        
    if card_face_value < 2 || card_face_value > 14 {
        current_status_lines.push(String::from("Invalid card face value"));
        return (current_player, current_opponent);
    }
    
    let mut has_card: bool = false;
    for &player_card in current_player.cards.iter() {
        if player_card.number == card_face_value {
            has_card = true;
            break;
        }
    }
    
    if !has_card && !current_player.is_cpu {
        current_status_lines.push(String::from("You dont have that card"));
        return (current_player, current_opponent);
    }
    
    let mut found_cards: Vec<Card> = vec![];
    
    for &other_player_card in current_opponent.cards.iter() {
        if other_player_card.number == card_face_value {
            found_cards.push(other_player_card);
        }
    }
    
    let found_cards_length = found_cards.len();
    
    if found_cards_length > 0 {
        current_status_lines.push(String::from("Opponent had card"));
        
        for card in found_cards.iter() {
            current_player.cards.push(*card);
        }
        
        found_cards.clear();
        
        current_opponent.cards.retain(|&card| card.number != card_face_value);
        return (current_player, current_opponent);
    }
    
    if deck.len() <= 0 {
        return (current_opponent, current_player);
    }
    
    current_status_lines.push(String::from("Go Fish"));
    give_random_card(&mut deck, &mut current_player, 1);
    
    return (current_opponent, current_player);
}

fn check_cards_for_books(current_status_cards: &mut Vec<String>, current_player: &mut Player) {
    let mut cards_count: HashMap<u8, u8> = HashMap::new();
    
    for &card in current_player.cards.iter() {
        *cards_count.entry(card.number).or_insert(0) += 1;
        
        if let Some(x) = cards_count.get(&card.number) {
            if *x >= 4 {
                // create a book of this.
                current_player.books.push(Book {
                    number: card.number,
                });
                
                let has_books_status = {
                    let mut str = String::new();
                    str.push_str(&current_player.name);
                    str.push_str(" now has ");
                    str.push(char::from_digit(current_player.books.len() as u32, 10).unwrap());
                    str.to_string()
                };
                current_status_cards.push(has_books_status);
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
