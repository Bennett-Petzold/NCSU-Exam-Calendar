/*
* Copyright (C) 2023 Bennett Petzold
*
* This file is part of ncsu_exam_calendar.
*
* ncsu_exam_calendar is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 2 of the License, or (at your option) any later version.
*
* ncsu_exam_calendar is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License along with ncsu_exam_calendar. If not, see <https://www.gnu.org/licenses/>.
*/

#![cfg(feature = "dioxus")]

use std::ops::Range;
use std::ops::{Deref, DerefMut};

use anyhow::Result;
use chrono::NaiveTime;
use dioxus::prelude::*;
use itertools::Itertools;

use reqwest::Client;
use tokio::runtime::Builder;

use crate::calendar::Class;
use crate::calendar::Weekday;
use crate::calendar::{get_calendars, Calendar, CalendarMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    URL,
    JSON,
}

#[derive(Debug, Clone, PartialEq, Eq, Props)]
pub struct SourceTypeProp {
    #[props(into)]
    inner: SourceType,
}

impl Deref for SourceTypeProp {
    type Target = SourceType;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SourceTypeProp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<SourceType> for SourceTypeProp {
    fn from(value: SourceType) -> Self {
        Self { inner: value }
    }
}

const DEFAULT_URL: &str = "https://studentservices.ncsu.edu/calendars/exam-calendar/";
const DEFAULT_JSON: &str = "./exams.json";

async fn load_default_map() -> Result<CalendarMap> {
    #[cfg(not(target_family = "wasm"))]
    let src = std::fs::read_to_string(DEFAULT_JSON)?;
    // TODO: Look into fetching this from server instead
    #[cfg(target_family = "wasm")]
    let src = { include_str!("../../../public/exams.json") };

    Ok(serde_json::from_str(&src)?)
}

pub fn app(cx: Scope) -> Element {
    let version = "v".to_string() + option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN");

    use_shared_state_provider(cx, || {
        Builder::new_current_thread()
            .build()
            .unwrap()
            .handle()
            .block_on(async { load_default_map().await.ok() })
    });
    let source = use_shared_state::<Option<CalendarMap>>(cx).unwrap();

    use_shared_state_provider(cx, Option::<Calendar>::default);
    let semester = use_shared_state::<Option<Calendar>>(cx).unwrap();

    cx.render(rsx! {
        h1 {
            "ABSOLUTELY NO WARRANTY"
        },
        source_select {},
        br {},
        semesters_display {
            source: source.read().clone()
        },
        br {},
        get_exam_time {
            semester: semester.read().clone()
        }
        br {},
        br {},
        i {
            version
        }
    })
}

fn source_select(cx: Scope) -> Element {
    let source = use_shared_state::<Option<CalendarMap>>(cx).unwrap();

    let source_type = use_state(cx, || SourceTypeProp {
        #[cfg(not(target_family = "wasm"))]
        inner: SourceType::URL,
        #[cfg(target_family = "wasm")]
        inner: SourceType::JSON,
    });

    cx.render(rsx! {
        div {
            display: "flex",
            gap: "20px",
            div {
                flex: true,
                label {
                    r#for: "source_select",
                    "Select a source: "
                }
                select {
                    onchange: move |event| {
                        match event.value.as_str() {
                            "URL" => source_type.set(SourceType::URL.into()),
                            "JSON" => source_type.set(SourceType::JSON.into()),
                            x => panic!("Impossible select value: {x}"),
                        }
                    },
                    name: "source_select",
                    id: "source_select",
                    option {
                        selected: source_type.get().inner == SourceType::URL,
                        "URL"
                    },
                    option {
                        selected: source_type.get().inner == SourceType::JSON,
                        "JSON"
                    },
                },
            },
        },
        br {},
        url_source {
            s_type: source_type.get().clone()
        },
        json_source {
            s_type: source_type.get().clone()
        },
        h2 {
            if source.read().is_some() {
                "LOADED DATA"
            } else {
                "WAITING FOR DATA"
            },
        },
    })
}

#[inline_props]
fn url_source(cx: Scope, s_type: SourceTypeProp) -> Element {
    if s_type.deref() != &SourceType::URL {
        return None;
    };

    let source = use_shared_state::<Option<CalendarMap>>(cx).unwrap();
    let path = use_state(cx, || DEFAULT_URL.to_string());

    let invalid_url = use_future(cx, (path,), |(path,)| async move {
        if let Ok(client) = Client::builder().build() {
            if let Ok(response) = client.head(path.get()).send().await {
                response.error_for_status().is_err()
            } else {
                true
            }
        } else {
            true
        }
    });

    #[cfg(not(target_family = "wasm"))]
    const BUTTON_TEXT: &str = "GET from site";
    #[cfg(target_family = "wasm")]
    const BUTTON_TEXT: &str = "Impossible in web due to CORS";

    cx.render(rsx! {
        div {
            input {
                size: 50,
                id: "source_string",
                value: "{path}",
                oninput: move |event| {
                    path.set(event.value.clone());
                },
            },
            br {},
            br {},
            input {
                r#type: "button",
                id: "source_send",
                value: BUTTON_TEXT,
                disabled: *invalid_url.value().unwrap_or(&true),
                onclick: move |_| {
                    to_owned!(source, path);
                    async move {
                        *source.write() = get_calendars(path.get()).await.ok();
                    }
                }
            }
        }
    })
}

#[inline_props]
fn json_source(cx: Scope, s_type: SourceTypeProp) -> Element {
    if s_type.deref() != &SourceType::JSON {
        return None;
    };

    let file_name = use_state(cx, || "".to_string());

    let source = use_shared_state::<Option<CalendarMap>>(cx).unwrap();

    cx.render(rsx! {
        div {
            input {
                r#type: "file",
                accept: ".json",
                id: "json_file",
                onchange: move |event| {
                    to_owned!(source, file_name);
                    async move {
                        if let Some(file_engine) = &event.files {
                            let input_file = &file_engine.files()[0];
                            if let Some(contents) = file_engine.read_file_to_string(input_file).await {
                                *source.write() = serde_json::from_str(&contents).ok();
                                file_name.set(r"C:\fakepath\".to_string() + input_file);
                            }
                        }
                    }
                }
            }
        }
    })
}

#[inline_props]
fn semesters_display(cx: Scope, source: Option<Option<CalendarMap>>) -> Element {
    let semester = use_shared_state::<Option<Calendar>>(cx).unwrap();

    let semester_options = use_future(cx, (source,), |(source,)| async move {
        if let Some(Some(source)) = source {
            source.keys().cloned().sorted().collect()
        } else {
            vec![]
        }
    });

    match semester_options.value() {
        None => return None,
        Some(sem_opts) if sem_opts.is_empty() => return None,
        _ => (),
    };

    cx.render(rsx! {
        div {
            flex: true,
            label {
                r#for: "sem_select",
                "Select a semester: "
            }
            select {
                onchange: move |event| {
                    if let Some(Some(cal_map)) = source {
                        *semester.write() = cal_map.get(&event.value).cloned();
                    } else {
                        *semester.write() = None;
                    }
                },
                name: "sem_select",
                id: "sem_select",
                option {
                    ""
                },
                for sem in semester_options.value().unwrap_or(&vec![]) {
                    option {
                        "{sem}"
                    }
                },
            },
        }
    })
}

#[inline_props]
fn get_exam_time(cx: Scope, semester: Option<Option<Calendar>>) -> Element {
    use_shared_state_provider(cx, Option::<Class>::default);
    let class_choice = use_shared_state::<Option<Class>>(cx).unwrap();

    if let Some(Some(semester)) = semester {
        let classes = use_state(cx, <Vec<Class>>::default);
        use_future(cx, (semester,), |(semester,)| {
            to_owned![classes];
            async move {
                classes.set(semester.keys().cloned().collect());
            }
        });

        cx.render(rsx! {
            h3 {
                "Select your class:"
            },
            div {
                display: "flex",
                gap: "40px",
                named_classes {
                    classes: classes.get().clone(),
                },
                day_time_classes {
                    classes: classes.get().clone(),
                },
                time_range_classes {
                    classes: classes.get().clone(),
                },
            },
            br {},
            exam_display {
                class_choice: class_choice.read().clone(),
                semester: semester.clone(),
            },
        })
    } else {
        None
    }
}

const TIME_FORMAT: &str = "%-I:%M %p";

#[inline_props]
fn day_time_classes(cx: Scope, classes: Vec<Class>) -> Element {
    let class_choice = use_shared_state::<Option<Class>>(cx).unwrap();

    let class_days = use_state(cx, Vec::<Weekday>::new);
    use_future(cx, (classes,), |(classes,)| {
        to_owned![class_days];
        async move {
            class_days.set(
                classes
                    .iter()
                    .filter_map(|class| {
                        if let Class::Time(days, _) = class {
                            Some(days)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .unique()
                    .sorted()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        }
    });

    if class_days.is_empty() {
        return None;
    }

    let selected_class_days = use_state(cx, Vec::<Weekday>::new);

    let class_times = use_state(cx, Vec::<NaiveTime>::new);
    use_future(
        cx,
        (classes, selected_class_days),
        |(classes, selected_class_days)| {
            to_owned![class_times];
            async move {
                class_times.set(
                    classes
                        .iter()
                        .filter_map(|class| {
                            if let Class::Time(days, time) = class {
                                if selected_class_days
                                    .clone()
                                    .iter()
                                    .all(|sel_day| days.contains(sel_day))
                                {
                                    Some(time)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unique()
                        .sorted()
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }
        },
    );

    let debug = use_state(cx, || "".to_string());

    let selected_time = use_state(cx, Option::<NaiveTime>::default);

    use_future(
        cx,
        (selected_class_days, selected_time),
        |(selected_class_days, selected_time)| {
            to_owned![class_choice];
            async move {
                if let Some(time) = selected_time.get() {
                    *class_choice.write() = Some(Class::Time(
                        selected_class_days.get().iter().sorted().cloned().collect(),
                        *time,
                    ));
                }
            }
        },
    );

    cx.render(rsx! {
        div {
            flex: true,
            "{debug}",
            h4 {
                "By day & time:",
            },
            div {
                display: "flex",
                gap: "10px",
                for day in class_days.get() {
                    div {
                        flex: true,
                        input {
                            r#type: "checkbox",
                            name: "select_{day:?}",
                            value: "{day:?}",
                            onchange: move |event| {
                                if event.value.to_lowercase() == "true" {
                                    selected_class_days.with_mut(|sels| { sels.push(day.clone()); sels.sort() });
                                } else {
                                    selected_class_days.with_mut(|sels| sels.retain(|x| x != day));
                                }
                            },
                        },
                        label {
                            r#for: "select_{day:?}",
                            "{day:?}"
                        },
                    }
                },
            },
            br {},
            form {
                for time in class_times.get() {
                    div {
                        input {
                            r#type: "radio",
                            name: "select_time",
                            value: "select_{time:?}",
                            checked: "{*class_choice.read() == Some(Class::Time(selected_class_days.get().clone(), *time))}",
                            onclick: move |_| {
                                selected_time.set(Some(*time));
                            },
                        },
                        label {
                            r#for: "select_{time:?}",
                            "{time.format(TIME_FORMAT).to_string()}",
                            //"{time:?}",
                        },
                    }
                },
            },
        },
    })
}

#[inline_props]
fn named_classes(cx: Scope, classes: Vec<Class>) -> Element {
    let class_choice = use_shared_state::<Option<Class>>(cx).unwrap();

    let named_classes = use_state(cx, Vec::<String>::new);
    use_future(cx, (classes,), |(classes,)| {
        to_owned![named_classes];
        async move {
            named_classes.set(
                classes
                    .iter()
                    .filter_map(|class| {
                        if let Class::Name(text) = class {
                            Some(text)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        }
    });

    if named_classes.is_empty() {
        return None;
    }

    let _owner = if let Some(choice) = class_choice.read().clone() {
        matches!(choice, Class::Name(_))
    } else {
        false
    };

    cx.render(rsx! {
    div {
        flex: true,
        h4 {
            "By name:",
        },
        form {
            name: "named_class_select",
            id: "named_class_select",
            for named in named_classes.get() {
                div {
                    input {
                        r#type: "radio",
                        name: "select_named",
                        value: "select_{named}",
                        checked: "{*class_choice.read() == Some(Class::Name(named.to_string()))}",
                        onclick: move |_| {
                            *class_choice.write() = Some(Class::Name(named.to_string()));
                        },
                    },
                    label {
                        r#for: "select_{named}",
                        "{named}",
                    },
                }
            },
        },
    }})
}

#[inline_props]
fn time_range_classes(cx: Scope, classes: Vec<Class>) -> Element {
    let class_choice = use_shared_state::<Option<Class>>(cx).unwrap();

    let class_ranges = use_state(cx, Vec::<Range<NaiveTime>>::new);
    use_future(cx, (classes,), |(classes,)| {
        to_owned![class_ranges];
        async move {
            class_ranges.set(
                classes
                    .iter()
                    .filter_map(|class| {
                        if let Class::Range(span) = class {
                            Some(span)
                        } else {
                            None
                        }
                    })
                    .sorted_by(|a, b| Ord::cmp(&a.start, &b.start))
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        }
    });

    if class_ranges.is_empty() {
        return None;
    }

    cx.render(rsx! {
    div {
        flex: true,
        h4 {
            "By time range:",
        },
        form {
            name: "named_class_select",
            id: "named_class_select",
            for range in class_ranges.get() {
                div {
                    input {
                        r#type: "radio",
                        name: "select_named",
                        value: "select_{range:?}",
                        checked: "{*class_choice.read() == Some(Class::Range(range.clone()))}",
                        onclick: move |_| {
                            *class_choice.write() = Some(Class::Range(range.clone()));
                        },
                    },
                    label {
                        r#for: "select_{range:?}",
                        "{range.start.format(TIME_FORMAT).to_string()} - {range.end.format(TIME_FORMAT).to_string()}",
                    },
                }
            },
        },
    }})
}

const DATE_FORMAT: &str = "%A, %B %e";

#[inline_props]
fn exam_display(cx: Scope, class_choice: Option<Option<Class>>, semester: Calendar) -> Element {
    if let Some(Some(choice)) = class_choice {
        if let Some(exam) = semester.get(choice) {
            return cx.render(rsx! {
                h4 {
                    "Exam:",
                },
                "{exam.0.format(DATE_FORMAT)}",
                br {},
                "{exam.1.start.format(TIME_FORMAT).to_string()} - {exam.1.end.format(TIME_FORMAT).to_string()}",
            });
        }
    }
    None
}
