use libzeropool::{POOL_PARAMS, 
    circuit::delegated_deposit::{CDelegatedDepositBatchPub, CDelegatedDepositBatchSec, check_delegated_deposit_batch, CDelegatedDeposit, num_to_iter_bits_be},
    native::note::Note,
    native::delegated_deposit::{DelegatedDeposit},
    helpers::sample_data::serialize_scalars_and_delegated_deposits_be,
    fawkes_crypto::{
        circuit::{
            cs::{CS, DebugCS},
            num::CNum,
            bool::CBool
        }, 
        core::{signal::Signal, sizedvec::SizedVec},
        rand::{thread_rng, Rng},
    }, 
};

use std::time::Instant;
use libzeropool::fawkes_crypto::engines::bn256::Fr;



#[test]
fn test_check_delegated_deposit_batch() {
    let ref cs = DebugCS::rc_new();
    let ref p = CDelegatedDepositBatchPub::alloc(cs, None);
    let ref s = CDelegatedDepositBatchSec::alloc(cs, None);

    
    let mut n_gates = cs.borrow().num_gates();
    let start = Instant::now();
    check_delegated_deposit_batch(p, s, &*POOL_PARAMS);
    let duration = start.elapsed();
    n_gates=cs.borrow().num_gates()-n_gates;

    println!("tx constraints = {}", n_gates);
    println!("Time elapsed in check_delegated_deposit_batch() is: {:?}", duration);

}    

#[test]
fn test_bitify_delegated_deposits_be() {
    const N_ITEMS:usize = 10;
    let mut rng = thread_rng();

    let deposits:SizedVec<_,{N_ITEMS}> = (0..N_ITEMS).map(|_| {
        let n = Note::sample(&mut rng, &*POOL_PARAMS);
        DelegatedDeposit {
            d:n.d,
            p_d:n.p_d,
            b:n.b,
        }
    }).collect();

    let och = rng.gen();
    let out_account_hash = rng.gen();

    let data = serialize_scalars_and_delegated_deposits_be(och, out_account_hash, deposits.as_slice());

    let bitlen = data.len()*8;

    let bits = (0..bitlen).map(|i| {
        let byte = data[i/8];
        let bit = byte & (1 << (i%8));
        bit != 0
    }).collect::<Vec<_>>();

    let ref cs = DebugCS::rc_new();

    let c_deposits:SizedVec<CDelegatedDeposit<DebugCS<Fr>>,{N_ITEMS}> = Signal::alloc(cs, Some(deposits).as_ref());
    let c_true = CBool::from_const(cs, &true);
    let c_och = CNum::alloc(cs, Some(och).as_ref());
    let c_out_account_hash = CNum::alloc(cs, Some(out_account_hash).as_ref());
    
    let c_bits = num_to_iter_bits_be(&c_och)
    .chain(std::iter::repeat(c_true).take(32))
    .chain(num_to_iter_bits_be(&c_out_account_hash))
    .chain(c_deposits.iter().flat_map(
        |d| d.to_iter_bits_be()
    )).collect::<Vec<_>>();

    assert_eq!(bits.len(), c_bits.len());

    for (i, (b, c_b)) in bits.iter().zip(c_bits.iter()).enumerate() {
        assert_eq!(*b, c_b.get_value().unwrap(), "bit {} is not equal", i);
    }

}
 