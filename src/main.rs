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
    // Specifies the initial state of all the dice
    roll_die: Box<dyn Fn(u64) -> i32>,
}

struct Transition {
    // Specifies how the macrostate evolves forward in time, given a die roll
    evolve_forward: Box<dyn Fn(i32, i32) -> i32>,
    // Specifies how the macrostate evolves backward in time, given a die roll
    // We must have evolve_backward(evolve_forward(x, r), r) = x for all x and r
    evolve_backward: Box<dyn Fn(i32, i32) -> i32>,
}

impl Transition {
    fn idle() -> Self {
        Transition {
            evolve_forward: Box::new(|macrostate, _| macrostate),
            evolve_backward: Box::new(|macrostate, _| macrostate),
        }
    }
    fn random_step() -> Self {
        Transition {
            evolve_forward: Box::new(|macrostate, dice| macrostate + dice),
            evolve_backward: Box::new(|macrostate, dice| macrostate - dice),
        }
    }
    fn record(val: i32) -> Self {
        Transition {
            evolve_forward: Box::new(move |macrostate, _| macrostate + val),
            evolve_backward: Box::new(move |macrostate, _| macrostate - val),
        }
    }
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
    fn uniform_rolls(microstate_seed: u64) -> Box<dyn Fn(u64) -> i32> {
        Box::new(move |t| ChaCha8Rng::seed_from_u64(t ^ microstate_seed).random_range(0..6))
    }

    // Create a new state with the given macrostate
    fn new(macrostate: i32) -> Self {
        let microstate_seed: u64 = rand::rng().random();
        State {
            t: 0,
            macrostate,
            future_dice: Vec::new(),
            past_dice: Vec::new(),
            roll_die: Self::uniform_rolls(microstate_seed),
        }
    }

    fn peturbed(old_state: &Self) -> Self {
        let microstate_seed: u64 = rand::rng().random();
        State {
            t: old_state.t,
            macrostate: old_state.macrostate,
            future_dice: Vec::new(),
            past_dice: Vec::new(),
            roll_die: Self::uniform_rolls(microstate_seed),
        }
    }

    // Step the state forward in time
    fn step_forward(&mut self, transition: &Transition) {
        let die = self
            .future_dice
            .pop()
            .unwrap_or_else(|| (self.roll_die)(self.t as u64));
        self.macrostate = (transition.evolve_forward)(self.macrostate, die);
        self.past_dice.push(die);
        self.t += 1;
    }

    // Step the state backward in time
    fn step_backward(&mut self, transition: &Transition) {
        self.t -= 1;
        let die = self
            .past_dice
            .pop()
            .unwrap_or_else(|| (self.roll_die)(self.t as u64));
        self.macrostate = (transition.evolve_backward)(self.macrostate, die);
        self.future_dice.push(die);
    }
}

fn main() {
    let mut walk = State::new(0);
    let mut memory = State::new(0);

    for t in 0..10 {
        if t == 5 {
            //walk.step_forward(&Transition::idle());
            memory.step_forward(&Transition::record(walk.macrostate));
        } else {
            walk.step_forward(&Transition::random_step());
            //memory.step_forward(&Transition::idle());
        }
        println!("{} {}", walk, memory);
    }

    for t in (0..10).rev() {
        if t == 5 {
            //walk.step_backward(&Transition::idle());
            memory.step_backward(&Transition::record(walk.macrostate));
        } else {
            walk.step_backward(&Transition::random_step());
            //memory.step_backward(&Transition::idle());
        }
        println!("{} {}", walk, memory);
    }
}
