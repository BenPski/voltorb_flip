/*
 * to go from a game to the possible games at the most reduced form
 * expand the options from makign one selection at a time and check the constraints game -> [game, game, game] 
 * if game is fully known and passes the constraints then don't need to keep expanding 
 */
use core::fmt;
use std::str::FromStr;
use std::fmt::Display;

use rand::{thread_rng, seq::SliceRandom};

use std::env;

#[derive(Debug, Clone, PartialEq, Eq) ]
struct Grid {
    grid: Vec<Box<GCell>>,
}

impl Grid {
    fn new() -> Self {
        Grid { grid: (0..25).map(|_| { Box::new(GCell::new()) }).collect() }
    }

    fn row(&self, row: usize) -> Vec<Box<GCell>> {
        let grid = &self.grid;
        grid[(row*5)..((row+1)*5)].to_vec()
    }

    fn col(&self, col: usize) -> Vec<Box<GCell>> {
        let grid = &self.grid;
        let mut arr = Vec::new();
        (0..5).for_each(|i| {
            arr.push(grid[col + 5*i].to_owned());
        });
        arr
    }

    fn set(&self, row: usize, col: usize, val: u8) -> Self {
        if row < 5 && col < 5 {
            let mut new = self.clone();
            new.grid[5*row + col].set(val);
            new
        } else {
            self.clone()
        }
    }

    fn cell(&self, row: usize, col: usize) -> Box<GCell> {
        self.grid[5*row + col].to_owned()
    }

    // the only unknown cells consist of 0, 1
    fn complete(&self) -> bool {
        for cell in &self.grid {
            if !cell.is_known() {
                if cell.has(2) || cell.has(3) {
                    return false;
                }
            }
        }
        return true;
    }
}

// [0,1,2,3] or a subset of that
#[derive(Debug, Clone, PartialEq, Eq)]
struct GCell {
    val: Vec<u8>,
}

impl GCell {
    fn new() -> Self {
        GCell { val: vec![0, 1, 2, 3] }
    }

    fn empty() -> Self {
        GCell { val: Vec::new() }
    }

    fn set(&mut self, item: u8) {
        if self.val.contains(&item) {
            self.val = vec![item]
        }
    }

    fn has(&self, item: u8) -> bool {
        self.val.contains(&item)
    }

    fn is_known(&self) -> bool {
        self.val.len() == 1
    }

    fn intersect(&self, other: &GCell) -> GCell {
        let mut new = Vec::new();
        for x in &self.val {
            if other.has(*x) {
                new.push(*x);
            }
        }
        GCell { val: new }
    }

    fn union(&self, other: &GCell) -> GCell {
        let mut new = self.val.clone();
        for x in &other.val {
            if !new.contains(x) {
                new.push(*x)
            }
        }
        GCell { val: new }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Constraint {
    sum: u8,
    zeros: u8,
}

impl Constraint {
    fn new(sum: u8, zeros: u8) -> Self {
        Constraint { sum, zeros }
    }

    // constraints from some input vector
    fn from_list(arr: Vec<u8>) -> Option<Vec<Self>> {
        if arr.len() == 20 {
            let mut res = Vec::new();
            for i in 0..10 {
                res.push(Constraint::new(arr[2*i], arr[2*i + 1]));
            }
            Some(res)
        } else {
            None
        }
    }

    // constraint from a row or column of a board
    fn from_vec(arr: Vec<u8>) -> Self {
        let mut zeros = 0;
        for item in &arr {
            if item == &0 {
                zeros += 1;
            }
        }
        Constraint { sum: arr.into_iter().sum(), zeros }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Game {
    grid: Grid,
    constraints: Vec<Constraint>,
}

impl Game {
    // game is defined in terms of the contraints
    fn new(constraints: Vec<Constraint>) -> Self {
        Game { grid: Grid::new(), constraints }
    }

    fn from_sections(row_sections: &[Section], col_sections: &[Section]) -> Self {
        let mut new_board = Vec::new();
        let mut constraints = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let row_tile = &row_sections[row].section[col];
                let col_tile = &col_sections[col].section[row];
                new_board.push(Box::new(row_tile.intersect(&col_tile)));
            }
        }
        for row in 0..5 {
            constraints.push(row_sections[row].constraint);
        }
        for col in 0..5 {
            constraints.push(col_sections[col].constraint);
        }
        Game { grid: Grid { grid: new_board }, constraints }
    }
    
    fn set(&self, row: usize, col: usize, val: u8) -> Self {
        Game { grid: self.grid.set(row, col, val), constraints: self.constraints.clone() }
    }

    fn safe(&self) -> Vec<(usize, usize)> {
        let mut guesses = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let cell = self.grid.cell(row, col);
                if !cell.is_known() && !cell.has(0) {
                    guesses.push((row, col));
                }
            }
        }
        guesses
    }

