use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{PoisonError, RwLock};
use chrono::{Datelike, DateTime, Local, TimeDelta, Timelike};
use derive_builder::Builder;


#[derive(Debug, Default, Clone)]
pub enum Rotation {
    #[default]
    Daily,
    Hourly,
    Minutely,
    Never,
}

impl Rotation {
    fn next_time(&self, current_time: Time) -> Option<Time> {
        let time = match self {
            Rotation::Daily => {
                Some(current_time.add(TimeDelta::days(1)))
            }
            Rotation::Hourly => {
                Some(current_time.add(TimeDelta::hours(1)))
            }
            Rotation::Minutely => {
                Some(current_time.add(TimeDelta::minutes(1)))
            }
            _ => None
        };
        time.and_then(|t| self.round_time(t))
    }

    fn round_time(&self, time: DateTime<Local>) -> Option<Time> {
        match self {
            Rotation::Daily => {
                Some(time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap())
            }
            Rotation::Hourly => {
                Some(time.with_minute(0).unwrap().with_second(0).unwrap())
            }
            Rotation::Minutely => {
                Some(time.with_second(0).unwrap())
            }
            _ => None
        }
    }

    pub const DAILY: &'static str = "%Y-%m-%d";
    pub const HOURLY: &'static str = "%Y-%m-%d-%H";
    pub const MINUTELY: &'static str = "%Y-%m-%d-%H-%M";
    pub const NEVER: &'static str = Self::DAILY;

    fn date_format(&self) -> &'static str {
        match self {
            Rotation::Daily => Self::DAILY,
            Rotation::Hourly => Self::HOURLY,
            Rotation::Minutely => Self::MINUTELY,
            Rotation::Never => Self::NEVER,
        }
    }
}

pub struct TracingFileAppender<'a, 'c> {
    state: State<'a, 'c>,
    writer: RwLock<File>,
}

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Appender<'c> {
    rotation: Rotation,
    prefix: Option<&'c str>,
    suffix: Option<&'c str>,
}

struct State<'a, 'c> {
    rotation: Rotation,
    next_time: AtomicUsize,
    directory: &'a Path,
    prefix: Option<&'c str>,
    suffix: Option<&'c str>,
}

