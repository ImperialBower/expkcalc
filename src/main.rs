use clap::Parser;
use pkcore::prelude::*;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long)]
    dealt: String,

    #[clap(short = 'b', long)]
    board: String,

    #[clap(short = 'n', long)]
    nuts: bool,
}

/// The goal of calc isn't to run a full simulation of play at a holdem poker table. It's
/// to provide a quick tool that can calculate odds and outs for a specific combination of hands.
///
/// NOTE ON PERSPECTIVE (double dummy)
///
/// We are taking the all knowing view of play, granted to us by modern poker TV shows, pioneered
/// by [Henry Orenstein](https://www.usbets.com/remembering-poker-pioneer-henry-orenstein/).
///
/// ## Step One
///
/// We want to be able to take the cards dealt, and display them representing the hole cards
/// for each of the players.
///
/// ## Step Two
///
/// Show me who has the best hand at the flop
///
/// The hand:
/// `❯ cargo run -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"`
///
/// To add logging:
/// RUST_LOG=trace cargo run -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
///
/// What about calling this hand The Fold?
/// RUST_LOG=trace cargo run -- -d  "5♠ 5♦ 9♠ 9♥ K♣ T♦" -b "5♣ 9♦ T♥ T♣ Q♦"
///
/// ## Step Three
///
/// Show me the winning percentages for each hand at the flop.
///
/// At this point I am starting to feel the strain on my system from my main method
/// trying to do too much. This is when I try to build code that will take the load
/// off and make things easier to maintain and build upon.
///
/// ## Step Four - Calc Structure
///
/// We're reaching the point in our code where the repl is doing to much...maintaining too
/// much state. Our `Game` struct was designed to simply hold all the cards that were needed
/// for the game.
///
/// For now, I want to get all the ducks in a row. Two things that I am missing:
/// * An ordered list of the possible hands at the flop.
/// * A collection of all types of possible hands for a player at the flop.
///
/// ## PHASE 3.1: Outs
///
/// Now that we have the win percentages displayed at the flop, we need to add the icing on the cake:
/// player outs. One of the clearest ways to display the meaning behind the odds is to show the
/// cards that the player behind on the hand would need in order to win.
///
/// Since our calc example is starting to take on a lot of business logic, this may be a good time
/// to do some refactoring and move it into dedicated structs.
///
/// Calculating win percentages and outs should be part of the same iteration through the possible
/// cases. I'm feeling the need to break this problem down with a spike in our example hear and
/// see where it leads us.
///
/// The structure that I am thinking to hold each of the player's outs is simple:
///
/// ```
/// #[derive(Clone, Debug, Default, Eq, PartialEq)]
/// pub struct Outs(Vec<Cards>);
/// ```
///
/// `cargo run -- -d "A♠ K♥ 8♦ 6♣" -b "A♣ 8♥ 7♥ 9♠ 5♠" -n`
///
/// Interesting hands:
/// cargo run -- -d "3♥ A♠ 5♥ A♦ 8♦ 7♦ K♥ K♠ 2♥ Q♠" -b "6♦ 6♣ 7♣ 9♦ 5♦" - Straight Flush at the river
/// cargo run -- -d "3♠ 9♦ J♠ 8♦ 2♠ Q♠ 6♣ 4♠" -b "Q♥ 5♥ 5♣ 7♥ 4♥" -- Two Pair vs Straight Draw
///
/// cargo run -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
/// cargo run -- -d "K♠ Q♠ 5♦ K♥ 5♥ J♥" -b "J♦ T♣ A♥ K♣ 2♣" -n -- Flopping the nuts
/// cargo run -- -d "A♣ Q♠ T♦ T♣ 6♦ 4♦ 2♥ 2♦" -b "J♦ J♠ J♥ A♥ 3♦" HSP S04E08 Harman/Safai
/// cargo run -- -d "T♦ 2♦ 9♠ 6♥" -b "3♠ 8♦ A♦" HSP S04E08 Elezra/Negreanu
/// cargo run -- -d "A♣ 4♠ K♥ 6♥ K♦ T♥" -b "7♠ 3♦ A♠ 4♦" HSP S04E08 Farha/Harman/Safai
/// cargo run -- -d "6♠ 6♦ A♣ Q♠ A♥ 9♥ Q♦ 5♠" -b "9♦ T♦ 6♥ T♥ K♠" HSP S04E08 Harman/Elezra
/// cargo run -- -d "T♠ 9♣ J♦ J♣ Q♥ T♣" -b "T♥ 7♣ A♥ J♠ 8♦" HSP S04E08 Harman/Elezra/Farha
/// cargo run -- -d "A♦ 7♦ T♠ T♥ K♦ K♥" -b "7♠ 6♥ 4♣" HSP S01E01 Negreanu/Buss/Nasseri
/// cargo run -- -d "A♠ J♦ 6♥ 6♣" -b "A♥ 3♠ 6♠ J♠ 5♠" HSP S01E01 Negreanu/Greenstein
/// cargo run -- -d "7♣ 6♥ K♣ 2♣ J♦ 9♦" -b "Q♣ 7♥ K♥ 6♣ Q♠" HSP S01E01 Alaei/Negreanu/Harman
/// cargo run -- -d "A♠ K♠ A♣ K♥" -b "4♠ 7♠ K♣" HSP S04E09 Hellmuth/Gold
/// cargo run -- -d "6♠ 4♠ 8♣ 6♣ A♦ 2♦ K♥ J♣" -b "2♣ 3♦ 3♣ 4♦ 4♣" HSP S06E10 Grospellier/Benyamine
/// cargo run -- -d "A♠ K♥ 9♦ 8♥" -b "6♦ 7♥ T♣ 3♥ 5♥" HSP S06E11 Galfond/Negreanu
/// cargo run -- -d "7♠ 6♠ Q♠ Q♦" -b "2♠ 7♥ 9♠ T♦ 4♣" HSP S08E07 Bellande Schwimer FIRST RUN
/// cargo run -- -d "7♠ 6♠ Q♠ Q♦" -b "2♠ 7♥ 9♠ A♠ K♠" HSP S08E07 Bellande Schwimer SECOND RUN
/// cargo run -- -d "T♦ 9♦ 2♠ 2♥" -b "2♦ T♥ 7♦ 8♦ 6♥" DNEGS https://youtu.be/yyPU25EGLkA?t=123
/// cargo run -- -d "A♦ Q♠ K♣ Q♦" -b "J♥ 9♠ A♣ 4♦ T♣" HSP S09E03 DNEGS/Bellands
/// cargo run -- -d "J♥ 8♠ K♠ J♠ 3♠ 3♥" -b "7♥ 8♦ 2♣ 5♣ Q♠" HSP S09E04 Adelstein/Liu/Antonius
/// cargo run -- -d "A♥ 8♦ K♣ 7♣ T♥ T♦" -b "4♠ K♦ 2♦ J♥ 3♠" HSP S09E05 Brunson/Tilly/Antonius
/// cargo run -- -d "J♥ J♣ A♥ 4♥" -b "3♣ 4♠ 4♣ 7♣ A♣" HSP S09E05 Adelstein/Brunson 1st
/// cargo run -- -d "J♥ J♣ A♥ 4♥" -b "3♣ 4♠ 4♣ 7♣ 9♠" HSP S09E05 Adelstein/Brunson 2nd
/// cargo run -- -d "8♦ 5♦ K♦ J♥ 2♠ 2♥" -b "9♥ 2♦ K♥ 4♥ J♠" HSP S09E05 Tilly/Hultman
/// cargo run -- -d "J♥ J♦ A♠ K♦ T♣ 9♣" -b "7♦ K♠ 2♥ 7♣ A♦" HSP S09E05 Liu/Tilly/Menon
/// cargo run -- -d "7s 6c js 4d" -b "8h 5h 9d" -- Hand with KDog
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let game = Game::new(
        HoleCards::from_str(&args.dealt)?,
        Board::from_str(&args.board)?,
    );

    println!("{}", game);

    println!();
    let flop_eval = FlopEval::try_from(game.clone()).unwrap();
    println!("{}", flop_eval);

    if args.nuts {
        println!();
        println!("The Nuts @ Flop:");
        println!("{}", game.board.flop.evals());
    }

    println!();
    println!();
    game.turn_display_odds()?;

    if args.nuts {
        game.turn_display_evals();
    }

    game.river_display_results();

    println!();
    println!("{}", command(game));

    println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}

fn command(game: Game) -> String {
    format!(
        "cargo run -- -d  \"{}\" -b \"{}\"",
        game.hands.cards(),
        game.board.cards()
    )
}
