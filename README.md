## What?

A monte carlo simulation-based deck optimizer for Magic the Gathering.

## Why?

i dunno know

## How?

### How to run:

1. Install cargo.
2. Build this project `cargo build`.
3. Run this with cargo or via the stand alone exectuble. `cargo run -- -d <your deck>.json`.

### How to test

1. `cargo test`
2. To check coverage: `cargo llvm-cov --open`.


### Terminology

#### Strategies

The decision making process used during the game.
This is a pretty simplified version of how a human would actually play the game.
This approach uses heuristics and randomness to model the player behavior.

The strategies are responsible for
- choosing what land drops to make
- mulligan
- card plays

#### Metrics

Numbers and stats that measure how good or bad a game was.
For example, we can measure stats like "what turn, on average, is you commander played?"

There is a complex interaction with the player behavior here.

#### Trial

A single gold-fish run of the game.

#### Experiment

A number of trials ran.
Typically, this is to see the affect of some meta-parameter.

#### Scenario

Defines the meta-parameters and experiments to run.
For example: "what # of lands maximizes the speed at which I can burn out an opponent?"
