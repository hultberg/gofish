use std::io;
use std::ops;
use std::collections::HashMap;
use rand::Rng;

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
    
    fn give_card(&mut self, card: &Card) {
        self.cards.push(card.clone());
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
    let max_books: usize = 13;
    let mut deck = generate_deck();
    assert_eq!(52, deck.len());
    
    // Welcome:
    println!("Welcome to Go Fish");
    println!("Please enter your name:");
    
    let mut player_name = String::new();
    io::stdin().read_line(&mut player_name).expect("Unable to read your name");
    let player_name: String = player_name.trim().to_string();
    
    // Create player instances.
    let mut player = Player::new(&player_name);
    let mut opponent = Player::new_cpu();
    
    give_random_card(&mut deck, &mut player, 7);
    give_random_card(&mut deck, &mut opponent, 7);
    
    loop {
        let (next_player, next_opponent) = turn(&mut deck, player, opponent);
        
        player = next_player;
        opponent = next_opponent;
        
        // create books.
        check_cards_for_books(&mut player);
        check_cards_for_books(&mut opponent);
    }
}

fn print_player_cards(player: &Player) {
    if player.cards.len() == 0 {
        println!("Player has no cards");
        return;
    }
    
    for card_label in player.get_card_labels().iter() {
        print!("{}, ", card_label);
    }
    println!("");
}

fn turn(
    mut deck: &mut Vec<Card>,
    mut current_player: Player,
    mut current_opponent: Player,
) -> (Player, Player) {
    println!("");
    println!("Its now {}'s turn", current_player.name);
    print_player_cards(&current_player);
    println!("");

    let card_face_value = get_requesting_card_value(&current_player);
    
    match card_face_value {
        60 => {
            print_player_cards(&current_player);
            return (current_player, current_opponent);
        },
        61 => {
            println!("Deck length: {}", deck.len());
            return (current_player, current_opponent);
        },
        _ => (),
    }
        
    if card_face_value < 2 || card_face_value > 14 {
        println!("Invalid card face value");
        return (current_player, current_opponent);
    }
    
    let mut found_cards: Vec<(usize, Card)> = vec![];
    
    for (pos, &other_player_card) in current_opponent.cards.iter().enumerate() {
        if other_player_card.number == card_face_value {
            found_cards.push((pos, other_player_card));
        }
    }
    
    let found_cards_length = found_cards.len();
    
    if found_cards_length > 0 {
        println!("DEBUG: found cards in other player");
        
        for (pos, card) in found_cards.iter() {
            current_player.cards.push(*card);
            current_opponent.cards.remove(*pos);
        }
        
        return (current_player, current_opponent);
    }
    
    if deck.len() <= 0 {
        println!("DEBUG: No more cards in deck");
        return (current_opponent, current_player);
    }
    
    println!("DEBUG: Nothing, Go Fish");
    give_random_card(&mut deck, &mut current_player, 1);
    
    return (current_opponent, current_player);
}

fn check_cards_for_books(current_player: &mut Player) {
    let mut cards_count: HashMap<CardRank, u8> = HashMap::new();
    
    for card in current_player.cards.iter() {
        *cards_count.entry(card.rank).or_insert(0) += 1;
        
        if let Some(x) = cards_count.get(&card.rank) {
            if *x >= 4 {
                // create a book of this.
                current_player.books.push(Book {
                    number: card.number,
                });
            }
        };
    }
    
    cards_count.retain(|&_k, &mut val| val >= 4);
    
    if cards_count.len() > 0 {
        println!("BEFORE:");
        print_player_cards(&current_player);
        current_player.cards.retain(|&card| {
            if cards_count.contains_key(&card.rank) {
                return *cards_count.get(&card.rank).unwrap() <= 4;
            }
            
            return true;
        });
        println!("AFTER:");
        print_player_cards(&current_player);
    }
}

fn get_requesting_card_value(player: &Player) -> u8 {
    if !player.is_cpu {
        println!("Enter a card face value to request (0-10, Jack, Queen, King, Ace):");
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
        
        return match command {
            "j" | "J" => 11,
            "q" | "Q" => 12,
            "k" | "K" => 13,
            "a" | "A" => 14,
            "?hand" => 60,
            "?deck_len" => 61,
            _ => 0,
        };
    }
    
    // Computer API
    let mut cards_count: HashMap<u8, u8> = HashMap::new();
    
    for (pos, card) in player.cards.iter().enumerate() {
        *cards_count.entry(card.number).or_insert(0) += 1;
    }
    
    let count_vec: Vec<_> = cards_count.into_iter().collect();
    if count_vec.len() > 0 {
        for (item, item2) in count_vec.iter() {
            println!("DEBUG: {}", item);
        }
        println!("DEBUG: Computer requests: {}", count_vec[0].0);
        return count_vec[0].0;
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
    for num in range {
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
