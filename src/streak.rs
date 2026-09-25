use crate::config::StreakState;
use time::OffsetDateTime;

pub enum StreakUpdate {
    AlreadyLoggedToday,
    Incremented,
    StartedNew,
    Reset,
}

fn today_day_number() -> i64 {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    now.date().to_julian_day() as i64
}

pub fn record_practice(state: &mut StreakState, enabled: bool) -> StreakUpdate {
    if !enabled {
        return StreakUpdate::AlreadyLoggedToday;
    }
    let today = today_day_number();

    if state.last_day == today && state.current > 0 {
        return StreakUpdate::AlreadyLoggedToday;
    }

    let update = if state.last_day == 0 {
        state.current = 1;
        StreakUpdate::StartedNew
    } else if state.last_day == today - 1 {
        state.current += 1;
        StreakUpdate::Incremented
    } else if state.last_day == today {
        state.current = state.current.max(1);
        StreakUpdate::AlreadyLoggedToday
    } else {
        state.current = 1;
        StreakUpdate::Reset
    };

    state.last_day = today;
    state.best = state.best.max(state.current);
    update
}
