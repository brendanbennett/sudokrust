use core::fmt;
use std::{collections::{HashMap, HashSet}, default, hash::Hash, iter::Chain, str::FromStr, vec};
use itertools::Itertools;

type Digit = u8;
type ValuesVec = Vec<(Coords, Square)>;

const DIGITS: [Digit; 9] = [1,2,3,4,5,6,7,8,9];

#[derive(Debug)]
enum SudokuError {
    RuntimeError(String),
    ParseError(String),
    InvalidState,
}

#[derive(Debug, Clone, PartialEq)]
enum Square {
    Value(Digit),
    Candidate(HashSet<Digit>),
}

impl Default for Square {
    fn default() -> Self {
        Square::Candidate(HashSet::from_iter(1..10))
    }
}

impl Square {
    fn new_with_value(d: Digit) -> Result<Self, SudokuError> {
        if d > 9 {
            Err(SudokuError::ParseError(format!("Value {} is not 1-9", d)))
        } else if d == 0 {
            Ok(Self::default())
        } else {
            Ok(Self::Value(d))
        }
    }

    fn remove_candidate(&mut self, value: Digit) {
        if let Self::Candidate(digits) = self {
            digits.remove(&value);
        }
    }

    fn has_candidate(&self, value: Digit) -> bool {
        if let Square::Candidate(candidates) = self {
            candidates.contains(&value)
        } else {
            false
        }
    }
}

