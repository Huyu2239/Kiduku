use std::collections::HashSet;

use serenity::model::prelude::UserId;

pub fn calculate_read_status(
    mentioned_users: &[UserId],
    reaction_users: &[UserId],
) -> (Vec<UserId>, Vec<UserId>) {
    let mentioned = dedup_sorted(mentioned_users.iter().copied());
    let mentioned_set = mentioned.iter().map(|id| id.get()).collect::<HashSet<_>>();

    let read = dedup_sorted(
        reaction_users
            .iter()
            .copied()
            .filter(|id| mentioned_set.contains(&id.get())),
    );
    let read_set = read.iter().map(|id| id.get()).collect::<HashSet<_>>();

    let unread = mentioned
        .into_iter()
        .filter(|id| !read_set.contains(&id.get()))
        .collect::<Vec<_>>();

    (read, unread)
}

fn dedup_sorted<I>(ids: I) -> Vec<UserId>
where
    I: IntoIterator<Item = UserId>,
{
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort_by_key(|id| id.get());
    ids.dedup_by_key(|id| id.get());
    ids
}

#[cfg(test)]
mod tests {
    use serenity::model::prelude::UserId;

    use super::calculate_read_status;

    #[test]
    fn calculates_read_and_unread_users() {
        let mentioned = vec![UserId::new(3), UserId::new(1), UserId::new(2)];
        let reactions = vec![UserId::new(2), UserId::new(9), UserId::new(2)];

        let (read, unread) = calculate_read_status(&mentioned, &reactions);

        assert_eq!(read, vec![UserId::new(2)]);
        assert_eq!(unread, vec![UserId::new(1), UserId::new(3)]);
    }
}
