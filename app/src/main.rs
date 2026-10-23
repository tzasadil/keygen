//! Browser playground: score text on two keyboard layouts side by side, and
//! swap keys to see the scores change.

use gloo_net::http::Request;
use keygen_core::layout::{self, Layout};
use keygen_core::penalty::{
    Penalty, PenaltyKind, QuartadList, calculate_penalty, normalize_text, prepare_quartad_list,
};
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Corpus {
    English,
    Czech,
    German,
    Python,
}

impl Corpus {
    const ALL: [Corpus; 4] = [Corpus::English, Corpus::Czech, Corpus::German, Corpus::Python];

    fn name(self) -> &'static str {
        match self {
            Corpus::English => "English",
            Corpus::Czech => "Czech",
            Corpus::German => "German",
            Corpus::Python => "Python",
        }
    }

    // Trunk copies `corpus/` next to index.html.
    fn path(self) -> &'static str {
        match self {
            Corpus::English => "corpus/english.txt",
            Corpus::Czech => "corpus/czech.txt",
            Corpus::German => "corpus/german.txt",
            Corpus::Python => "corpus/py.txt",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Source {
    Corpus(Corpus),
    Pasted,
}

#[derive(Clone, Copy, PartialEq)]
enum Preset {
    Qwerty,
    Dvorak,
    Colemak,
    Mine,
}

impl Preset {
    const ALL: [Preset; 4] = [Preset::Qwerty, Preset::Dvorak, Preset::Colemak, Preset::Mine];

    fn name(self) -> &'static str {
        match self {
            Preset::Qwerty => "QWERTY",
            Preset::Dvorak => "Dvorak",
            Preset::Colemak => "Colemak",
            Preset::Mine => "My layout",
        }
    }

    fn layout(self) -> &'static Layout {
        match self {
            Preset::Qwerty => &layout::QWERTY_LAYOUT,
            Preset::Dvorak => &layout::DVORAK_LAYOUT,
            Preset::Colemak => &layout::COLEMAK_LAYOUT,
            Preset::Mine => &layout::MY_LAYOUT,
        }
    }
}

#[derive(Clone, PartialEq)]
enum Loaded<T> {
    Loading,
    Failed(String),
    Ready(T),
}

// What the page shows below the controls. Kept apart from the scores, so a key
// swap updates values in place instead of rebuilding the results (which would
// also drop keyboard focus).
#[derive(Clone, PartialEq)]
enum Status {
    Loading,
    Failed(String),
    // Text under 3 characters, or every trigram holds a character without a key.
    Empty,
    Ready,
}

// One compared layout: a preset, plus any swaps made on top of it.
#[derive(Clone, Copy)]
struct Side {
    preset: RwSignal<Preset>,
    layout: RwSignal<Layout>,
    // First key of a pending swap.
    selected: RwSignal<Option<usize>>,
}

impl Side {
    fn new(preset: Preset) -> Side {
        Side {
            preset: RwSignal::new(preset),
            layout: RwSignal::new(preset.layout().clone()),
            selected: RwSignal::new(None),
        }
    }

    fn edited(self) -> bool {
        self.layout.with(|layout| layout != self.preset.get().layout())
    }

    fn name(self) -> String {
        let name = self.preset.get().name();
        if self.edited() { format!("{name} (edited)") } else { name.to_string() }
    }

    fn choose(self, preset: Preset) {
        self.preset.set(preset);
        self.layout.set(preset.layout().clone());
        self.selected.set(None);
    }

    fn click_key(self, pos: usize) {
        match self.selected.get() {
            None => self.selected.set(Some(pos)),
            Some(first) if first == pos => self.selected.set(None),
            Some(first) => {
                self.layout.update(|layout| layout.swap(first, pos));
                self.selected.set(None);
            }
        }
    }
}

// Both sides score the same text, so they are always present together.
type Scores = Memo<Option<[Penalty; 2]>>;

