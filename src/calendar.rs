use std::{
    collections::HashMap,
    ops::{Deref, Range},
    str::FromStr,
};

use anyhow::{anyhow, bail, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use select::{
    document::Document,
    node::Node,
    predicate::{Any, Name},
};
use serde::{de::Visitor, Deserialize, Serialize};
use strum::EnumString;

use crate::get_page_document;

#[derive(
    Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Clone, PartialOrd, Ord,
)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

lazy_static! {
    static ref WEEKDAY: Regex = Regex::new(r"(:?M|(?:Tu)|W|(?:Th)|F)+").unwrap();
}
lazy_static! {
    static ref WEEKDAY_INNER: Regex = Regex::new(r"M|(?:Tu)|W|(?:Th)|F").unwrap();
}

impl Weekday {
    pub fn factory<S: AsRef<str>>(value: S) -> Result<Vec<Vec<Self>>> {
        WEEKDAY
            .find_iter(value.as_ref())
            .map(|caps| {
                WEEKDAY_INNER
                    .find_iter(caps.into())
                    .map(|m| m.as_str())
                    .map(Self::new)
                    .collect::<Result<Vec<_>>>()
            })
            .collect()
    }

    pub fn new<S: AsRef<str>>(value: S) -> Result<Self> {
        match value.as_ref() {
            "M" => Ok(Self::Monday),
            "Tu" => Ok(Self::Tuesday),
            "W" => Ok(Self::Wednesday),
            "Th" => Ok(Self::Thursday),
            "F" => Ok(Self::Friday),
            x => bail!("{x} is not in the valid set (M, Tu, W, Th, F)"),
        }
    }

    pub fn from_name<S: AsRef<str>>(value: S) -> Result<Self> {
        match value.as_ref() {
            "Monday" => Ok(Self::Monday),
            "Tuesday" => Ok(Self::Tuesday),
            "Wednesday" => Ok(Self::Wednesday),
            "Thursday" => Ok(Self::Thursday),
            "Friday" => Ok(Self::Friday),
            x => bail!("{x} is not a weekday"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Class {
    Time(Vec<Weekday>, NaiveTime),
    Range(Range<NaiveTime>),
    Name(String),
}

lazy_static! {
    static ref CLASS_ENTRY: Regex =
        Regex::new(r"(\d{1,2}:\d{1,2} (?:a|p)\.m\.).*(:?M|Tu|W|Th|F)").unwrap();
}

impl Class {
    pub fn factory<S: AsRef<str>>(value: S) -> Result<Vec<Self>> {
        let value = value.as_ref().trim();
        if value.is_empty() || value == "Common:" {
            bail!("Non-class line")
        }

        let caps = CLASS_ENTRY.captures(value);
        if let Some(caps) = caps {
            let time = dateparser::parse_with_timezone::<Utc>(
                &caps[1].replace('.', ""),
                &Utc::now().timezone(),
            )?
            .time();
            let weekdays = Weekday::factory(value)?;
            Ok(weekdays
                .into_iter()
                .map(|day| Self::Time(day, time))
                .collect())
        } else if let Some((start, end)) = value.split(&['-', '–']).map(str::trim).collect_tuple()
        {
            let start = dateparser::parse_with_timezone::<Utc>(
                &start.replace('.', ""),
                &Utc::now().timezone(),
            )?
            .time();
            let end = dateparser::parse_with_timezone::<Utc>(
                &end.replace('.', ""),
                &Utc::now().timezone(),
            )?
            .time();
            Ok(vec![Self::Range(Range { start, end })])
        } else if let Some((start, end)) = value.split("and").map(str::trim).collect_tuple() {
            let start = dateparser::parse_with_timezone::<Utc>(
                &start.replace('.', ""),
                &Utc::now().timezone(),
            )?
            .time();

            let end = if end.trim().to_lowercase() == "later" {
                "11:59 p.m."
            } else {
                end
            };
            let end = dateparser::parse_with_timezone::<Utc>(
                &end.replace('.', ""),
                &Utc::now().timezone(),
            )?
            .time();
            Ok(vec![Self::Range(Range { start, end })])
        } else {
            Ok(value
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .map(Self::Name)
                .collect())
        }
    }
}

impl Serialize for Class {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Time(weekdays, time) => {
                serializer.serialize_str(&format!("{:?} {:?}", weekdays, time))
            }
            Self::Range(range) => serializer.serialize_str(&format!("{:?}", range)),
            Self::Name(name) => serializer.serialize_str(name),
        }
    }
}

struct ClassVisitor;

impl Visitor<'_> for ClassVisitor {
    type Value = Class;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("\"WEEKDAYS TIME\", \"RANGE\", or NAME")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.contains('[') && v.contains(']') {
            let (days_str, time_str) = v
                .split(']')
                .collect_tuple()
                .ok_or(E::custom("Zero or more than one \"]\""))?;
            let days = days_str[1..]
                .split(", ")
                .map(Weekday::from_str)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| E::custom("Invalid weekday format"))?;
            let time =
                dateparser::parse_with_timezone::<Utc>(time_str.trim(), &Utc::now().timezone())
                    .map_err(|_| E::custom("Invalid time format"))?
                    .time();
            Ok(Class::Time(days, time))
        } else if let Some((start_str, end_str)) = v.split("..").collect_tuple() {
            let start = dateparser::parse_with_timezone::<Utc>(start_str, &Utc::now().timezone())
                .map_err(|_| E::custom("Range begin format is invalid"))?
                .time();
            let end = dateparser::parse_with_timezone::<Utc>(end_str, &Utc::now().timezone())
                .map_err(|_| E::custom("Range end format is invalid"))?
                .time();
            Ok(Class::Range(Range { start, end }))
        } else {
            Ok(Class::Name(v.to_string()))
        }
    }
}