    fn approximate_odds(&self) -> Vec<(usize, usize, [f32; 4])> {
        let (row_sections, col_sections) = self.sections();
        let mut stats = Vec::new();
        let mut row_stats = Vec::new();
        let mut col_stats = Vec::new();
        for row in row_sections {
            row_stats.push(row.stats());
        }
        for col in col_sections {
            col_stats.push(col.stats());
        }
        for i in 0..5 {
            for j in 0..5 {
                let r = row_stats[i][j];
                let c = col_stats[j][i];
                let s = combine_stats(r, c);
                stats.push((i, j, s));
            }
        }
        stats
    }

    fn sections(&self) -> (Vec<Section>, Vec<Section>) {
        let mut row_sections = Vec::new();
        let mut col_sections = Vec::new();
        for i in 0..5 {
            row_sections.push(Section::new(self.grid.row(i), self.constraints[i]));
        }
        for i in 0..5 {
            col_sections.push(Section::new(self.grid.col(i), self.constraints[5+i]));
        }
        (row_sections, col_sections)
    }

    // reduce the game grid based on the contraints
    fn simplify(&self) -> Self {
        let (mut row_sections, mut col_sections) = self.sections();
        row_sections = row_sections.into_iter().map(|x| {
            x.simplify()
        }).collect();
        col_sections = col_sections.into_iter().map(|x| {
            x.simplify()
        }).collect();

        Self::from_sections(&row_sections, &col_sections)   
    }

    fn simplify_complete(&self) -> Self {
        let next = self.simplify();
        if next == *self {
            next
        } else {
            next.simplify()
        }
    }

}

impl Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
         * Goal
         * ----------------
         * |01|01|01|01|01|
         * |23|23|23|23|23|
         * ----------------
         * |01|01|01|01|01|
         * |23|23|23|23|23|
         * ----------------
         * |01|01|01|01|01|
         * |23|23|23|23|23|
         * ----------------
         * |01|01|01|01|01|
         * |23|23|23|23|23|
         * ----------------
         * |01|01|01|01|01|
         * |23|23|23|23|23|
         *
         * probably a way better way of doing this
         */
        let mut lines = Vec::new();
        let bar = String::from("----------------");
        for i in 0..5 {
            lines.push(bar.clone());
            let mut line1 = String::new();
            let mut line2 = String::new();
            let row = self.row(i);
            for elem in row {
                line1.push('|');
                line2.push('|');
                if elem.has(0) { line1.push('0') } else { line1.push(' ') };
                if elem.has(1) { line1.push('1') } else { line1.push(' ') };
                if elem.has(2) { line2.push('2') } else { line2.push(' ') };
                if elem.has(3) { line2.push('3') } else { line2.push(' ') };
            }
            line1.push('|');
            line2.push('|');
            lines.push(line1);
            lines.push(line2);
        }
        lines.push(bar);
        write!(f, "{}", lines.join("\n"))
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
         * Goal
         * ------------------
         * |01|01|01|01|01|##
         * |23|23|23|23|23|##
         * ------------------
         * |01|01|01|01|01|##
         * |23|23|23|23|23|##
         * ------------------
         * |01|01|01|01|01|##
         * |23|23|23|23|23|##
         * ------------------
         * |01|01|01|01|01|##
         * |23|23|23|23|23|##
         * ------------------
         * |01|01|01|01|01|##
         * |23|23|23|23|23|##
         * ------------------
         * |##|##|##|##|##|
         * |##|##|##|##|##|
         *
         * probably a way better way of doing this
         */
        let constraints = &self.constraints;
        let mut lines = Vec::new();
        let bar = String::from("------------------");
        for i in 0..5 {
            lines.push(bar.clone());
            let mut line1 = String::new();
            let mut line2 = String::new();
            let row = self.grid.row(i);
            for elem in row {
                line1.push('|');
                line2.push('|');
                if elem.has(0) { line1.push('0') } else { line1.push(' ') };
                if elem.has(1) { line1.push('1') } else { line1.push(' ') };
                if elem.has(2) { line2.push('2') } else { line2.push(' ') };
                if elem.has(3) { line2.push('3') } else { line2.push(' ') };
            }
            line1 = format!("{}|{:2}",line1, constraints[i].sum);
            line2 = format!("{}|{:2}",line2, constraints[i].zeros);
            lines.push(line1);
            lines.push(line2);
        }
        lines.push(bar);
        let mut line1 = String::new();
        let mut line2 = String::new();
        for i in 0..5 {
            line1 = format!("{}|{:2}", line1, constraints[i+5].sum);
            line2 = format!("{}|{:2}", line2, constraints[i+5].zeros);
        }
        line1.push('|');
        line2.push('|');
        lines.push(line1);
        lines.push(line2);
        write!(f, "{}", lines.join("\n"))
    }
}