fn share(penalty: &Penalty, count: i64) -> f64 {
    count as f64 / penalty.len as f64
}

fn per_keystroke(penalty: &Penalty) -> f64 {
    penalty.total / penalty.len as f64
}

// Results only render while scores exist, so the 0.0 fallback never shows.
fn read(scores: Scores, side: usize, value: impl Fn(&Penalty) -> f64) -> f64 {
    scores.with(|scores| scores.as_ref().map_or(0.0, |scores| value(&scores[side])))
}

async fn fetch_text(path: &str) -> Result<String, String> {
    let response = Request::get(path).send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.text().await.map_err(|e| e.to_string())
}

#[component]
fn App() -> impl IntoView {
    let source = RwSignal::new(Source::Corpus(Corpus::English));
    let pasted = RwSignal::new(String::new());
    let sides = [Side::new(Preset::Qwerty), Side::new(Preset::Mine)];

    // Tagged with its corpus: a resource keeps its old value while it reloads,
    // so an untagged value could score English text under the Czech label.
    let corpus_text = LocalResource::new(move || {
        let source = source.get();
        async move {
            match source {
                Source::Corpus(corpus) => Some((corpus, fetch_text(corpus.path()).await)),
                Source::Pasted => None,
            }
        }
    });

    // Separate from the scores, so switching layouts skips the trigram count.
    // TODO: move to a Web Worker with the live search. The English corpus
    // blocks the page for a moment while it is counted.
    let quartads = Memo::new(move |_| -> Loaded<QuartadList> {
        match source.get() {
            Source::Pasted => Loaded::Ready(prepare_quartad_list(&normalize_text(&pasted.read()))),
            Source::Corpus(corpus) => match corpus_text.read().as_ref() {
                Some(Some((loaded, Ok(text)))) if *loaded == corpus => {
                    Loaded::Ready(prepare_quartad_list(&normalize_text(text)))
                }
                Some(Some((loaded, Err(error)))) if *loaded == corpus => {
                    Loaded::Failed(error.clone())
                }
                _ => Loaded::Loading,
            },
        }
    });

    let penalties = Memo::new(move |_| {
        quartads.with(|quartads| match quartads {
            Loaded::Loading => Loaded::Loading,
            Loaded::Failed(error) => Loaded::Failed(error.clone()),
            Loaded::Ready(quartads) => {
                let penalties = sides.map(|side| {
                    side.layout.with(|layout| calculate_penalty(quartads, layout).penalty)
                });
                // Every preset maps all letters, space, and newline, and swaps
                // keep that, so either side being empty means both are.
                Loaded::Ready((penalties[0].len > 0).then_some(penalties))
            }
        })
    });
    let status = Memo::new(move |_| {
        penalties.with(|penalties| match penalties {
            Loaded::Loading => Status::Loading,
            Loaded::Failed(error) => Status::Failed(error.clone()),
            Loaded::Ready(None) => Status::Empty,
            Loaded::Ready(Some(_)) => Status::Ready,
        })
    });
    let scores: Scores = Memo::new(move |_| {
        penalties.with(|penalties| match penalties {
            Loaded::Ready(scores) => scores.clone(),
            _ => None,
        })
    });

    view! {
        <main>
            <header>
                <h1>"keygen"</h1>
                <p class="tagline">"Keyboard layout playground"</p>
            </header>
            <section class="controls">
                <SourcePicker source pasted />
                <div class="layout-pickers">
                    <LayoutPicker label="Layout A" side=sides[0] />
                    <LayoutPicker label="Layout B" side=sides[1] />
                </div>
            </section>
            {move || match status.get() {
                Status::Loading => {
                    view! { <p class="status" role="status">"Loading…"</p> }.into_any()
                }
                Status::Failed(error) => {
                    view! {
                        <p class="status error" role="status">"Could not load text: " {error}</p>
                    }
                        .into_any()
                }
                Status::Empty => {
                    view! { <p class="status" role="status">"Nothing to score"</p> }.into_any()
                }
                Status::Ready => view! { <Results sides scores /> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn SourcePicker(source: RwSignal<Source>, pasted: RwSignal<String>) -> impl IntoView {
    let option = move |value: Source, name: &'static str| {
        view! {
            <label class="segment">
                <input
                    type="radio"
                    name="source"
                    prop:checked=move || source.get() == value
                    on:change=move |_| source.set(value)
                />
                <span>{name}</span>
            </label>
        }
    };
    view! {
        <fieldset class="source">
            <legend>"Text"</legend>
            <div class="segments">
                {Corpus::ALL.map(|corpus| option(Source::Corpus(corpus), corpus.name()))}
                {option(Source::Pasted, "Paste")}
            </div>
            <Show when=move || source.get() == Source::Pasted>
                <textarea
                    aria-label="Text to score"
                    placeholder="Paste text"
                    rows="6"
                    prop:value=pasted
                    on:input:target=move |ev| pasted.set(ev.target().value())
                ></textarea>
            </Show>
        </fieldset>
    }
}

#[component]
fn LayoutPicker(label: &'static str, side: Side) -> impl IntoView {
    view! {
        <div class="layout-picker">
            <label>
                <span>{label}</span>
                <select
                    prop:value=move || {
                        let preset = side.preset.get();
                        Preset::ALL.iter().position(|p| *p == preset).unwrap_or(0).to_string()
                    }
                    on:change:target=move |ev| {
                        let chosen = ev.target().value().parse::<usize>().ok();
                        if let Some(&chosen) = chosen.and_then(|i| Preset::ALL.get(i)) {
                            side.choose(chosen);
                        }
                    }
                >
                    {Preset::ALL
                        .iter()
                        .enumerate()
                        .map(|(i, p)| view! { <option value=i.to_string()>{p.name()}</option> })
                        .collect_view()}
                </select>
            </label>
            // Stays mounted while disabled: unmounting a focused button would drop
            // keyboard focus to <body>. aria-disabled, not disabled, for the same reason.
            <button
                type="button"
                class="reset"
                aria-disabled=move || (!side.edited()).to_string()
                on:click=move |_| {
                    if side.edited() {
                        side.choose(side.preset.get());
                    }
                }
            >
                "Reset"
            </button>
        </div>
    }
}

#[component]
fn Results(sides: [Side; 2], scores: Scores) -> impl IntoView {
    // One colour scale for both keyboards, so equal colours mean equal load.
    // Thumbs are left out: space would dwarf every letter key.
    let max_share = Memo::new(move |_| {
        scores.with(|scores| {
            scores.as_ref().map_or(0.0, |scores| {
                scores
                    .iter()
                    .flat_map(|p| p.keys[..32].iter().map(move |&n| share(p, n)))
                    .fold(0.0, f64::max)
            })
        })
    });
    let keystrokes = move || scores.with(|s| s.as_ref().map_or(0, |s| s[0].len));
    // A pending key would otherwise come back selected after a text change.
    on_cleanup(move || {
        for side in sides {
            side.selected.set(None);
        }
    });

    view! {
        <section class="scores" aria-label="Penalty per keystroke" aria-live="polite">
            <ScoreCard sides scores side=0 />
            <ScoreCard sides scores side=1 />
        </section>
        <p class="meta">"Keystrokes " {move || group_thousands(keystrokes())}</p>
        <section>
            <h2>"Heatmap"</h2>
            <p class="hint" id="swap-hint">"Select two keys to swap"</p>
            <div class="keyboards">
                <Keyboard sides scores side=0 max_share />
                <Keyboard sides scores side=1 max_share />
            </div>
        </section>
        <section>
            <h2>"Finger load"</h2>
            <FingerLoad sides scores />
        </section>
        <section>
            <h2>"Penalties"</h2>
            <PenaltyTable sides scores />
        </section>
    }
}

fn group_thousands(n: i64) -> String {
    let digits = n.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

#[component]
fn ScoreCard(sides: [Side; 2], scores: Scores, side: usize) -> impl IntoView {
    let score = move || read(scores, side, per_keystroke);
    // Only the second card compares against the first.
    let change = move || {
        (side == 1).then(|| {
            let base = read(scores, 0, per_keystroke);
            format!("{:+.0}%", (score() - base) / base * 100.0)
        })
    };
    view! {
        <div class="score">
            <span class="score-name">{move || sides[side].name()}</span>
            <span class="score-value">{move || format!("{:.3}", score())}</span>
            <span class="score-change">{change}</span>
        </div>
    }
}

// Grid row, column, and column span. 12 columns: 5 left keys, a gap column,
// then 6 right keys. The space thumb sits on row 4 under the inner columns.
fn key_cell(pos: usize) -> (usize, usize, usize) {
    let (row, i) = match pos {
        0..=10 => (1, pos),
        11..=21 => (2, pos - 11),
        22..=31 => (3, pos - 22),
        33 => return (4, 7, 4),
        _ => unreachable!("the left thumb is empty in every preset and never rendered"),
    };
    (row, if i < 5 { i + 1 } else { i + 2 }, 1)
}

fn heat(share: f64, max_share: f64) -> f64 {
    if max_share > 0.0 { (share / max_share).min(1.0) } else { 0.0 }
}

#[component]
fn Keyboard(sides: [Side; 2], scores: Scores, side: usize, max_share: Memo<f64>) -> impl IntoView {
    let this = sides[side];
    let key_view = move |pos: usize| {
        let label = move || match this.layout.with(|layout| layout.base_keys()[pos]) {
            ' ' => "space".to_string(),
            c => c.to_string(),
        };
        let key_share = move || read(scores, side, |p| share(p, p.keys[pos]));
        let key_heat = move || heat(key_share(), max_share.get());
        let percent = move || format!("{:.1}", key_share() * 100.0);
        let (row, column, span) = key_cell(pos);
        let style = move || {
            format!("grid-row: {row}; grid-column: {column} / span {span}; --heat: {:.3}", key_heat())
        };
        (label, key_heat, percent, style)
    };

    // Letter rows swap. The right thumb holds space and newline, so it stays
    // put, and the left thumb is empty in every preset.
    let letter_keys = (0..32)
        .map(|pos| {
            let (label, key_heat, percent, style) = key_view(pos);
            let selected = move || this.selected.get() == Some(pos);
            // Black below 0.7 heat, white above: every mix of --cold and --hot
            // then keeps at least 4.6:1 contrast (WCAG AA).
            view! {
                <button
                    type="button"
                    class="key"
                    class:hot={move || key_heat() > 0.7}
                    class:selected=selected
                    aria-pressed=move || selected().to_string()
                    aria-describedby="swap-hint"
                    aria-label=move || format!("{} {}%", label(), percent())
                    style=style
                    on:click=move |_| {
                        // Swaps stay within one keyboard, so a click here cancels
                        // a key pending on the other one.
                        sides[1 - side].selected.set(None);
                        this.click_key(pos);
                    }
                >
                    <span class="key-char" aria-hidden="true">{label}</span>
                    <span class="key-share" aria-hidden="true">{percent}</span>
                </button>
            }
        })
        .collect_view();
    let (label, key_heat, percent, style) = key_view(33);

    view! {
        <figure class="keyboard">
            <figcaption>{move || this.name()}</figcaption>
            <div class="keys">
                {letter_keys}
                <div
                    class="key"
                    class:hot={move || key_heat() > 0.7}
                    role="img"
                    aria-label=move || format!("{} {}%", label(), percent())
                    style=style
                >
                    <span class="key-char" aria-hidden="true">{label}</span>
                    <span class="key-share" aria-hidden="true">{percent}</span>
                </div>
            </div>
        </figure>
    }
}

#[derive(Clone, Copy)]
enum Load {
    Finger(usize),
    Hand(usize),
}

impl Load {
    fn share(self, penalty: &Penalty) -> f64 {
        match self {
            Load::Finger(i) => share(penalty, penalty.fingers[i]),
            Load::Hand(i) => share(penalty, penalty.hands[i]),
        }
    }
}

// `Penalty::fingers` order, rearranged to read left to right across the hands.
static FINGERS: [(&str, Load); 8] = [
    ("L pinky", Load::Finger(0)),
    ("L ring", Load::Finger(1)),
    ("L middle", Load::Finger(2)),
    ("L index", Load::Finger(3)),
    ("R index", Load::Finger(7)),
    ("R middle", Load::Finger(6)),
    ("R ring", Load::Finger(5)),
    ("R pinky", Load::Finger(4)),
];

static HANDS: [(&str, Load); 2] = [("Left hand", Load::Hand(0)), ("Right hand", Load::Hand(1))];

#[component]
fn FingerLoad(sides: [Side; 2], scores: Scores) -> impl IntoView {
    view! {
        <table class="data-table">
            <thead>
                <tr>
                    <th scope="col">"Finger"</th>
                    <th scope="col">{move || sides[0].name()}</th>
                    <th scope="col">{move || sides[1].name()}</th>
                </tr>
            </thead>
            <BarRows scores rows=&FINGERS />
            <BarRows scores rows=&HANDS />
        </table>
    }
}

// Each group gets its own scale: hands carry several times a finger's load.
#[component]
fn BarRows(scores: Scores, rows: &'static [(&'static str, Load)]) -> impl IntoView {
    let max = Memo::new(move |_| {
        rows.iter()
            .flat_map(|&(_, load)| [0, 1].map(|s| read(scores, s, |p| load.share(p))))
            .fold(0.0, f64::max)
    });
    let width = move |share: f64| if max.get() > 0.0 { share / max.get() * 100.0 } else { 0.0 };
    view! {
        <tbody>
            {rows
                .iter()
                .map(|&(name, load)| {
                    let cell = move |s: usize| {
                        let value = move || read(scores, s, |p| load.share(p));
                        view! {
                            <td class="bar-cell">
                                <div class="bar-line">
                                    <span class="bar-track">
                                        <span
                                            class=format!("bar side-{s}")
                                            style=move || format!("width: {:.1}%", width(value()))
                                        ></span>
                                    </span>
                                    <span class="bar-value">
                                        {move || format!("{:.1}%", value() * 100.0)}
                                    </span>
                                </div>
                            </td>
                        }
                    };
                    view! {
                        <tr>
                            <th scope="row">{name}</th>
                            {cell(0)}
                            {cell(1)}
                        </tr>
                    }
                })
                .collect_view()}
        </tbody>
    }
}

#[component]
fn PenaltyTable(sides: [Side; 2], scores: Scores) -> impl IntoView {
    let cost = move |s: usize, kind: PenaltyKind| {
        move || format!("{:.3}", read(scores, s, |p| p.row(kind).total / p.len as f64))
    };
    let rows = PenaltyKind::ALL
        .iter()
        .map(|&kind| {
            view! {
                <tr>
                    <th scope="row">{kind.name()}</th>
                    <td>{cost(0, kind)}</td>
                    <td>{cost(1, kind)}</td>
                </tr>
            }
        })
        .collect_view();
    let total = move |s: usize| move || format!("{:.3}", read(scores, s, per_keystroke));

    view! {
        <table class="data-table numeric">
            <thead>
                <tr>
                    <th scope="col">"Penalty"</th>
                    <th scope="col">{move || sides[0].name()}</th>
                    <th scope="col">{move || sides[1].name()}</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
            <tfoot>
                <tr>
                    <th scope="row">"Total"</th>
                    <td>{total(0)}</td>
                    <td>{total(1)}</td>
                </tr>
            </tfoot>
        </table>
    }
}