impl<'a, 'c> State<'a, 'c> {
    pub fn new<'b: 'a, T: AsRef<Path> + 'b + ?Sized>(
        now: Time,
        rotation: Rotation,
        directory: &'b T,
        prefix: Option<&'c str>,
        suffix: Option<&'c str>,
    ) -> Result<(Self, RwLock<File>), anyhow::Error> {
        let next_time = rotation.next_time(now);
        let state = State {
            rotation,
            next_time: AtomicUsize::new(next_time.map(|x| x.timestamp() as usize).unwrap_or(0)),
            directory: directory.as_ref(),
            prefix,
            suffix,
        };
        let filename = state.join_date(now);
        let writer_file = Self::create_writer(directory.as_ref(), &filename)?;
        let writer = RwLock::new(writer_file);
        Ok((state, writer))
    }

    pub fn should_rollover(&self, time: Time) -> Option<usize> {
        let next_time = self.next_time.load(Ordering::Acquire);
        if next_time == 0 {
            return None;
        }
        if time.timestamp() >= next_time as i64 {
            return Some(next_time);
        }
        None
    }

    fn add_date(&self, now: Time, current_timestamp: usize) -> bool {
        let next_time = self
            .rotation
            .next_time(now)
            .map(|date| date.timestamp() as usize)
            .unwrap_or(0);
        let result = self.next_time
            .compare_exchange(current_timestamp, next_time, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        result
    }

    fn join_date(&self, time: Time) -> String {
        let format = self.rotation.date_format();
        let format_time = time.format(format);
        match (
            &self.rotation,
            &self.prefix,
            &self.suffix,
        ) {
            (Rotation::Never, Some(filename), None) => filename.to_string(),
            (Rotation::Never, Some(filename), Some(suffix)) => format!("{}.{}", filename, suffix),
            (Rotation::Never, None, Some(suffix)) => suffix.to_string(),
            (_, Some(filename), Some(suffix)) => format!("{}.{}.{}", filename, format_time, suffix),
            (_, Some(filename), None) => format!("{}.{}", filename, format_time),
            (_, None, Some(suffix)) => format!("{}.{}", format_time, suffix),
            (_, None, None) => format_time.to_string(),
        }
    }

    fn refresh_writer(&self, now: Time, file: &mut File) {
        let filename = self.join_date(now);
        match Self::create_writer(&self.directory, &filename) {
            Ok(new_file) => {
                if let Err(err) = file.flush() {
                    eprintln!("Couldn't flush previous writer: {}", err);
                }
                *file = new_file;
            }
            Err(err) => eprintln!("Couldn't create writer for logs: {}", err),
        }
    }
    pub(crate) fn create_writer(directory: &Path, filename: &str) -> Result<File, anyhow::Error> {
        let mut open_options = OpenOptions::new();
        open_options.append(true);
        open_options.create(true);
        let path = directory.join(filename);
        let new_file = open_options.open(path.as_path());
        if new_file.is_err() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
                return Ok(open_options.open(path)?);
            }
        }
        new_file.map_err(|e| e.into())
    }

    #[inline]
    pub fn now() -> Time {
        chrono::Local::now()
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::path::Path;
    use chrono::{Datelike, DateTime, Local, NaiveDate, NaiveDateTime, TimeDelta, TimeZone};
    use crate::file_appender::{Rotation, State};

    #[test]
    fn test_state_add_date_fail() -> Result<(), anyhow::Error> {
        //时间未达一个小时，不能触发rollover
        let now = Local.with_ymd_and_hms(2024, 12, 12, 12, 0, 0).unwrap();
        let state = State::new(
            now,
            Rotation::Hourly,
            "logs",
            None,
            None,
        )?.0;
        let now = now.add(TimeDelta::minutes(59));
        let current_timestamp = state.should_rollover(now);
        assert!(current_timestamp.is_none());
        Ok(())
    }

    #[test]
    fn test_state_add_date_ok() -> Result<(), anyhow::Error> {
        //时间过去一个小时，应该触发一次rollover
        let now = Local.with_ymd_and_hms(2024, 12, 12, 12, 0, 0).unwrap();
        let state = State::new(
            now,
            Rotation::Hourly,
            "logs",
            None,
            None,
        )?.0;
        let now = now.add(TimeDelta::hours(1));
        let current_timestamp = state.should_rollover(now);
        assert!(current_timestamp.is_some());
        let current_timestamp = current_timestamp.unwrap();
        let res = state.add_date(now, current_timestamp);
        assert_eq!(true, res);
        Ok(())
    }

    #[test]
    fn test_state_join_date_format() -> Result<(), anyhow::Error> {
        assert_eq!(State::new(
            State::now(),
            Rotation::Hourly,
            "logs",
            None,
            None,
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 12, 0, 0).unwrap()), "2024-12-12-12");
        assert_eq!(State::new(
            State::now(),
            Rotation::Daily,
            "logs",
            None,
            None,
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "2024-12-12");
        assert_eq!(State::new(
            State::now(),
            Rotation::Daily,
            "logs",
            None,
            Some("log"),
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "2024-12-12.log");
        assert_eq!(State::new(
            State::now(),
            Rotation::Daily,
            "logs",
            Some("app"),
            Some("log"),
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "app.2024-12-12.log");
        assert_eq!(State::new(
            State::now(),
            Rotation::Never,
            "logs",
            Some("app"),
            Some("log"),
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "app.log");
        assert_eq!(State::new(
            State::now(),
            Rotation::Never,
            "logs",
            None,
            Some("log"),
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "log");
        assert_eq!(State::new(
            State::now(),
            Rotation::Never,
            "logs",
            Some("log"),
            None,
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "log");
        assert_eq!(State::new(
            State::now(),
            Rotation::Never,
            "logs",
            None,
            None,
        )?.0.join_date(Local.with_ymd_and_hms(2024, 12, 12, 0, 0, 0).unwrap()), "2024-12-12");
        Ok(())
    }
}

type Time = DateTime<Local>;

impl<'a, 'c> TracingFileAppender<'a, 'c> {
    pub fn from_builder<'b: 'a, T: AsRef<Path> + 'b + ?Sized>(builder: AppenderBuilder<'c>, directory: &'b T) -> Result<Self, anyhow::Error> {
        let Appender {
            rotation,
            prefix,
            suffix,
        } = builder.build()?;
        let now = State::now();
        let (state, writer) = State::new(
            now,
            rotation,
            directory,
            prefix,
            suffix,
        )?;
        Ok(TracingFileAppender {
            state,
            writer,
        })
    }
}

impl<'a, 'c> Write for TracingFileAppender<'a, 'c> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let now = State::now();
        let writer = self.writer.get_mut().unwrap_or_else(PoisonError::into_inner);
        if let Some(current_timestamp) = self.state.should_rollover(now) {
            let _a = self.state.add_date(now, current_timestamp);
            self.state.refresh_writer(now, writer);
        }
        writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.get_mut().unwrap_or_else(PoisonError::into_inner).flush()
    }
}