fn constraints_from_board(board: Vec<u8>) -> Vec<Constraint> {
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut constraints = Vec::new();
    for i in 0..5 {
        rows.push(board[i*5..(i+1)*5].to_vec());
        cols.push((0..5).map(|x| board[i + 5*x]).collect());
    }
    for row in rows {
        constraints.push(Constraint::from_vec(row));
    }
    for col in cols {
        constraints.push(Constraint::from_vec(col));
    }
    constraints
}

#[derive(Debug, Clone)]
struct Section {
    section: Vec<Box<GCell>>,
    constraint: Constraint,
}

impl Section {
    fn new(section: Vec<Box<GCell>>, constraint: Constraint) -> Self {
        Section { section, constraint } 

    }

    fn stats(&self) -> Vec<[f32; 4]> {
        let mut stats = vec![
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
        ];
        let sols = solutions(self.section.clone(), self.constraint.clone());
        for sol in &sols {
            for i in 0..5 {
                let val = sol[i].val[0];
                stats[i][val as usize] += 1.0;
            }
        }
        let mut odds = vec![
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
            [0.0,0.0,0.0,0.0],
        ];
        for i in 0..5 {
            for j in 0..4 {
                odds[i][j] = stats[i][j]/stats[i].into_iter().sum::<f32>();
            }
        }
        odds
    }

    // compute possibilities and combine them to one solution
    fn simplify(&self) -> Self {
        let mut res = self.section.iter().map(|_| Box::new(GCell::empty())).collect();

        let sols = solutions(self.section.clone(), self.constraint.clone());
        for sol in sols {
            res = combine(res, sol);
        }
        Section { section: res, constraint: self.constraint }
    }
}

fn combine(left: Vec<Box<GCell>>, right: Vec<Box<GCell>>) -> Vec<Box<GCell>> {
    let mut res = Vec::new();
    for i in 0..left.len() {
        res.push(Box::new(left[i].union(&right[i])));
    }
    res
}

fn combine_stats(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    let mut new = [0.0,0.0,0.0,0.0];
    for i in 0..4 {
        new[i] = left[i]*right[i];
    }

    let n: f32 = new.into_iter().sum();
    for i in 0..4 {
        new[i] = new[i]/n;
    }
    new
}

fn solutions(section: Vec<Box<GCell>>, constraint: Constraint) -> Vec<Vec<Box<GCell>>> {
    if section.len() == 0 {
        return Vec::new();
    } else if section.len() == 1 {
        let vals = &section[0].val;
        let sum = constraint.sum;
        let zeros = constraint.zeros;
        return match (sum, zeros) {
            (0, 0) => Vec::new(),
            (s, 0) => if vals.contains(&s) {
                vec![vec![Box::new(GCell { val: vec![s] })]]
            } else {
                Vec::new()
            },
            (0, 1) => if vals.contains(&0) {
                vec![vec![Box::new(GCell { val: vec![0] })]]
            } else {
                Vec::new()
            },
            _ => Vec::new()
        }
    } else {
        let cell = section.clone().pop().expect("oh geez");
        if cell.is_known() {
            let mut new_section = section.clone();
            let mut res = Vec::new();
            new_section.pop();
            let val = cell.val[0];
            if val > constraint.sum {
                return Vec::new();
            }
            if val == 0 && constraint.zeros == 0 {
                return Vec::new();
            }
            let new_sum = constraint.sum - cell.val[0];
            let new_zero = constraint.zeros - if cell.val[0] == 0 { 1 } else { 0 };
            let sols = solutions(new_section, Constraint::new(new_sum, new_zero));
            for mut sol in sols {
                sol.push(cell.clone());
                res.push(sol);
            }
            return res;
        } else {
            let mut res = Vec::new();
            for val in cell.val {
                let mut new_section = section.clone();
                new_section.pop();
                new_section.push(Box::new(GCell { val: vec![val] }));
                let sols = solutions(new_section, constraint.clone());
                res.extend(sols);
            }
            return res;
        }
    }
}

