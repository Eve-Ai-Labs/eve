use common::test_storage;
use storage::WriteSet;

mod common;

#[test]
fn test_sequence_table() {
    let (_, store) = test_storage();

    let mut users = (0..10)
        .map(|_| {
            (
                crypto::ed25519::private::PrivateKey::generate().public_key(),
                0,
            )
        })
        .collect::<Vec<(_, u64)>>();

    let users_count = users.len();

    for _ in 0..100 {
        let (key, expected) = &mut users[rand::random::<usize>() % users_count];
        let mut ws = WriteSet::default();
        let seq = store
            .sequence_table
            .increment_and_get(key, &mut ws)
            .unwrap();
        store.commit(ws).unwrap();
        assert_eq!(seq, *expected + 1);
        *expected = seq;
    }

    for (key, expected) in users {
        let seq = store.sequence_table.get(&key).unwrap();
        assert_eq!(seq, expected);
    }
}
