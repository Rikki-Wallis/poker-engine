// betting rules, game state machine, pot logic
use std::collections::HashMap;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::card::Card;
use crate::deck::Deck;
use crate::player::Player;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Round {
    Start,
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    End
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ActionType {
    Fold,
    Check,
    Call,
    Raise,
    AllIn
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    In,
    Out,
    AllIn
}

// per hand player state
pub struct PlayerMetaData {
    status: Status,
    committed_street: i32,
    committed_total: i32
}

pub struct TableConfig {
    pub num_players: usize,
    pub sb_amount: i32,
    pub bb_amount: i32,
    pub buy_in: i32,
}


pub struct Action {
    action_type: ActionType,
    bet_size: Option<i32>, // relative to the current street 
    all_in_pot_size: Option<i32>,
}

impl Action {
    pub fn new(&mut self, action_type: ActionType, bet_size: Option<i32>, all_in_pot_size: Option<i32>) -> Self {
        Self {
            action_type: action_type,
            bet_size: bet_size,
            all_in_pot_size: all_in_pot_size
        }
    }
}


pub struct GameRunner {
    // cards
    community: Vec::<Card>,
    muck: Vec::<Card>,
    burn: Vec::<Card>,
    deck: Deck,

    // interface layer
    hole_cards: HashMap<i32, [Card;2]>,
    seats: Vec::<Player>,
    round_history: HashMap<Round, Vec::<Action>>,

    // meta
    table_config: TableConfig,
    current_round: Round,
    rng: StdRng,
    // indexes
    button_pos: usize,
    sb_pos: usize,
    bb_pos: usize,
    action_pos: usize,

    // money
    overall_pot: i32,
    street_pot: i32,
    bet_to_match: i32,

    // per-hand player state
    player_meta: HashMap<i32, PlayerMetaData>,
}
impl GameRunner {
    pub fn new(table_config: TableConfig, players: Vec<Player>, seed: u64) -> Self {
        Self {
            community: Vec::with_capacity(5),
            muck: Vec::with_capacity(18), // max 9 players
            burn: Vec::with_capacity(5),
            deck: Deck::new(3385958),

            hole_cards: HashMap::new(),
            seats: players,
            round_history: HashMap::new(),

            table_config: table_config,
            current_round: Round::Start,
            rng: StdRng::seed_from_u64(seed),

            button_pos: 0,
            sb_pos: 0,
            bb_pos: 0,
            action_pos: 0,

            overall_pot: 0,
            street_pot: 0,
            bet_to_match: 0,
            player_meta: HashMap::new(),
        }
    }


    pub fn deal(&mut self) {
        self.deck.shuffle(&mut self.rng);
        for player in self.seats.iter() {
            self.hole_cards.insert(player.id, [self.deck.next_card(), self.deck.next_card()]);
            self.player_meta.insert(player.id, PlayerMetaData {
                status: Status::In,
                committed_street: 0,
                committed_total: 0,
            });
        }
        self.button_pos = (self.button_pos + 1) % self.table_config.num_players - 1;
        self.sb_pos = (self.button_pos + 1) % self.table_config.num_players - 1;
        self.bb_pos = (self.sb_pos + 1) % self.table_config.num_players - 1;
        self.action_pos = (self.bb_pos + 1) % self.table_config.num_players - 1;
    }


    pub fn generate_legal_actions(&mut self, player: &Player ) -> [Option<ActionType>; 5] {
        // Generates possible actions for a player.
        // Assumes that a player given is currently in the round and hasnt folded yet.

        let mut legal: [Option<ActionType>; 5] = [None, None, None, None, None];
        let committed = self.player_meta[&player.id].committed_street;

        // fold:
        //      Someone has raised and you do not want to continue
        if committed < self.bet_to_match {
            legal[0] = Some(ActionType::Fold);
        }

        // check:
        //      No one has betted yet this round and it is not preflop
        if self.bet_to_match == 0 {
            legal[1] = Some(ActionType::Check);
        }

        // call:
        //      Someone has raised and you put more money in
        if self.bet_to_match > 0 && committed + player.money >= self.bet_to_match {
            legal[2] = Some(ActionType::Call);
        }

        // raise:
        //      If no one has bet this street, minimum of 2x the big blind
        //      If someone has bet in this street, 2x the bet to match
        let min_raise_amount = if self.bet_to_match == 0 {2*self.table_config.bb_amount} else {2*self.bet_to_match};
        if min_raise_amount <= player.money + committed {
            legal[3] = Some(ActionType::Raise);
        }

        // all-in:
        //      anytime you have money and have not folded
        //      if you do not have enough money to call a raise you can go all in instead
        legal[4] = Some(ActionType::AllIn);


        legal
    }


    pub fn settup_betting_round(&mut self) {
        self.overall_pot += self.street_pot;
        self.street_pot = 0;
        self.action_pos = (self.button_pos + 1) % self.table_config.num_players - 1;
    }

    pub fn handle_betting_round(&mut self) {

        let mut current_action_i = self.action_pos;

        // The number of live players who have not yet acted
        // since the last raise. A raise resets this value
        let live = |meta: &HashMap<i32, PlayerMetaData>| meta.values().filter(|m| m.status != Status::Out).count();
        let mut players_to_act = live(&self.player_meta);

        loop {
            // Hand is over if everyone else has folded
            if live(&self.player_meta) <= 1 {
                break;
            }

            if players_to_act == 0 {
                break;
            }

            let current_id = self.seats[current_action_i].id;
            let current_player_meta = self.player_meta.get_mut(&current_id).unwrap();

            // Do not let player play if they are out of the hand or have gone all in already
            let status = current_player_meta.status;
            if status == Status::Out || status == Status::AllIn {
                current_action_i = (current_action_i + 1) % self.table_config.num_players;
                continue;
            }

            // Get action
            let chosen_action: Action = self.seats[current_action_i].act();

            // Handle action
            match chosen_action.action_type {
                ActionType::Fold => {
                    current_player_meta.status = Status::Out;
                    players_to_act -= 1;
                }

                ActionType::Call => {
                    let committed = current_player_meta.committed_street;
                    let to_call = self.bet_to_match - committed;
                    self.street_pot += to_call;
                    self.seats[current_action_i].money -= to_call;
                    current_player_meta.committed_street += to_call;
                    current_player_meta.committed_total += to_call;
                    players_to_act -= 1;
                }

                ActionType::Check => {
                    players_to_act -= 1;
                }

                ActionType::Raise => {
                    let committed = current_player_meta.committed_street;
                    let raise_amount = chosen_action.bet_size.unwrap_or(self.bet_to_match*2);
                    let to_add = raise_amount - committed;
                    self.street_pot += to_add;
                    self.bet_to_match = raise_amount;
                    self.seats[current_action_i].money -= to_add;
                    current_player_meta.committed_street = raise_amount;
                    current_player_meta.committed_total += to_add;
                    players_to_act = live(&self.player_meta) - 1;
                }

                ActionType::AllIn => {
                    let remaining_money = self.seats[current_action_i].money;
                    self.street_pot += remaining_money;
                    self.seats[current_action_i].money = 0;
                    current_player_meta.committed_street += remaining_money;
                    current_player_meta.committed_total += remaining_money;
                    current_player_meta.status = Status::AllIn;
                    players_to_act -= 1;
                }
            }

            current_action_i = (current_action_i + 1) % self.table_config.num_players;
        }
    }

    pub fn handle_showdown(&mut self) {

    }

    pub fn play_game(&mut self) {
        
        loop {
            match self.current_round {
                Round::Start => {
                    self.deal();
                    self.current_round = Round::PreFlop;
                }

                Round::PreFlop => {
                    self.settup_betting_round();
                    self.handle_betting_round();
                    self.current_round = Round::Flop;
                }

                Round::Flop => {
                    self.settup_betting_round();
                    self.handle_betting_round();
                    self.current_round = Round::Turn;
                }

                Round::Turn => {
                    self.settup_betting_round();
                    self.handle_betting_round();
                    self.current_round = Round::River;
                }

                Round::River => {
                    self.settup_betting_round();
                    self.handle_betting_round();
                    self.current_round = Round::Showdown;
                }

                Round::Showdown => {
                    self.handle_showdown();
                    self.current_round = Round::End;
                }

                Round::End => {
                    // Handle giving each player money and resetting
                    self.current_round = Round::Start;
                }
            }
        }


    }
}