// read the user input for selecting a row, column, and value
// assume no preprocessing
// format is "row, col, val"
fn parse_input(input: &str) -> Option<(usize, usize, u8)> {
   let vals: Vec<&str> = input.trim().split(",").collect();
   if vals.len() == 3 {
       let row = vals[0].trim().parse().ok()?;
       let col = vals[1].trim().parse().ok()?;
       let val = vals[2].trim().parse().ok()?;
       Some((row, col, val))
   } else {
       None
   }
}

fn quit_input(input: &str) -> bool {
    let val = input.trim();
    match val {
        "q" => true,
        "quit" => true,
        _ => false,
    }
}

fn game_loop(board: Game) {
    let mut board = board.simplify();
    loop {
        println!("\n\n");
        if board.grid.complete() {
            println!("Completed board:\n{}", board);
            break;
        }
        println!("Board:\n{}", board);
        let safe = board.safe();
        if safe.len() > 0 {
            println!("Safe plays: {:?}", safe);
        } else {
            let mut odds = board.approximate_odds();
            odds = odds.into_iter().filter(|x| !board.grid.cell(x.0, x.1).is_known()).collect();
            odds.sort_by(|a, b| a.2[0].partial_cmp(&b.2[0]).unwrap()); 
            println!("choices: {:?}", odds);
        }
        if let Ok(val) = rprompt::prompt_reply("Set value [row, col, val]: ") {
            if quit_input(&val) {
                println!("Exiting");
                break;
            }
            if let Some((row, col, val)) = parse_input(&val) {
                board = board.set(row, col, val);
                board = board.simplify_complete();
            } else {
                println!("Invalid input {:?}", val);
            }
        } else {
            println!("Couldn't parse input")
        }
    }
}

fn parse_list<F: FromStr>(input: &str) -> Result<Vec<F>, F::Err> {
    let vals = input.trim().split(",");
    let mut res = Vec::new();
    for val in vals {
        let item = val.trim().parse()?;
        res.push(item);
    }
    Ok(res)
}

fn main() {
    /*
    let board = vec![0,1,2,3,1,
                     1,1,2,1,0,
                     2,1,1,0,0,
                     3,1,1,1,1,
                     2,2,2,1,0]
    */

    // if you want a random game
    // although this is fundamentally a different experience because the answer
    // is known so it is emulating the real game and should probably be treated
    // like that

    let mut random_board = Vec::new();
    let choices = vec![0,1,2,3];
    let mut rng = thread_rng();
    for _ in 0..25 {
        let val = choices.choose(&mut rng).unwrap();
        random_board.push(*val);
    }
    let constraints = constraints_from_board(random_board);
    let _board = Game::new(constraints);
    //game_loop(board);


    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Pass in a single string for the game to play");
        return;
    }
    //let cons_string = "7,1,5,1,4,2,7,0,7,1,8,1,6,0,8,0,6,1,2,3";
    let cons_string = &args[1];

    if let Ok(cons) = parse_list(&cons_string) {
        if let Some(constraints) = Constraint::from_list(cons) {
            let board = Game::new(constraints);
            game_loop(board);
        } else {
            println!("Invalid constraints string, should be a comma separated list of 20 numbers. row1 sum, row1 zeros, row2 sum, row2 zeros,...,col5 sum, col5 zeros");
        }
    } else {
        println!("Invalid constraints string, should be a comma separated list of 20 numbers. row1 sum, row1 zeros, row2 sum, row2 zeros,...,col5 sum, col5 zeros")
    }

}
