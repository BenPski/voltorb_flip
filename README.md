# voltorb_flip

A command line utility for solving games of Voltorb Flip. Important to note
that generally the game is not always solvable so it helps by pointing out safe
moves or if there are no safe moves then it approximates the odds of each tile
being a value and reports that to you.

Simple usage is clone the repo and use `cargo run` as the interface unless you
actually want to install it.

Takes one argument: the list of sums and zero counts going in row to column order.

For example an ingame board that looks like:
```
------------------
|0 |1 |2 |3 |1 | 7
|  |  |  |  |  | 1
------------------
|1 |1 |2 |1 |0 | 5
|  |  |  |  |  | 1
------------------
|2 |1 | 1|0 |0 | 4
|  |  |  |  |  | 2
------------------
|3 |1 |1 |1 |1 | 7
|  |  |  |  |  | 0
------------------
|2 |2 |2 |1 |0 | 7
|  |  |  |  |  | 1
------------------
| 8| 6| 8| 6| 2|
| 1| 0| 0| 1| 3|
```

translates to: `7,1,5,1,4,2,7,0,7,1,8,1,6,0,8,0,6,1,2,3`.

## Usage

Pass in the sum and zero values for the rows and columns
```
cargo run -- "7,1,5,1,4,2,7,0,7,1,8,1,6,0,8,0,6,1,2,3"
```

You'll then be given how much the program can simplify the board to and either:

 - the safe guesses to make
 - guesses with the approximated probability of them being a Voltorb, 1, 2, or 3 sorted in order of least likely to be a Voltorb
 - the completed board (up to positions where it is a toss up between Voltorb or 1 since they don't matter)

For your input: 

 - row, column, value
 - or q/quit to exit

All positions on the board are indexed starting at 0, so upper-left is 0, 0. 
Positions are also listed in row, column order.


