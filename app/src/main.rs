//! Browser playground: score text on two keyboard layouts side by side.

use gloo_net::http::Request;
use keygen_core::layout::{self, Layout};
use keygen_core::penalty::{
    Penalty, QuartadList, calculate_penalty, normalize_text, prepare_quartad_list,
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

#[derive(Clone, PartialEq)]
struct Side {
    preset: Preset,
    penalty: Penalty,
}

impl Side {
    fn share(&self, count: i64) -> f64 {
        count as f64 / self.penalty.len as f64
    }

    fn per_keystroke(&self) -> f64 {
        self.penalty.total / self.penalty.len as f64
    }
}

#[derive(Clone, PartialEq)]
enum Comparison {
    Loading,
    Failed(String),
    // Text under 3 characters, or every trigram holds a character without a key.
    Empty,
    Ready([Side; 2]),
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
    let presets = [RwSignal::new(Preset::Qwerty), RwSignal::new(Preset::Mine)];

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

    let comparison = Memo::new(move |_| {
        quartads.with(|quartads| match quartads {
            Loaded::Loading => Comparison::Loading,
            Loaded::Failed(error) => Comparison::Failed(error.clone()),
            Loaded::Ready(quartads) => {
                let sides = presets.map(|preset| {
                    let preset = preset.get();
                    Side {
                        preset,
                        penalty: calculate_penalty(quartads, preset.layout()).penalty,
                    }
                });
                // All presets map every letter, space, and newline, so either
                // side being empty means both are.
                if sides[0].penalty.len == 0 {
                    Comparison::Empty
                } else {
                    Comparison::Ready(sides)
                }
            }
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
                    <LayoutPicker label="Layout A" preset=presets[0] />
                    <LayoutPicker label="Layout B" preset=presets[1] />
                </div>
            </section>
            {move || match comparison.get() {
                Comparison::Loading => view! { <p class="status" role="status">"Loading…"</p> }.into_any(),
                Comparison::Failed(error) => {
                    view! { <p class="status error" role="status">"Could not load text: " {error}</p> }.into_any()
                }
                Comparison::Empty => view! { <p class="status" role="status">"Nothing to score"</p> }.into_any(),
                Comparison::Ready(sides) => view! { <Results sides /> }.into_any(),
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
fn LayoutPicker(label: &'static str, preset: RwSignal<Preset>) -> impl IntoView {
    view! {
        <label class="layout-picker">
            <span>{label}</span>
            <select
                prop:value=move || {
                    Preset::ALL.iter().position(|p| *p == preset.get()).unwrap_or(0).to_string()
                }
                on:change:target=move |ev| {
                    let chosen = ev.target().value().parse::<usize>().ok();
                    if let Some(&chosen) = chosen.and_then(|i| Preset::ALL.get(i)) {
                        preset.set(chosen);
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
    }
}

#[component]
fn Results(sides: [Side; 2]) -> impl IntoView {
    // One colour scale for both keyboards, so equal colours mean equal load.
    // Thumbs are left out: space would dwarf every letter key.
    let max_share = sides
        .iter()
        .flat_map(|side| side.penalty.keys[..32].iter().map(move |&n| side.share(n)))
        .fold(0.0, f64::max);
    let keystrokes = sides[0].penalty.len;

    view! {
        <section class="scores" aria-label="Penalty per keystroke">
            <ScoreCard side=sides[0].clone() baseline=None />
            <ScoreCard side=sides[1].clone() baseline=Some(sides[0].per_keystroke()) />
        </section>
        <p class="meta">"Keystrokes " {group_thousands(keystrokes)}</p>
        <section>
            <h2>"Heatmap"</h2>
            <div class="keyboards">
                <Keyboard side=sides[0].clone() max_share />
                <Keyboard side=sides[1].clone() max_share />
            </div>
        </section>
        <section>
            <h2>"Finger load"</h2>
            <FingerLoad sides=sides.clone() />
        </section>
        <section>
            <h2>"Penalties"</h2>
            <PenaltyTable sides />
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
fn ScoreCard(side: Side, baseline: Option<f64>) -> impl IntoView {
    let score = side.per_keystroke();
    let change = baseline.map(|base| format!("{:+.0}%", (score - base) / base * 100.0));
    view! {
        <div class="score">
            <span class="score-name">{side.preset.name()}</span>
            <span class="score-value">{format!("{score:.3}")}</span>
            <span class="score-change">{change}</span>
        </div>
    }
}

// Grid row, column, and column span. 12 columns: 5 left keys, a gap column,
// then 6 right keys. Thumb keys sit on row 4 under the inner columns.
fn key_cell(pos: usize) -> (usize, usize, usize) {
    match pos {
        0..=31 => {
            let (row, i) = match pos {
                0..=10 => (1, pos),
                11..=21 => (2, pos - 11),
                _ => (3, pos - 22),
            };
            (row, if i < 5 { i + 1 } else { i + 2 }, 1)
        }
        32 => (4, 4, 2),
        _ => (4, 7, 4),
    }
}

#[component]
fn Keyboard(side: Side, max_share: f64) -> impl IntoView {
    let keys = side
        .preset
        .layout()
        .base_keys()
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c != '\0')
        .map(|(pos, &c)| {
            let share = side.share(side.penalty.keys[pos]);
            let heat = if max_share > 0.0 { (share / max_share).min(1.0) } else { 0.0 };
            let (row, column, span) = key_cell(pos);
            let label = if c == ' ' { "space".to_string() } else { c.to_string() };
            let percent = format!("{:.1}", share * 100.0);
            let description = format!("{label} {percent}%");
            view! {
                // Black below 0.7 heat, white above: every mix of --cold and
                // --hot then keeps at least 4.6:1 contrast (WCAG AA).
                <div
                    class="key"
                    class:hot={heat > 0.7}
                    role="img"
                    style=format!(
                        "grid-row: {row}; grid-column: {column} / span {span}; --heat: {heat:.3}",
                    )
                    aria-label=description
                >
                    <span class="key-char" aria-hidden="true">{label}</span>
                    <span class="key-share" aria-hidden="true">{percent}</span>
                </div>
            }
        })
        .collect_view();

    view! {
        <figure class="keyboard">
            <figcaption>{side.preset.name()}</figcaption>
            <div class="keys">{keys}</div>
        </figure>
    }
}

// `Penalty::fingers` order, rearranged to read left to right across the hands.
const FINGERS: [(&str, usize); 8] = [
    ("L pinky", 0),
    ("L ring", 1),
    ("L middle", 2),
    ("L index", 3),
    ("R index", 7),
    ("R middle", 6),
    ("R ring", 5),
    ("R pinky", 4),
];

#[component]
fn FingerLoad(sides: [Side; 2]) -> impl IntoView {
    let finger_rows: Vec<(&str, [f64; 2])> = FINGERS
        .iter()
        .map(|&(name, i)| (name, [0, 1].map(|s| sides[s].share(sides[s].penalty.fingers[i]))))
        .collect();
    let hand_rows: Vec<(&str, [f64; 2])> = [("Left hand", 0), ("Right hand", 1)]
        .iter()
        .map(|&(name, i)| (name, [0, 1].map(|s| sides[s].share(sides[s].penalty.hands[i]))))
        .collect();

    view! {
        <table class="data-table">
            <thead>
                <tr>
                    <th scope="col">"Finger"</th>
                    <th scope="col">{sides[0].preset.name()}</th>
                    <th scope="col">{sides[1].preset.name()}</th>
                </tr>
            </thead>
            <BarRows rows=finger_rows />
            <BarRows rows=hand_rows />
        </table>
    }
}

// Each group gets its own scale: hands carry several times a finger's load.
#[component]
fn BarRows(rows: Vec<(&'static str, [f64; 2])>) -> impl IntoView {
    let max = rows.iter().flat_map(|(_, shares)| *shares).fold(0.0, f64::max);
    let width = move |share: f64| if max > 0.0 { share / max * 100.0 } else { 0.0 };
    view! {
        <tbody>
            {rows
                .into_iter()
                .map(|(name, shares)| {
                    view! {
                        <tr>
                            <th scope="row">{name}</th>
                            {shares
                                .iter()
                                .enumerate()
                                .map(|(s, &share)| {
                                    view! {
                                        <td class="bar-cell">
                                            <div class="bar-line">
                                                <span class="bar-track">
                                                    <span
                                                        class=format!("bar side-{s}")
                                                        style=format!("width: {:.1}%", width(share))
                                                    ></span>
                                                </span>
                                                <span class="bar-value">
                                                    {format!("{:.1}%", share * 100.0)}
                                                </span>
                                            </div>
                                        </td>
                                    }
                                })
                                .collect_view()}
                        </tr>
                    }
                })
                .collect_view()}
        </tbody>
    }
}

#[component]
fn PenaltyTable(sides: [Side; 2]) -> impl IntoView {
    let rows = sides[0]
        .penalty
        .penalties
        .iter()
        .zip(&sides[1].penalty.penalties)
        .map(|(a, b)| {
            let cost = |side: &Side, total: f64| format!("{:.3}", total / side.penalty.len as f64);
            view! {
                <tr>
                    <th scope="row">{a.kind.name()}</th>
                    <td>{cost(&sides[0], a.total)}</td>
                    <td>{cost(&sides[1], b.total)}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <table class="data-table numeric">
            <thead>
                <tr>
                    <th scope="col">"Penalty"</th>
                    <th scope="col">{sides[0].preset.name()}</th>
                    <th scope="col">{sides[1].preset.name()}</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
            <tfoot>
                <tr>
                    <th scope="row">"Total"</th>
                    <td>{format!("{:.3}", sides[0].per_keystroke())}</td>
                    <td>{format!("{:.3}", sides[1].per_keystroke())}</td>
                </tr>
            </tfoot>
        </table>
    }
}
