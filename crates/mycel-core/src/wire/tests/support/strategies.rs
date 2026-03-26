use super::*;

pub(crate) fn valid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        any::<bool>(),
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_map(
            |(
                year,
                month,
                day,
                hour,
                minute,
                second,
                use_z,
                offset_sign,
                offset_hour,
                offset_minute,
            )| {
                if use_z {
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
                } else {
                    format!(
                        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}:{offset_minute:02}"
                    )
                }
            },
        )
}

pub(crate) fn invalid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_flat_map(
            |(year, month, day, hour, minute, second, offset_sign, offset_hour, offset_minute)| {
                let no_t = format!(
                    "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z"
                );
                let slash_date = format!(
                    "{year:04}/{month:02}/{day:02}T{hour:02}:{minute:02}:{second:02}Z"
                );
                let no_offset_colon = format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}{offset_minute:02}"
                );
                let missing_offset =
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
                let short_time = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}Z");
                prop_oneof![
                    Just(no_t),
                    Just(slash_date),
                    Just(no_offset_colon),
                    Just(missing_offset),
                    Just(short_time),
                ]
            },
        )
}

pub(crate) fn invalid_object_type_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_type must be unsupported", |value| {
        !matches!(
            value.as_str(),
            "document" | "block" | "patch" | "revision" | "view" | "snapshot"
        )
    })
}

pub(crate) fn invalid_canonical_object_id_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_id must violate canonical prefix rules", |value| {
        !["patch:", "rev:", "view:", "snap:"]
            .iter()
            .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
    })
}