impl TryFrom<char> for Square {
    type Error = SudokuError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let parsed = value.to_digit(10).ok_or(SudokuError::ParseError("Not a digit".to_string())).map(|x| x as Digit)?;
        Self::new_with_value(parsed)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", 
            match self {
                Self::Value(digit) => {format!("▒▒▒\n▒{}▒\n▒▒▒\n", digit)},
                Self::Candidate(candidates) => {
                    let mut square_print = String::new();
                    for i in 1..10 {
                        if candidates.contains(&i) {
                            square_print.push_str(&i.to_string());
                        } else {
                            square_print.push(' ');
                        }
                        if i % 3 == 0 {
                            square_print.push('\n')
                        }
                    }
                    format!("{square_print}")
                },
            }
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Coords {
    x: u8,
    y: u8
}

impl Coords {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x: x, y: y }
    }

    pub fn is_valid(&self) -> bool {
        if self.x > 8 || self.y > 8 {
            return false;
        }
        true
    }

    pub fn flat(&self) -> u8 {
        self.x + 9 * self.y
    }

    pub fn from_flat(flat_repr: u8) -> Self {
        Self {
            x: flat_repr % 9,
            y: flat_repr / 9,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Board (Vec<Square>);

impl Default for Board {
    fn default() -> Self {
        Board(vec![Square::default(); 81])
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            let mut buffers: [String; 3] = [String::new(), String::new(), String::new()];
            for col in 0..9 {
                let square_print = format!("{}", self.0[row * 9 + col]);
                let square_print_vec = square_print.split('\n').collect::<Vec<_>>();
                //println!("{:?}", square_print_vec.clone());
                
                for i in 0..3 {
                    buffers[i].push_str(square_print_vec[i]);
                    buffers[i].push(' ');
                }
            }
            for b in buffers {
                writeln!(f, "{b}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl FromStr for Board {
    type Err = SudokuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits: Vec<Square> = s.chars().map(|x| x.try_into()).collect::<Result<_,_>>()?;

        if digits.len() != 81 {
            Err(SudokuError::ParseError(format!("Digits have length {}, expected 81", digits.len())))
        } else {
            Ok(Board(digits))
        }
    }
}

impl Board {
    fn get_row(pos: Coords) -> Vec<Coords> {
        let mut neighbours: Vec<Coords> = Vec::new();

        for i in 0..9 {
            neighbours.push(Coords::new(i, pos.y));
        }
        neighbours
    }

    fn get_column(pos: Coords) -> Vec<Coords> {
        let mut neighbours: Vec<Coords> = Vec::new();

        for i in 0..9 {
            neighbours.push(Coords::new(pos.x, i));
        }
        neighbours
    }

    fn get_box(pos: Coords) -> Vec<Coords> {
        let mut neighbours: Vec<Coords> = Vec::new();
        let box_origin_x = pos.x / 3 * 3;
        let box_origin_y = pos.y / 3 * 3;

        for i in 0..3 {
            for j in 0..3 {
                let x = i + box_origin_x;
                let y = j + box_origin_y;
                neighbours.push(Coords::new(x, y));
            }
        }
        neighbours
    }

    fn get_neighbours(pos: Coords) -> Vec<Coords> {
        let mut neighbours: Vec<Coords> = Vec::new();

        for i in 0..9 {
            if i != pos.y {
                neighbours.push(Coords::new(pos.x, i));
            }
            if i != pos.x {
                neighbours.push(Coords::new(i, pos.y));
            }
        }

        let box_origin_x = pos.x / 3 * 3;
        let box_origin_y = pos.y / 3 * 3;

        for i in 0..3 {
            for j in 0..3 {
                let x = i + box_origin_x;
                let y = j + box_origin_y;
                if x == pos.x && y == pos.y {
                    continue;
                }
                let coords = Coords::new(x, y);
                if neighbours.contains(&coords) {
                    continue;
                }
                neighbours.push(Coords::new(x, y));
            }
        }
        neighbours
    }

    fn get_square_by_coords(&self, coords: &Coords) -> &Square {
        &self.0[coords.flat() as usize]
    }

    fn get_mut_square_by_coords(&mut self, coords: &Coords) -> &mut Square {
        &mut self.0[coords.flat() as usize]
    }

    fn set_square(&mut self, coords: &Coords, new_value: Square) -> bool {
        if new_value == self.0[coords.flat() as usize] {
            return false
        }
        self.0[coords.flat() as usize] = new_value;
        true
    }

    fn remove_visible_candidates(&mut self) {
        for i in 0..81 {
            if let Square::Value(value)  = self.0[i] {
                let coords = Coords::from_flat(i as u8);
                let neighbours = Self::get_neighbours(coords);
                for neighbour in neighbours {
                    self.get_mut_square_by_coords(&neighbour).remove_candidate(value);
                }
            }
        }
    }
    
    fn find_hidden_singles_in_coords_list(&self, coords_vec: &Vec<Coords>) -> ValuesVec {
        // println!("{:?}", coords_vec);
        let mut squares: Vec<&Square> = Vec::new();
        let mut singles: ValuesVec = Vec::new();

        for coords in coords_vec {
            squares.push(self.get_square_by_coords(coords))
        }

        for d in 1..10 {
            let mut occurances: usize = 0;
            let mut coords_buffer: Vec<Coords> = Vec::new();
            for (i, square) in squares.iter().enumerate() {
                // println!("digit {}, {:?}, occurs {}", d, square, occurances);
                if square.has_candidate(d) {
                    occurances += 1;
                    coords_buffer.push(coords_vec[i].clone())
                }
                if occurances > 1 {
                    break;
                }
            }
            if occurances == 1 {
                singles.push((coords_buffer[0].clone(), Square::Value(d)))
            }
        }
        // println!("{:?}", singles);
        singles
    }

    fn fill_hidden_singles(&mut self) -> Result<bool, SudokuError> {
        let mut singles: ValuesVec = Vec::new();
        for neighbourhood in self.clone().into_iter_full() {
            singles.extend(self.find_hidden_singles_in_coords_list(&neighbourhood));
        }
        
        self.fill_values_and_update_candidates(singles)
    }

    fn find_naked_singles(&self) -> ValuesVec {
        let mut singles: ValuesVec = ValuesVec::new();
        for i in 0..81 {
            match &self.0[i] {
                Square::Value(_) => (),
                Square::Candidate(candidates) => {
                    if candidates.len() == 1 {
                        let digit = *candidates.into_iter().next().unwrap();
                        singles.push((Coords::from_flat(i as u8), Square::Value(digit)));
                    }
                }
            }
        }
        singles
    }

    fn fill_naked_singles(&mut self) -> Result<bool, SudokuError> {
        self.fill_values_and_update_candidates(self.find_naked_singles())
    }

    fn is_digit_set_by_coords(&self, digit: Digit, coords_list: &Vec<Coords>) -> bool {
        for coords in coords_list {
            let square = self.get_square_by_coords(coords);
            if let Square::Value(set_digit) = square {
                if *set_digit == digit {
                    return true
                }
            }
        }
        false
    }

    fn is_candidate_set_at_coords(&self, digit: Digit, coords: &Coords) -> bool {
        let square = self.get_square_by_coords(coords);
        square.has_candidate(digit)
    }

    fn get_candidate_one_hot(&self, digit: Digit, coords_list: &Vec<Coords>) -> Vec<bool> {
        let mut one_hot: Vec<bool> = Vec::with_capacity(coords_list.len());
        for coords in coords_list {
            one_hot.push(self.is_candidate_set_at_coords(digit, coords));
        }
        one_hot
    }

    fn find_hidden_pairs(&self, neighbourhood: &Vec<Coords>) -> Vec<Digit> {
        let mut one_hots: HashMap<Digit, Vec<bool>> = HashMap::new();
        let mut hidden_digits: Vec<Digit> = Vec::new();

        for digit in DIGITS {
            one_hots.insert(digit, self.get_candidate_one_hot(digit, neighbourhood));
        }

        let mut paired_digits: Vec<Digit> = Vec::new();
        for (digit, one_hot) in &one_hots {
            let count = one_hot.into_iter().filter(|b| **b).count();
            if count == 2 {
                paired_digits.push(*digit)
            }
        }
        
        if paired_digits.len() >= 2 {
            let combinations = paired_digits.iter().combinations(2);
            for combination in combinations {
                let reduced_one_hots = one_hots.clone().into_iter()
                .filter(|(digit, _)| combination.contains(&digit))
                .map(|(_,a)| a.clone())
                .reduce(|oh1,oh2| (oh1.into_iter().zip(oh2).map(|(b1, b2)| b1 && b2)).collect())
                .unwrap();
                
                if reduced_one_hots.into_iter().filter(|b| *b).count() == 2 {
                    hidden_digits = combination.into_iter().map(|a| *a).collect();
                }
            }
        }
        hidden_digits
    }

    fn fill_values_and_update_candidates(&mut self, values: ValuesVec) -> Result<bool, SudokuError> {
        let mut was_changed = false;
        
        for (coords, value) in values {
            was_changed |= self.set_square(&coords, value);
        }
        
        if was_changed {
            self.remove_visible_candidates();
            if !self.verify() {
                return Err(SudokuError::InvalidState)
            }
        }
        Ok(was_changed)
    }

    fn verify(&self) -> bool {
        for neighbourhood in self.clone().into_iter_full() {
            if !self.verify_single_neighbourhood(&neighbourhood) {
                return false
            }
        }
        true
    }

    fn verify_single_neighbourhood(&self, coords_vec: &Vec<Coords>) -> bool {
        let mut values: Vec<Digit> = Vec::new();
        for coords in coords_vec {
            if let Square::Value(value) = self.get_square_by_coords(coords) {
                values.push(*value);
            }
        }
        let values_length = values.len();
        let values_set: HashSet<Digit> = values.into_iter().collect();
        if !values_set.is_subset(&(1..10).collect::<HashSet<Digit>>())
            || values_length != values_set.len() {
            return false
        }
        true
    }

    fn is_complete(&self) -> Result<bool, SudokuError> {
        if !self.verify() {
            return Err(SudokuError::InvalidState);
        }
        for i in 0..81 {
            if let Square::Candidate(_) = self.get_square_by_coords(&Coords::from_flat(i as u8)) {
                return Ok(false)
            }
        }
        return Ok(true)
    }

    pub fn solve(&mut self) {
        println!("{self}");
        self.remove_visible_candidates();
        println!("{self}");

        loop {
            if self.is_complete().unwrap() {
                println!("Solved.");
                break;
            }
            let mut was_changed = false;
            was_changed |= self.fill_hidden_singles().unwrap();
            println!("{self}");
            was_changed |= self.fill_naked_singles().unwrap();
            println!("{self}");
            
            if !was_changed {
                break;
            }
        }
    }

    fn show_coords_on_board(coords_list: Vec<Coords>) -> String {
        let mut out = String::new();
        for row in 0..9 {
            for col in 0..9 {
                if coords_list.contains(&Coords::new(col, row)) {
                    out.push('*');
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }
        out
    }

    fn into_iter_box(self) -> BoardBoxIterator {
        BoardBoxIterator {
            index: 0,
        }
    }
    
    fn into_iter_row(self) -> BoardRowIterator {
        BoardRowIterator {
            index: 0,
        }
    }

    fn into_iter_column(self) -> BoardColumnIterator {
        BoardColumnIterator {
            index: 0,
        }
    }

    fn into_iter_full(self) -> BoardFullIterator {
        BoardFullIterator::new(self)
    }
}

struct BoardBoxIterator {
    index: u8,
}

impl Iterator for BoardBoxIterator {
    type Item = Vec<Coords>;

    fn next(&mut self) -> Option<Self::Item> {
        let member_x = self.index % 3 * 3;
        let member_y = self.index / 3 * 3;
        self.index += 1;
        if self.index > 9 {
            return None
        }
        Some(Board::get_box(Coords::new(member_x, member_y)))
    }
}

struct BoardRowIterator {
    index: u8,
}

impl Iterator for BoardRowIterator {
    type Item = Vec<Coords>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= 8 {
            let result = Some(Board::get_row(Coords::new(0, self.index)));
            self.index += 1;
            return result;
        }
        None
    }
}

struct BoardColumnIterator {
    index: u8,
}

impl Iterator for BoardColumnIterator {
    type Item = Vec<Coords>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= 8 {
            let result = Some(Board::get_column(Coords::new(self.index, 0)));
            self.index += 1;
            return result;
        }
        None
    }
}

struct BoardFullIterator {
    iterator: Chain<BoardBoxIterator,Chain<BoardRowIterator, BoardColumnIterator>>
}

impl BoardFullIterator {
    fn new(board: Board) -> Self {
        Self {
            iterator: board.clone().into_iter_box().chain(
                board.clone().into_iter_row().chain(
                    board.clone().into_iter_column()
                )
            )
        }
    }
}

impl Iterator for BoardFullIterator {
    type Item = Vec<Coords>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
fn main() {
    let mut board = Board::from_str("000090500200010060050002004123000090060000000408501630005903000000000000600700008").unwrap();
    board.solve();
}
