use common::test_storage;
use crypto::{ed25519::private::PrivateKey, hash::sha3};
use std::collections::HashSet;
use storage::WriteSet;
use types::ai::{
    query::{NodeResult, Query},
    request::AiRequest,
};

mod common;

#[test]
pub fn test_put_load_query() {
    let (_, store) = test_storage();
    let alice = PrivateKey::generate();
    let mut ws = WriteSet::default();
    let user_seq = store
        .sequence_table
        .increment_and_get(&alice.public_key(), &mut ws)
        .unwrap();
    let query = test_query(user_seq, "hello world", &alice);
    store.query_table.put_query(&query, &mut ws).unwrap();
    store.commit(ws).unwrap();
    let query_1 = store.query_table.get_query(&query.id).unwrap().unwrap();
    assert_eq!(query, query_1);

    let ids = store.query_table.get_in_progress_ids(10, 0).unwrap();
    assert_eq!(ids.len(), 0);
}

#[test]
pub fn test_update_query() {
    let (_, store) = test_storage();
    let alice = PrivateKey::generate();
    let mut ws = WriteSet::default();
    let user_seq = store
        .sequence_table
        .increment_and_get(&alice.public_key(), &mut ws)
        .unwrap();
    let query = test_query(user_seq, "hello world", &alice);
    store.query_table.put_query(&query, &mut ws).unwrap();
    store.commit(ws).unwrap();
    let mut query_1 = store.query_table.get_query(&query.id).unwrap().unwrap();
    assert_eq!(query, query_1);

    query_1
        .response
        .push(NodeResult::SentRequest(PrivateKey::generate().public_key()));
    let mut ws = WriteSet::default();
    store.query_table.put_query(&query_1, &mut ws).unwrap();
    store.commit(ws).unwrap();
    let query_2 = store.query_table.get_query(&query.id).unwrap().unwrap();
    assert_eq!(query_1, query_2);

    let ids = store.query_table.get_in_progress_ids(10, 0).unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], query.id);

    query_1.response.clear();
    let mut ws = WriteSet::default();
    store.query_table.put_query(&query_1, &mut ws).unwrap();
    store.commit(ws).unwrap();
    let query_3 = store.query_table.get_query(&query.id).unwrap().unwrap();
    assert_eq!(query_1, query_3);

    let ids = store.query_table.get_in_progress_ids(10, 0).unwrap();
    assert_eq!(ids.len(), 0);
}

#[test]
pub fn test_in_progress_ids() {
    let (_, store) = test_storage();
    let alice = PrivateKey::generate();
    let mut in_progress = HashSet::new();
    let mut ws = WriteSet::default();
    for i in 0..30 {
        let user_seq = store
            .sequence_table
            .increment_and_get(&alice.public_key(), &mut ws)
            .unwrap();
        let mut query = test_query(user_seq, &format!("{}", i), &alice);
        if i % 2 == 0 {
            in_progress.insert(query.id);
            query
                .response
                .push(NodeResult::SentRequest(PrivateKey::generate().public_key()));
        }
        store.query_table.put_query(&query, &mut ws).unwrap();
    }
    store.commit(ws).unwrap();

    let ids = store.query_table.get_in_progress_ids(10, 0).unwrap();
    assert_eq!(ids.len(), 10);
    for id in ids.iter() {
        assert!(in_progress.remove(id));
    }

    let ids = store.query_table.get_in_progress_ids(10, 10).unwrap();
    assert_eq!(ids.len(), 5);

    for id in ids.iter() {
        assert!(in_progress.remove(id));
    }
    assert!(in_progress.is_empty());
}

#[test]
pub fn test_users_ids() {
    tracing_subscriber::fmt::init();
    let (_, store) = test_storage();
    let alice = PrivateKey::generate();
    let bob = PrivateKey::generate();
    let mut alice_set = Vec::with_capacity(500);
    let mut bob_set = Vec::with_capacity(500);

    for i in 0..1000 {
        let mut ws = WriteSet::default();
        let key = if i % 2 == 0 { &alice } else { &bob };
        let user_seq = store
            .sequence_table
            .increment_and_get(&key.public_key(), &mut ws)
            .unwrap();
        let query = test_query(user_seq, &format!("{}", i), key);
        store.query_table.put_query(&query, &mut ws).unwrap();
        if i % 2 == 0 {
            alice_set.push(query.id);
        } else {
            bob_set.push(query.id);
        }
        store.commit(ws).unwrap();
    }

    let alice_ids = store
        .query_table
        .users_query_ids(&alice.public_key(), 500, 0)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(alice_ids, alice_set);

    let bob_ids = store
        .query_table
        .users_query_ids(&bob.public_key(), 500, 0)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(bob_ids, bob_set);
}

fn test_query(sequence: u64, msg: &str, key: &PrivateKey) -> Query {
    let request = AiRequest::new(msg.to_string(), vec![], key.public_key());
    let request = request.sign(key).unwrap();

    Query {
        id: sha3(&(sequence, &request)),
        request,
        response: vec![],
        sequence,
    }
}
