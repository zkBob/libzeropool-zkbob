use fawkes_crypto::{rand::Rng, ff_uint::NumRepr};
use libzeropool::{POOL_PARAMS, circuit::tx::{CTransferPub, CTransferSec, c_transfer},
    fawkes_crypto::{
        circuit::cs::DebugCS, 
        rand::thread_rng, core::signal::Signal
    }, native::params::PoolBN256, 
};
use std::panic;

use libzeropool::helpers::sample_data::State;

#[test]
fn test_daily_limits_1() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 1, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_2() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 1, 0, true);
}

#[test]
fn test_daily_limits_3() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    transfer(&mut state, &mut rng, 201, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_4() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    transfer(&mut state, &mut rng, 200, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 200, 200, 1000, 1, 0, true);
    transfer(&mut state, &mut rng, 200, 200, 1000, 10, 0, true);
}

#[test]
fn test_daily_limits_5() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    transfer(&mut state, &mut rng, 200, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 200, 200, 1000, 10, 0, true);
    transfer(&mut state, &mut rng, 200, 200, 1000, 9, 0, false);
}

#[test]
fn test_daily_limits_6() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 1, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_7() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 1, 0, true);
}

#[test]
fn test_daily_limits_8() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 201, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_9() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 200, 200, 1000, 0, 0, true);
    deposit(&mut state, &mut rng, 200, 200, 1000, 1, 0, true);
    deposit(&mut state, &mut rng, 200, 200, 1000, 10, 0, true);
}

#[test]
fn test_daily_limits_10() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 200, 200, 1000, 0, 0, true);
    deposit(&mut state, &mut rng, 200, 200, 1000, 10, 0, true);
    deposit(&mut state, &mut rng, 1, 200, 1000, 9, 0, false);
}

#[test]
fn test_daily_limits_11() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    withdrawal(&mut state, &mut rng, 1, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_12() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 200, 1000, 0, 0, true);
    withdrawal(&mut state, &mut rng, 100, 200, 1000, 1, 0, true);
}

#[test]
fn test_daily_limits_13() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    withdrawal(&mut state, &mut rng, 201, 200, 1000, 0, 0, false);
}

#[test]
fn test_daily_limits_14() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);

    deposit(&mut state, &mut rng, 100, 300, 1000, 0, 0, true);
    transfer(&mut state, &mut rng, 100, 300, 1000, 0, 0, true);
    withdrawal(&mut state, &mut rng, 100, 300, 1000, 0, 0, true);

    deposit(&mut state, &mut rng, 100, 300, 1000, 1, 0, true);
    transfer(&mut state, &mut rng, 100, 300, 1000, 1, 0, true);
    withdrawal(&mut state, &mut rng, 100, 300, 1000, 1, 0, true);

    deposit(&mut state, &mut rng, 100, 300, 1000, 2, 0, true);
    transfer(&mut state, &mut rng, 100, 300, 1000, 2, 0, true);
    withdrawal(&mut state, &mut rng, 101, 300, 1000, 2, 0, false);
}

#[test]
fn test_out_note_min_limit_1() {
    let mut rng = thread_rng();
   
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 0, 300, 1000, 0, 0, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 0, 300, 1000, 0, 100, false);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 99, 300, 1000, 0, 100, false);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 100, 300, 1000, 0, 100, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 101, 300, 1000, 0, 100, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 1000, 1000, 1000, 0, 100, true);
}

#[test]
fn test_transfer_limit_1() {
    let mut rng = thread_rng();
    
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 1, 300, 100, 0, 0, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 99, 300, 100, 0, 0, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 100, 300, 100, 0, 0, true);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 101, 300, 100, 0, 0, false);

    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    transfer(&mut state, &mut rng, 1000, 1000, 100, 0, 0, false);
}

#[test]
fn test_negative_deposit() {
    let mut rng = thread_rng();
    let mut state = State::sample_deterministic_state(&mut rng, &*POOL_PARAMS, 1000);
    let deposit_amount = -1000 as i64;

    let mut starting_balance = state.items[state.account_id].0.b.to_num();
    for &i in state.note_id.iter() {
        starting_balance+=state.items[i].1.b.to_num();
    }

    deposit(&mut state, &mut rng, deposit_amount, 3000, 1000, 0, 0, true);

    let acc = state.items[state.account_id];

    let delta = starting_balance - acc.0.b.to_num();

    
    assert_eq!(delta.to_uint(), NumRepr::from(- deposit_amount as u64));

}

fn deposit<R: Rng>(
    state: &mut State<PoolBN256>, 
    rng: &mut R, 
    amount: i64, 
    daily_limit: u64, 
    transfer_limit: u64, 
    day: u64, 
    out_note_min: u64, 
    success: bool,
) {
    let (public, secret) = state.sample_deterministic_deposit(rng, &*POOL_PARAMS, amount, day, daily_limit, transfer_limit, out_note_min);
    let params = POOL_PARAMS.clone();
    let result = panic::catch_unwind(move || {
        let ref cs = DebugCS::rc_new();
        let ref p = CTransferPub::alloc(cs, Some(&public));
        let ref s = CTransferSec::alloc(cs, Some(&secret));
        c_transfer(p, s, &params)
    });
    assert_eq!(result.is_ok(), success);
}

fn transfer<R: Rng>(
    state: &mut State<PoolBN256>, 
    rng: &mut R, 
    amount: u64, 
    daily_limit: u64,
    transfer_limit: u64, 
    day: u64, 
    out_note_min: u64, 
    success: bool,
) {
    let (public, secret) = state.sample_deterministic_transfer(rng, &*POOL_PARAMS, amount, day, daily_limit, transfer_limit, out_note_min);
    let params = POOL_PARAMS.clone();
    let result = panic::catch_unwind(move || {
        let ref cs = DebugCS::rc_new();
        let ref p = CTransferPub::alloc(cs, Some(&public));
        let ref s = CTransferSec::alloc(cs, Some(&secret));
        c_transfer(p, s, &params)
    });
    assert_eq!(result.is_ok(), success);
}

fn withdrawal<R: Rng>(
    state: &mut State<PoolBN256>, 
    rng: &mut R, amount: u64, 
    daily_limit: u64, 
    transfer_limit: u64,
    day: u64,
    out_note_min: u64, 
    success: bool
) {
    let (public, secret) = state.sample_deterministic_withdrawal(rng, &*POOL_PARAMS, amount, day, daily_limit, transfer_limit, out_note_min);
    let params = POOL_PARAMS.clone();
    let result = panic::catch_unwind(move || {
        let ref cs = DebugCS::rc_new();
        let ref p = CTransferPub::alloc(cs, Some(&public));
        let ref s = CTransferSec::alloc(cs, Some(&secret));
        c_transfer(p, s, &params)
    });
    assert_eq!(result.is_ok(), success);
}