use crate::{
    fawkes_crypto::{
        ff_uint::{Num, seedbox::{SeedboxChaCha20, SeedBox, SeedBoxGen}},
        borsh::{BorshSerialize, BorshDeserialize},
        native::ecc::{EdwardsPoint},

    },
    native::{
        account::Account,
        note::Note,
        params::PoolParams,
        key::{derive_key_a, derive_key_p_d}
    },
    constants::{self}
};

use sha3::{Digest, Keccak256};

use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::AeadMutInPlace};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::aead::heapless::Vec as HeaplessVec;

fn keccak256(data:&[u8])->[u8;constants::U256_SIZE] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut res = [0u8;constants::U256_SIZE];
    res.iter_mut().zip(hasher.finalize().into_iter()).for_each(|(l,r)| *l=r);
    res
}

//key stricly assumed to be unique for all messages. Using this function with multiple messages and one key is insecure!
fn symcipher_encode(key:&[u8], data:&[u8])->Vec<u8> {
    assert!(key.len()==constants::U256_SIZE);
    let nonce = Nonce::from_slice(&constants::ENCRYPTION_NONCE);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher.encrypt(nonce, data.as_ref()).unwrap()
}

//key stricly assumed to be unique for all messages. Using this function with multiple messages and one key is insecure!
fn symcipher_decode(key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    assert!(key.len()==constants::U256_SIZE);
    let nonce = Nonce::from_slice(&constants::ENCRYPTION_NONCE);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher.decrypt(nonce, data).ok()
}

//key stricly assumed to be unique for all messages. Using this function with multiple messages and one key is insecure!
fn symcipher_decode_in_place<const N: usize>(key: &[u8], ciphertext: &[u8]) -> Option<HeaplessVec<u8, N>> {
    assert!(key.len()==constants::U256_SIZE);
    let nonce = Nonce::from_slice(&constants::ENCRYPTION_NONCE);
    let mut cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut buffer = HeaplessVec::<u8, N>::from_slice(ciphertext).ok()?;
    cipher.decrypt_in_place(nonce, b"", &mut buffer).ok()?;
    Some(buffer)
}

fn decrypt_note<P: PoolParams>(key: &[u8], ciphertext: &[u8]) -> Option<Note<P::Fr>> {
    // 76 bytes is a note size for bls12-381, buffer needs 16-bytes overhead for auth tag
    const NOTE_BUFFER_SIZE: usize = 76 + 16;
    if ciphertext.len() <= NOTE_BUFFER_SIZE {
        let plain = symcipher_decode_in_place::<NOTE_BUFFER_SIZE>(key, ciphertext)?;
        Some(Note::try_from_slice(&plain).ok()?)
    } else {
        let plain = symcipher_decode(key, ciphertext)?;
        Some(Note::try_from_slice(&plain).ok()?)
    }
}

fn decrypt_account<P: PoolParams>(key: &[u8], ciphertext: &[u8]) -> Option<Account<P::Fr>> {
    // 86 bytes is an account size for bls12-381, buffer needs 16-bytes overhead for auth tag
    const ACCOUNT_BUFFER_SIZE: usize = 86 + 16;
    if ciphertext.len() <= ACCOUNT_BUFFER_SIZE {
        let plain = symcipher_decode_in_place::<ACCOUNT_BUFFER_SIZE>(key, ciphertext)?;
        Some(Account::try_from_slice(&plain).ok()?)
    } else {
        let plain = symcipher_decode(key, ciphertext)?;
        Some(Account::try_from_slice(&plain).ok()?)
    }
}

fn decrypt_shared_secrets(key: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
    // It's suitable for most transactions
    const SHARED_SECRETS_BUFFER_SIZE: usize = 32 * 10 + 16;
    if ciphertext.len() <= SHARED_SECRETS_BUFFER_SIZE {
        let plain = symcipher_decode_in_place::<SHARED_SECRETS_BUFFER_SIZE>(key, ciphertext)?;
        // It's still should be better in our case because it doesn't allocate memory when decryption failed
        Some(plain.to_vec())
    } else {
        let plain = symcipher_decode(key, ciphertext)?;
        Some(plain)
    }
}

