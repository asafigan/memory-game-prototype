use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use std::{rc::Rc, time::Duration};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let classic_game = |number_of_pairs| move || view! {<ClassicGame number_of_pairs/>};

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/memory-game-prototype.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/classic" view=ClassicPage/>
                    <Route path="/classic/2x3" view=classic_game(3)/>
                    <Route path="/classic/2x4" view=classic_game(4)/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="column">
            <A href="/classic" class="button">Classic</A>
        </div>
    }
}

#[component]
fn ClassicPage() -> impl IntoView {
    view! {
        <div class="column">
            <A href="/classic/2x3" class="button">2x3</A>
            <A href="/classic/2x4" class="button">2x4</A>
        </div>
    }
}

#[component]
fn ClassicGame(number_of_pairs: u8) -> impl IntoView {
    let number_of_pairs = number_of_pairs as usize;
    let available_pairs = [
        Pair {
            matches: ["A".into(), "A".into()],
        },
        Pair {
            matches: ["B".into(), "B".into()],
        },
        Pair {
            matches: ["C".into(), "C".into()],
        },
        Pair {
            matches: ["D".into(), "D".into()],
        },
        Pair {
            matches: ["E".into(), "E".into()],
        },
        Pair {
            matches: ["F".into(), "F".into()],
        },
    ];

    let mut pairs = Vec::new();
    while pairs.len() < number_of_pairs.into() {
        pairs.extend(
            available_pairs
                .iter()
                .take(number_of_pairs - pairs.len())
                .cloned(),
        );
    }

    view! {
        <Game options=pairs.into()/>
    }
}

#[component]
fn GamePage() -> impl IntoView {
    let options = [
        Pair {
            matches: ["A".into(), "A".into()],
        },
        Pair {
            matches: ["B".into(), "B".into()],
        },
        Pair {
            matches: ["C".into(), "C".into()],
        },
        Pair {
            matches: ["D".into(), "D".into()],
        },
        Pair {
            matches: ["E".into(), "E".into()],
        },
        Pair {
            matches: ["F".into(), "F".into()],
        },
    ]
    .into();

    view! {
        <Game options/>
    }
}

type Pairs = Rc<[Pair]>;

#[component]
fn Game(options: Pairs) -> impl IntoView {
    let (pairs, set_pairs) = create_signal([].into());
    let start = move || {
        set_pairs(options.clone());
    };
    let start_up = start.clone();
    create_effect(move |_| start_up());

    view! {
        <h1>"Game"</h1>
        <Show when=move || !pairs().is_empty() fallback=|| ()>
            <GameMatch pairs=pairs() restart=start.clone()/>
        </Show>
    }
}

#[component]
fn GameMatch<Restart>(pairs: Pairs, restart: Restart) -> impl IntoView
where
    Restart: Fn() + Clone + 'static,
{
    let mut cards: Vec<_> = pairs
        .into_iter()
        .flat_map(|x| &x.matches)
        .enumerate()
        .map(|(id, item)| {
            let (state, set_state) = create_signal(CardState::default());
            CardData {
                id,
                item: item.clone(),
                state,
                set_state,
            }
        })
        .collect();
    fastrand::shuffle(&mut cards);
    let (cards_left, set_cards_left) = create_signal(cards.len());
    let (win, set_win) = create_signal(false);
    create_effect(move |_| {
        if cards_left() == 0 {
            set_timeout(move || set_win(true), Duration::from_secs(1));
        }
    });
    let (_, set_selected) = create_signal(Vec::<CardData>::new());
    let cards = cards
        .into_iter()
        .map(|card| {
            let card_copy = card.clone();
            let select = move || {
                match (card_copy.state)() {
                    CardState::Unselected | CardState::Failure => {}
                    _ => {
                        return;
                    }
                };
                set_selected.update(|selected| {
                    if selected.len() > 1 {
                        for selected in selected.iter() {
                            selected.set_state.update(|state| {
                                *state = match *state {
                                    CardState::Failure => CardState::Unselected,
                                    CardState::Success => CardState::Hidden,
                                    x => x,
                                };
                            })
                        }

                        selected.clear();
                    }

                    if let Some(other) = selected.last() {
                        if other.id != card_copy.id {
                            let state = if other.item == card_copy.item {
                                set_cards_left.update(|left| {
                                    *left -= selected.len() + 1;
                                });
                                CardState::Success
                            } else {
                                CardState::Failure
                            };

                            other.set_state.set(state);
                            card_copy.set_state.set(state);
                            selected.push(card_copy.clone());
                        }
                    } else {
                        card_copy.set_state.set(CardState::Selected);
                        selected.push(card_copy.clone());
                    }
                });
            };

            let CardData { item, state, .. } = card;

            view! {
                <Card item state select/>
            }
        })
        .collect_view();

    view! {
        <div class="board">{cards}</div>
        <Show when=win fallback=|| ()>
            <WinScreen restart=restart.clone()/>
        </Show>
    }
}

#[derive(Clone)]
struct CardData {
    id: usize,
    item: Item,
    state: ReadSignal<CardState>,
    set_state: WriteSignal<CardState>,
}

type Item = Rc<str>;

#[derive(Clone)]
struct Pair {
    matches: [Item; 2],
}

#[component]
fn Card<StateFn, SelectFn>(item: Item, state: StateFn, mut select: SelectFn) -> impl IntoView
where
    StateFn: Fn() -> CardState + Copy + 'static,
    SelectFn: FnMut() + 'static,
{
    let flipped = move || state() != CardState::Unselected;
    let success = move || state() == CardState::Success;
    let fail = move || state() == CardState::Failure;
    let show = move || state() != CardState::Hidden;
    view! {
        <div on:click=move |_| select() class="card" class:flipped=flipped class:success=success class:fail=fail>
            <Show when=show fallback=|| ()>
                <div class="front">{item.to_string()}</div>
                <div class="back"></div>
            </Show>
        </div>
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
enum CardState {
    #[default]
    Unselected,
    Selected,
    Failure,
    Success,
    Hidden,
}

#[component]
fn WinScreen<Restart>(restart: Restart) -> impl IntoView
where
    Restart: Fn() + 'static,
{
    view! {
        <div class="shim">
            <div class="popup">
                <h2>"🎉 You win! 🎉"</h2>
                <div class="button" on:click=move |_| restart()>"Play Again"</div>
                <A class="button" href="/">"Home"</A>
            </div>
        </div>
    }
}
