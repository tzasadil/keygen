# keygen

A Rust keyboard layout optimizer. It searches millions of layouts with simulated annealing and scores each one by how much strain it puts on your hands for a given text.

This is my fork of [xsznix/keygen](https://github.com/xsznix/keygen). I parallelized the search, rebuilt the report, extended the penalty model, and added corpora for more languages.

## Result: the layout I type on every day

keygen produced the layout I use daily for both English and Czech:

```
m l d p f | ' u o y k /
r n t s v | g a e i h x
z j w c b | q , . = ;
```

Consonants sit under the left hand and vowels under the right, so most words alternate hands.

Score is the average penalty per keystroke, lower is better.

| Corpus | QWERTY | My layout | Change |
|---|---|---|---|
| English | 0.663 | **0.313** | -53% |
| Czech | 0.919 | **0.394** | -57% |
| German | 0.909 | **0.428** | -53% |
| Python | 0.762 | **0.285** | -63% |

| Metric | QWERTY (en / cs) | My layout (en / cs) |
|---|---|---|
| Same-finger presses | 3.85% / 6.49% | 1.05% / 1.25% |
| Hand alternation | 71.6% / 67.4% | 81.3% / 83.7% |
| Inward / outward rolls (English) | 11.0% / 10.3% | 10.2% / 4.5% |
| Left / right hand load (English) | 45.9% / 34.2% | 40.0% / 40.1% |

Thumbs carry the remaining load, which is why the hand sum does not add to to 100. 

## What I added to the fork

- **Parallel search**: `rayon` runs one annealing chain per CPU core. A shared pool keeps the best layouts (3 per core). Each round, every core refines one pool entry, then the results are merged, sorted, and trimmed. 5 rounds of 205,000 cycles per chain.
- **Report**: for every penalty category, a table shows how often it fires, its average cost, and its share of the total. Below it are the load per finger and per hand, so you can see why a layout scores well, not only that it does.
- **Penalty model**:
  - Every switch between hands earns a small reward. Upstream only penalized three alternations in a row.
  - Retuned per-key base costs.
  - Thumbs count as their own hand, so the space bar does not distort the alternation and roll statistics.
  - Tracks load per finger and per hand.
- **Corpora**: English, Czech, German, and a Python programming book. You can optimize for the languages you actually type.
- **Maintenance**: builds on stable Rust. I removed the nightly-only `linked_list_extras` feature and updated `rand` from 0.3 to 0.7.

## Run

```sh
# Score reference layouts only
cargo run --release -- run-ref corpus/english.txt

# Score reference layouts, then search for better ones
cargo run --release -- run corpus/english.txt
```

With [just](https://github.com/casey/just): `just ref czech`, `just search german`, `just test`. Run `just` to list all recipes.

Sample report for QWERTY:

```
q w e r t | y u i o p -
a s d f g | h j k l ; '
z x c v b | n m , . /
total: 1890322.62; scaled: 0.6633

Name                           | % times |   Avg   | % Total  | Total
----------------------------------------------------------------------
Base                           | 100.00  | 0.305   | 45.938   | 868379
Same finger                    | 3.85    | 0.192   | 29.018   | 548540
Long jump                      | 0.98    | 0.028   | 4.154    | 78520
Roll reversal                  | 0.09    | 0.003   | 0.390    | 7371
Alternation                    | 71.61   | -0.072  | -10.797  | -204098
Roll out                       | 10.29   | -0.001  | -0.155   | -2934
Roll in                        | 11.02   | -0.109  | -16.448  | -310926
----------------------------------------------------------------------

 6.7   7.1  15.2  16.9  | 17.1   6.3   9.6   1.2     <- load per finger
45.9 | 34.2                                           <- load per hand
```

## Original README

The text below is from the upstream project. Some penalty weights differ in this fork.

This code follows the simulated annealing method used in [Carpalx](http://mkweb.bcgsc.ca/carpalx/?simulated_annealing), but with a different model. Here, we try to maximise comfort by minimising the actions that cause discomfort---stretching or compressing the hand in uncomfortable ways. In addition to the base cost of the key due to its position, we also include:

* A penalty for using the same finger twice on different keys. Example (QWERTY): ED/DE, LO. Using the same finger twice is the second slowest thing you can do on a keyboard. An extra penalty is awarded if one of the keys in the combination is in a centre column, since lateral movements are slower.
* A penalty for jumping from the top row to the bottom row or from the bottom row to the top row on the same finger. Example: CE, UN. Jumping across the home row is the slowest thing you can do on a keyboard.
* A penalty for jumping from top to bottom row or from bottom to top row on consecutive fingers, excluding middle--index. Example: EX. This isn't as bad as jumping the home row on the same finger, but causes your fingers to awkwardly stretch to reach the second key.
* A penalty for jumping from top to bottom row or from bottom to top row on the same hand. Such a jump, even if not on the same or consecutive fingers, causes your hands to bend awkwardly.
* A penalty for an awkward pinky/ring combination where the pinky reaches above the ring finger. Example: QA/AQ, PL/LP, ZX/XZ. Since the pinky is longer than the ring finger, this causes your hand to awkwardly compress.
* A penalty for reversing a roll at the end of the hand, i.e. using the ring, pinky, then middle finger of the same hand. Examples: WAD. Since the movement of the ring finger is partially dependent on that of the middle finger, this motion is particularly tricky and therefore inaccurate. In Dvorak, typing "install" may sometimes result in "instnall" or "insntall" as a result of this dependency.
* A penalty for using the same hand four times in a row. Examples: EVER, WERE, LOOK. Using the same hand for too many letters in a row fatigues the hand creates the opportunity for error.
* A penalty for alternating hands three times in a row. Examples: WITH, IGHT, WHEN. Alternating too often may cause the timing of the alternation to fall apart, for example resulting in "teh" or "hte" for "the".
* A slight penalty for rolling outwards, accompanied by a slight award (negative penalty) for rolling inwards, since a rolling in motion feels more natural than a rolling out motion.

### Installing and running

You'll need a recent-ish version of [Rust](https://www.rust-lang.org/).

Then: `cargo run --release -- run corpus/english.txt`.

### Installing the (upcoming) optimal keyboard layout

If you're crazy enough to want to try this, you're probably smart enough to figure out how to install custom keyboards on your system of choice.

### Credits

The simulated annealing algorithm and corpus are taken from Carpalx by Martin Krzywinski.

### Other alternate keyboard layouts

mdickens has a good list of them [here](http://mdickens.me/typing/alternative_layouts.html).

### Licence

MIT