pub fn encrypt<P: PoolParams>(
    entropy: &[u8],
    eta:Num<P::Fr>,
    account: Account<P::Fr>,
    note: &[Note<P::Fr>],
    params:&P
) -> Vec<u8> {
    let nozero_notes_num = note.len();
    let nozero_items_num = nozero_notes_num+1;


    let mut sb = SeedboxChaCha20::new_with_salt(entropy);

    let account_data = {
        let mut account_key = [0u8;constants::U256_SIZE];
        sb.fill_bytes(&mut account_key);
        let account_ciphertext = symcipher_encode(&account_key, &account.try_to_vec().unwrap());
        (account_key, account_ciphertext)
    };
    
    
    let notes_data = note.iter().map(|e|{
        let a:Num<P::Fs> = sb.gen();
        let p_d = EdwardsPoint::subgroup_decompress(e.p_d, params.jubjub()).unwrap();
        let ecdh =  p_d.mul(a, params.jubjub());
        let key = keccak256(&ecdh.x.try_to_vec().unwrap());
        let ciphertext = symcipher_encode(&key, &e.try_to_vec().unwrap());
        let a_pub = derive_key_p_d(e.d.to_num(), a, params); 
        (a_pub.x, key, ciphertext)
        
    }).collect::<Vec<_>>();

    let shared_secret_data = {
        let a_p_pub = derive_key_a(sb.gen(), params);
        let ecdh = a_p_pub.mul(eta.to_other_reduced(), params.jubjub());
        let key = keccak256(&ecdh.x.try_to_vec().unwrap());
        let text:Vec<u8> = core::iter::once(&account_data.0[..]).chain(notes_data.iter().map(|e| &e.1[..])).collect::<Vec<_>>().concat();
        let ciphertext = symcipher_encode(&key, &text);
        (a_p_pub.x, ciphertext)
    };

    let mut res = vec![];

    (nozero_items_num as u32).serialize(&mut res).unwrap();
    account.hash(params).serialize(&mut res).unwrap();

    for e in note.iter() {
        e.hash(params).serialize(&mut res).unwrap();
    }
    shared_secret_data.0.serialize(&mut res).unwrap();
    res.extend(&shared_secret_data.1);

    res.extend(&account_data.1);

    notes_data.iter().for_each(|nd|{
        nd.0.serialize(&mut res).unwrap();
        res.extend(&nd.2);
    });

    res
}


fn buf_take<'a>(memo: &mut &'a[u8], size:usize) -> Option<&'a[u8]> {
    if memo.len() < size {
        None
    } else {
        let res = &memo[0..size];
        *memo = &memo[size..];
        Some(res)
    }
}

