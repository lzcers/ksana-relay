use crate::nostr::{Event, Filter};

pub struct EventFilter;

impl EventFilter {
    pub fn any_filter(evt: &Event, filters: &Vec<Filter>) -> bool {
        filters.into_iter().any(|filter| Self::filter(&evt, filter))
    }

    pub fn filter(evt: &Event, filter: &Filter) -> bool {
        let Filter {
            ids,
            authors,
            kinds,
            e,
            p,
            since,
            until,
            limit: _,
        } = filter;

        macro_rules! check_filter_vec {
            ($(($field:ident, $key:ident)),*) => {
                $(
                if $field.is_empty() {
                    true
                } else {
                    $field.contains(&evt.$key)
                } &&)* true
            };
        }
        if check_filter_vec! {(ids, id), (authors, pubkey), (kinds, kind), (e, id), (p, pubkey)}
            && since.as_ref().map_or(true, |s| evt.created_at > *s)
            && until.as_ref().map_or(true, |s| evt.created_at < *s)
        {
            true
        } else {
            false
        }
    }
}
