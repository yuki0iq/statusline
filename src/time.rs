pub fn microseconds_to_string(total: u64) -> Option<String> {
    let (_usec, total) = (total % 1000, total / 1000);
    if total < 100 {
        return None;
    }
    let (msec, total) = (total % 1000, total / 1000);
    let (sec, total) = (total % 60, total / 60);
    let (min, total) = (total % 60, total / 60);
    let (hrs, total) = (total % 24, total / 24);
    let (day, week) = (total % 7, total / 7);

    let times = vec![
        (week, "w"),
        (day, "d"),
        (hrs, "h"),
        (min, "m"),
        (sec, "s"),
        (msec, "ms"),
    ];
    let mut iter = times.iter().peekable();
    while let Some((0, _)) = iter.peek() {
        iter.next();
    }
    Some(
        iter.take(2)
            .map(|(val, ch)| val.to_string() + ch)
            .collect::<Vec<_>>()
            .join(" "),
    )
}