pub fn decrypt_out<P: PoolParams>(eta:Num<P::Fr>, mut memo:&[u8], params:&P)->Option<(Account<P::Fr>, Vec<Note<P::Fr>>)> {
    let num_size = constants::num_size_bits::<P::Fr>()/8;
    let account_size = constants::account_size_bits::<P::Fr>()/8;
    let note_size = constants::note_size_bits::<P::Fr>()/8;


    let nozero_items_num = u32::deserialize(&mut memo).ok()? as usize;
    if nozero_items_num == 0 {
        return None;
    }

    let nozero_notes_num = nozero_items_num - 1;
    let shared_secret_ciphertext_size = nozero_items_num * constants::U256_SIZE + constants::POLY_1305_TAG_SIZE;

    let account_hash = Num::deserialize(&mut memo).ok()?;
    let note_hash = (0..nozero_notes_num).map(|_| Num::deserialize(&mut memo)).collect::<Result<Vec<_>, _>>().ok()?;

    let shared_secret_text = {
        let a_p = EdwardsPoint::subgroup_decompress(Num::deserialize(&mut memo).ok()?, params.jubjub())?;
        let ecdh = a_p.mul(eta.to_other_reduced(), params.jubjub());
        let key = {
            let mut x: [u8; 32] = [0; 32];
            ecdh.x.serialize(&mut &mut x[..]).unwrap();
            keccak256(&x)
        };
        let ciphertext = buf_take(&mut memo, shared_secret_ciphertext_size)?;
        decrypt_shared_secrets(&key, ciphertext)?
    };
    let mut shared_secret_text_ptr =&shared_secret_text[..];

    let account_key= <[u8;constants::U256_SIZE]>::deserialize(&mut shared_secret_text_ptr).ok()?;
    let note_key = (0..nozero_notes_num).map(|_| <[u8;constants::U256_SIZE]>::deserialize(&mut shared_secret_text_ptr)).collect::<Result<Vec<_>,_>>().ok()?;

    let account_ciphertext = buf_take(&mut memo, account_size+constants::POLY_1305_TAG_SIZE)?;
    let account = decrypt_account::<P>(&account_key, account_ciphertext)?;

    if account.hash(params)!= account_hash {
        return None;
    }

    let note = (0..nozero_notes_num).map(|i| {
        buf_take(&mut memo, num_size)?;
        let ciphertext = buf_take(&mut memo, note_size+constants::POLY_1305_TAG_SIZE)?;
        let note = decrypt_note::<P>(&note_key[i], ciphertext)?;
        if note.hash(params) != note_hash[i] {
            None
        } else {
            Some(note)
        }
    }).collect::<Option<Vec<_>>>()?;
    
    Some((account, note))
}

fn _decrypt_in<P: PoolParams>(eta:Num<P::Fr>, mut memo:&[u8], params:&P)->Option<Vec<Option<Note<P::Fr>>>> {
    let num_size = constants::num_size_bits::<P::Fr>()/8;
    let account_size = constants::account_size_bits::<P::Fr>()/8;
    let note_size = constants::note_size_bits::<P::Fr>()/8;


    let nozero_items_num = u32::deserialize(&mut memo).ok()? as usize;
    if nozero_items_num == 0 {
        return None;
    }

    let nozero_notes_num = nozero_items_num - 1;
    let shared_secret_ciphertext_size = nozero_items_num * constants::U256_SIZE + constants::POLY_1305_TAG_SIZE;

    buf_take(&mut memo, num_size)?;
    let note_hash = (0..nozero_notes_num).map(|_| Num::deserialize(&mut memo)).collect::<Result<Vec<_>, _>>().ok()?;

    buf_take(&mut memo, num_size)?;
    buf_take(&mut memo, shared_secret_ciphertext_size)?;
    buf_take(&mut memo, account_size+constants::POLY_1305_TAG_SIZE)?;


    let note = (0..nozero_notes_num).map(|i| {
        let a_pub = EdwardsPoint::subgroup_decompress(Num::deserialize(&mut memo).ok()?, params.jubjub())?;
        let ecdh = a_pub.mul(eta.to_other_reduced(), params.jubjub());
        
        let key = {
            let mut x: [u8; 32] = [0; 32];
            ecdh.x.serialize(&mut &mut x[..]).unwrap();
            keccak256(&x)
        };

        let ciphertext = buf_take(&mut memo, note_size+constants::POLY_1305_TAG_SIZE)?;
        let note = decrypt_note::<P>(&key, ciphertext)?;
        if note.hash(params) != note_hash[i] {
            None
        } else {
            Some(note)
        }
    }).collect::<Vec<Option<_>>>();

    Some(note)
}

pub fn decrypt_in<P: PoolParams>(eta:Num<P::Fr>, memo:&[u8], params:&P)->Vec<Option<Note<P::Fr>>> {
    if let Some(res) = _decrypt_in(eta, memo, params) {
        res
    } else {
        vec![]
    }
}
