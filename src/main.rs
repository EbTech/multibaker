use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::fmt;

struct State {
    // The current time step
    t: i32,
    // The current macrostate
    macrostate: i32,
    // A cache of the dice to roll in the future
    future_dice: Vec<i32>,
    // A cache of the dice to roll in the past
    past_dice: Vec<i32>,
    // Specifies how the macrostate evolves forward in time, given a die roll
    evolve_forward: Box<dyn Fn(i32, i32) -> i32>,
    // Specifies how the macrostate evolves backward in time, given a die roll
    // We must have evolve_backward(evolve_forward(x, r), r) = x for all x and r
    evolve_backward: Box<dyn Fn(i32, i32) -> i32>,
    // Specifies the initial state of all the dice
    roll_die: Box<dyn Fn(u64) -> i32>,
}

// Display the dice on both sides of the macrostate
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State at t={}: ...", self.t)?;
        for &die in &self.past_dice {
            write!(f, " {}", die)?;
        }
        write!(f, " ({}) ", self.macrostate)?;
        for &die in self.future_dice.iter().rev() {
            write!(f, "{} ", die)?;
        }
        write!(f, "...")
    }
}

impl State {
    // Create a new state with the given macrostate
    fn new(macrostate: i32) -> Self {
        let key = 0x1234_5678_9ABC_DEF0;
        State {
            t: 0,
            macrostate,
            future_dice: Vec::new(),
            past_dice: Vec::new(),
            evolve_forward: Box::new(|macrostate, dice| macrostate + dice),
            evolve_backward: Box::new(|macrostate, dice| macrostate - dice),
            roll_die: Box::new(move |t| ChaCha8Rng::seed_from_u64(t ^ key).random_range(0..6)),
        }
    }

    // Step the state forward in time
    fn step_forward(&mut self) {
        let die = self
            .future_dice
            .pop()
            .unwrap_or_else(|| (self.roll_die)(self.t as u64));
        self.macrostate = (self.evolve_forward)(self.macrostate, die);
        self.past_dice.push(die);
        self.t += 1;
    }

    // Step the state backward in time
    fn step_backward(&mut self) {
        self.t -= 1;
        let die = self
            .past_dice
            .pop()
            .unwrap_or_else(|| (self.roll_die)(self.t as u64));
        self.macrostate = (self.evolve_backward)(self.macrostate, die);
        self.future_dice.push(die);
    }
}

fn main() {
    let mut state = State::new(0);
    for _ in 0..10 {
        state.step_forward();
        println!("{}", state);
    }
    for _ in 0..10 {
        state.step_backward();
        println!("{}", state);
    }
}