impl<'de> Deserialize<'de> for Class {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ClassVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Calendar(HashMap<Class, (NaiveDate, Range<NaiveTime>)>);

impl Deref for Calendar {
    type Target = HashMap<Class, (NaiveDate, Range<NaiveTime>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<(&str, Node<'_>)> for Calendar {
    type Error = anyhow::Error;
    fn try_from((year, value): (&str, Node)) -> std::result::Result<Self, Self::Error> {
        let head_node = value
            .find(Name("thead"))
            .next()
            .ok_or(anyhow!("No table head row (thead)"))?;
        let head = head_node
            .find(Name("tr"))
            .next()
            .ok_or(anyhow!("Table head row (tr) is empty"))?;
        let mut head_iter = head.find(Any).map(|node| node.text());

        if head_iter.next().ok_or(anyhow!("Table head row is empty"))? != "Exam Dates/Times" {
            bail!("Table does not start with \"Exam Dates/Times\", not a valid exam times table")
        }
        head_iter.next();
        let exam_times = head_iter
            .map(|time| {
                let range = time
                    .replace('.', "")
                    .split('–')
                    .map(str::trim)
                    .map(|x| dateparser::parse_with_timezone(x, &Utc::now().timezone()))
                    .collect_tuple();
                match range {
                    Some((Ok(first), Ok(second))) => Ok((first, second)),
                    Some((Err(first), _)) => Err(first),
                    Some((_, Err(second))) => Err(second),
                    None => bail!("Not a valid range of times separated by \"-\""),
                }
            })
            .map(|range| match range {
                Ok((first, second)) => Ok(Range {
                    start: first.time(),
                    end: second.time(),
                }),
                Err(range) => Err(range),
            })
            .collect::<Result<Vec<_>>>()?;

        let exam_times: Vec<_> = exam_times.into_iter().unique().collect();

        let body = value
            .find(Name("tbody"))
            .next()
            .ok_or(anyhow!("No table body"))?;
        let body_rows = body.find(Name("tr")).map(|row| row.find(Name("td")));
        let assignments = body_rows
            .map(|mut row| {
                let exam_date = row
                    .next()
                    .ok_or(anyhow!("At least one row is empty"))?
                    .text();
                let exam_date = exam_date.replace('.', "");
                let exam_date = exam_date.replace(',', "");
                let exam_date = exam_date.trim();
                let exam_date = exam_date
                    .split_once(' ')
                    .ok_or(anyhow!(
                        "Unexpected exam_date format (no space): {exam_date}"
                    ))?
                    .1;
                let exam_date = exam_date.to_string() + " " + year + " 00:00";

                let (parse_1, parse_2) = (
                    NaiveDateTime::parse_from_str(&exam_date, "%b %d %Y %R"),
                    NaiveDateTime::parse_from_str(&exam_date, "%B %d %Y %R"),
                );
                let parsed = if let Ok(parse_1) = parse_1 {
                    Ok(parse_1)
                } else {
                    parse_2
                };

                match parsed {
                    Ok(exam_date) => Ok(row
                        .zip(exam_times.clone())
                        .flat_map(|(columns, exam_time)| {
                            columns
                                .children()
                                .flat_map(|class| {
                                    let classes = Class::factory(class.text())?;
                                    Ok::<Vec<_>, anyhow::Error>(
                                        classes
                                            .into_iter()
                                            .map(|inner_class| {
                                                (inner_class, (exam_date.date(), exam_time.clone()))
                                            })
                                            .collect(),
                                    )
                                })
                                .flatten()
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()),
                    Err(exam_date) => Err(anyhow!(exam_date)),
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self(assignments.into_iter().flatten().collect()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CalendarMap(HashMap<String, Calendar>);

impl Deref for CalendarMap {
    type Target = HashMap<String, Calendar>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

lazy_static! {
    static ref SEM_YEAR: Regex = Regex::new(r"(\d+) Exam Calendar$").unwrap();
}

impl TryFrom<Document> for CalendarMap {
    type Error = anyhow::Error;
    fn try_from(value: Document) -> std::result::Result<Self, Self::Error> {
        let semesters: Vec<_> = value
            .find(Name("h2"))
            .map(|node| node.text())
            .filter(|node| node.ends_with("Exam Calendar"))
            .collect();
        let mut semester_years = Vec::with_capacity(semesters.len());
        for sem in &semesters {
            let cap = SEM_YEAR.captures(sem).ok_or(anyhow!(
                "\"{sem}\" does not contained expected pattern \"{}\"",
                SEM_YEAR.as_str()
            ))?[1]
                .to_string();
            semester_years.push(cap)
        }

        let mut sem_idx = 0;
        let cals = value.find(Name("table")).filter_map(|table| {
            if let Ok(cal) = Calendar::try_from((semester_years[sem_idx].as_str(), table)) {
                sem_idx += 1;
                Some(cal)
            } else {
                None
            }
        });

        let sem_len = semesters.len();
        let inner = semesters.into_iter().zip(cals).collect();

        if sem_len != sem_idx {
            bail!(
                "Number of semesters ({}) != number of calendars ({})",
                sem_len,
                sem_idx + 1
            )
        }

        Ok(Self(inner))
    }
}

pub async fn get_calendars<S: AsRef<str>>(url: S) -> Result<CalendarMap> {
    let text = get_page_document(url).await?;
    CalendarMap::try_from(text)
}
