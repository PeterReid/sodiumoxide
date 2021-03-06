#![macro_escape]
macro_rules! stream_module (($stream_name:ident, 
                             $xor_name:ident, 
                             $keybytes:expr, 
                             $noncebytes:expr) => (

#[link(name = "sodium")]
extern {
    fn $stream_name(c: *mut u8,
                    clen: c_ulonglong,
                    n: *u8,
                    k: *u8) -> c_int;
    fn $xor_name(c: *mut u8,
                 m: *u8,
                 mlen: c_ulonglong,
                 n: *u8,
                 k: *u8) -> c_int;
}

pub static KEYBYTES: uint = $keybytes;
pub static NONCEBYTES: uint = $noncebytes;

/**
 * `Key` for symmetric encryption
 *
 * When a `Key` goes out of scope its contents
 * will be zeroed out
 */
pub struct Key(pub [u8, ..KEYBYTES]);
impl Drop for Key {
    fn drop(&mut self) {
        let &Key(ref mut k) = self;
        for e in k.mut_iter() { *e = 0 }
    }
}

/**
 * `Nonce` for symmetric encryption
 */
pub struct Nonce(pub [u8, ..NONCEBYTES]);

/**
 * `gen_key()` randomly generates a key for symmetric encryption
 *
 * THREAD SAFETY: `gen_key()` is thread-safe provided that you have
 * called `sodiumoxide::init()` once before using any other function
 * from sodiumoxide.
 */
pub fn gen_key() -> Key {
    let mut key = [0, ..KEYBYTES];
    randombytes_into(key);
    Key(key)
}

/**
 * `gen_nonce()` randomly generates a nonce for symmetric encryption
 *
 * THREAD SAFETY: `gen_nonce()` is thread-safe provided that you have
 * called `sodiumoxide::init()` once before using any other function
 * from sodiumoxide.
 *
 * NOTE: When using primitives with short nonces (e.g. salsa20, salsa208, salsa2012)
 * do not use random nonces since the probability of nonce-collision is not negligible
 */
pub fn gen_nonce() -> Nonce {
    let mut nonce = [0, ..NONCEBYTES];
    randombytes_into(nonce);
    Nonce(nonce)
}

/**
 * `stream()` produces a `len`-byte stream `c` as a function of a
 * secret key `k` and a nonce `n`.
 */
pub fn stream(len: uint,
              &Nonce(n): &Nonce,
              &Key(k): &Key) -> ~[u8] {
    unsafe {
        let mut c = from_elem(len, 0u8);
        $stream_name(c.as_mut_ptr(),
                     c.len() as c_ulonglong,
                     n.as_ptr(),
                     k.as_ptr());
        c
    }
}

/**
 * `stream_xor()` encrypts a message `m` using a secret key `k` and a nonce `n`.
 * The `stream_xor()` function returns the ciphertext `c`.
 *
 * `stream_xor()` guarantees that the ciphertext has the same length as the plaintext,
 * and is the plaintext xor the output of `stream()`.
 * Consequently `stream_xor()` can also be used to decrypt.
 */
pub fn stream_xor(m: &[u8],
                  &Nonce(n): &Nonce,
                  &Key(k): &Key) -> ~[u8] {
    unsafe {
        let mut c = from_elem(m.len(), 0u8);
        $xor_name(c.as_mut_ptr(),
                  m.as_ptr(),
                  m.len() as c_ulonglong,
                  n.as_ptr(),
                  k.as_ptr());
        c
    }
}

/**
* `stream_xor_inplace` encrypts a message `m` using a secret key `k` and a nonce `n`.
* The `stream_xor_inplace()` function encrypts the message in place.
*
* `stream_xor_inplace()` guarantees that the ciphertext has the same length as
* the plaintext, and is the plaintext xor the output of `stream_inplace()`.
* Consequently `stream_xor_inplace()` can also be used to decrypt.
*/
pub fn stream_xor_inplace(m: &mut [u8],
                          &Nonce(n): &Nonce,
                          &Key(k): &Key) {
    unsafe {
        $xor_name(m.as_mut_ptr(),
                  m.as_ptr(),
                  m.len() as c_ulonglong,
                  n.as_ptr(),
                  k.as_ptr());
    }
}

#[test]
fn test_encrypt_decrypt() {
    use randombytes::randombytes;
    for i in range(0, 1024) {
        let k = gen_key();
        let n = gen_nonce();
        let m = randombytes(i as uint);
        let c = stream_xor(m, &n, &k);
        let m2 = stream_xor(c, &n, &k);
        assert!(m == m2);
    }
}

#[test]
fn test_stream_xor() {
    use randombytes::randombytes;
    for i in range(0, 1024) {
        let k = gen_key();
        let n = gen_nonce();
        let m = randombytes(i as uint);
        let mut c = m.clone();
        let s = stream(c.len(), &n, &k);
        for (e, v) in c.mut_iter().zip(s.iter()) {
            *e ^= *v;
        }
        let c2 = stream_xor(m, &n, &k);
        assert!(c == c2);
    }
}

#[test]
fn test_stream_xor_inplace() {
    use randombytes::randombytes;
    for i in range(0, 1024) {
        let k = gen_key();
        let n = gen_nonce();
        let mut m = randombytes(i as uint);
        let mut c = m.clone();
        let s = stream(c.len(), &n, &k);
        for (e, v) in c.mut_iter().zip(s.iter()) {
            *e ^= *v;
        }
        stream_xor_inplace(m, &n, &k);
        assert!(c == m);
    }
}

))
