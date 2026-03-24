use wasm_bindgen::prelude::*;

use harmonypir::prp::alf::AlfPrp;
use harmonypir::prp::fast::FastPrpWrapper;
use harmonypir::prp::ff1::Ff1Prp;
use harmonypir::prp::hoang::HoangPrp;
use harmonypir::prp::Prp;

// ================================================================
// Hoang PRP benchmarks
// ================================================================

#[wasm_bindgen]
pub fn bench_hoang_forward(domain: usize, rounds: usize, count: usize) -> f64 {
    let key = [0x42u8; 16];
    let prp = HoangPrp::new(domain, rounds, &key);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

#[wasm_bindgen]
pub fn bench_hoang_inverse(domain: usize, rounds: usize, count: usize) -> f64 {
    let key = [0x42u8; 16];
    let prp = HoangPrp::new(domain, rounds, &key);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

/// Simulate one HarmonyPIR online query on one bucket using Hoang PRP.
/// Does T forward + T inverse calls (the ~2T calls per query).
/// Returns elapsed ms.
#[wasm_bindgen]
pub fn bench_hoang_one_bucket(domain: usize, rounds: usize, t: usize) -> f64 {
    let key = [0x42u8; 16];
    let prp = HoangPrp::new(domain, rounds, &key);
    let start = now();
    let mut sum: usize = 0;
    // T forward (relocation)
    for i in 0..t {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    // T inverse (access)
    for i in 0..t {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

// ================================================================
// ALF PRP benchmarks
// ================================================================

#[wasm_bindgen]
pub fn bench_alf_forward(domain: usize, count: usize) -> f64 {
    let key = [0x42u8; 16];
    let tweak = [0u8; 16];
    let prp = AlfPrp::new(&key, domain, &tweak, 0);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

#[wasm_bindgen]
pub fn bench_alf_inverse(domain: usize, count: usize) -> f64 {
    let key = [0x42u8; 16];
    let tweak = [0u8; 16];
    let prp = AlfPrp::new(&key, domain, &tweak, 0);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

/// Simulate one HarmonyPIR query on one bucket using ALF.
#[wasm_bindgen]
pub fn bench_alf_one_bucket(domain: usize, t: usize, tweak_id: u64) -> f64 {
    let key = [0x42u8; 16];
    let mut tweak = [0u8; 16];
    tweak[..8].copy_from_slice(&tweak_id.to_le_bytes());
    let prp = AlfPrp::new(&key, domain, &tweak, 0);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..t {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    for i in 0..t {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

// ================================================================
// FastPRP benchmarks
// ================================================================

/// Returns [build_ms, eval_ms].
#[wasm_bindgen]
pub fn bench_fastprp_forward(domain: usize, count: usize) -> Vec<f64> {
    let key = [0x42u8; 16];
    let build_start = now();
    let prp = FastPrpWrapper::new(&key, domain);
    let build_ms = now() - build_start;

    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    let eval_ms = now() - start;
    black_box(sum);
    vec![build_ms, eval_ms]
}

#[wasm_bindgen]
pub fn bench_fastprp_inverse(domain: usize, count: usize) -> Vec<f64> {
    let key = [0x42u8; 16];
    let prp = FastPrpWrapper::new(&key, domain);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    vec![elapsed]
}

/// Simulate one HarmonyPIR query on one bucket using FastPRP.
/// Returns [build_ms, query_ms].
#[wasm_bindgen]
pub fn bench_fastprp_one_bucket(domain: usize, t: usize) -> Vec<f64> {
    let key = [0x42u8; 16];
    let build_start = now();
    let prp = FastPrpWrapper::new(&key, domain);
    let build_ms = now() - build_start;

    let start = now();
    let mut sum: usize = 0;
    for i in 0..t {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    for i in 0..t {
        sum = sum.wrapping_add(prp.inverse(i % domain));
    }
    let query_ms = now() - start;
    black_box(sum);
    vec![build_ms, query_ms]
}

// ================================================================
// Multi-bucket sequential benchmark (for comparison with workers)
// ================================================================

/// Run N_buckets sequential bucket queries with Hoang. Returns total ms.
#[wasm_bindgen]
pub fn bench_hoang_n_buckets(domain: usize, rounds: usize, t: usize, n_buckets: usize) -> f64 {
    let start = now();
    let mut sum: usize = 0;
    for b in 0..n_buckets {
        let mut key = [0x42u8; 16];
        key[0] = (b & 0xFF) as u8;
        key[1] = ((b >> 8) & 0xFF) as u8;
        let prp = HoangPrp::new(domain, rounds, &key);
        for i in 0..t {
            sum = sum.wrapping_add(prp.forward(i % domain));
        }
        for i in 0..t {
            sum = sum.wrapping_add(prp.inverse(i % domain));
        }
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

/// Run N_buckets sequential bucket queries with ALF. Returns total ms.
#[wasm_bindgen]
pub fn bench_alf_n_buckets(domain: usize, t: usize, n_buckets: usize) -> f64 {
    let key = [0x42u8; 16];
    let start = now();
    let mut sum: usize = 0;
    for b in 0..n_buckets {
        let mut tweak = [0u8; 16];
        tweak[..8].copy_from_slice(&(b as u64).to_le_bytes());
        let prp = AlfPrp::new(&key, domain, &tweak, 0);
        for i in 0..t {
            sum = sum.wrapping_add(prp.forward(i % domain));
        }
        for i in 0..t {
            sum = sum.wrapping_add(prp.inverse(i % domain));
        }
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

/// Run N_buckets sequential bucket queries with FastPRP (query only, includes build).
/// Returns [total_build_ms, total_query_ms].
#[wasm_bindgen]
pub fn bench_fastprp_n_buckets(domain: usize, t: usize, n_buckets: usize) -> Vec<f64> {
    let mut total_build = 0.0;
    let mut total_query = 0.0;
    let mut sum: usize = 0;
    for b in 0..n_buckets {
        let mut key = [0x42u8; 16];
        key[0] = (b & 0xFF) as u8;
        key[1] = ((b >> 8) & 0xFF) as u8;
        let build_start = now();
        let prp = FastPrpWrapper::new(&key, domain);
        total_build += now() - build_start;

        let start = now();
        for i in 0..t {
            sum = sum.wrapping_add(prp.forward(i % domain));
        }
        for i in 0..t {
            sum = sum.wrapping_add(prp.inverse(i % domain));
        }
        total_query += now() - start;
    }
    black_box(sum);
    vec![total_build, total_query]
}

// ================================================================
// FF1 benchmark
// ================================================================

#[wasm_bindgen]
pub fn bench_ff1_forward(domain: usize, count: usize) -> f64 {
    let key = [0x42u8; 16];
    let prp = Ff1Prp::new(domain, &key);
    let start = now();
    let mut sum: usize = 0;
    for i in 0..count {
        sum = sum.wrapping_add(prp.forward(i % domain));
    }
    let elapsed = now() - start;
    black_box(sum);
    elapsed
}

// ================================================================
// Correctness checks
// ================================================================

#[wasm_bindgen]
pub fn verify_hoang(domain: usize, rounds: usize) -> bool {
    let key = [0x42u8; 16];
    let prp = HoangPrp::new(domain, rounds, &key);
    for x in 0..domain.min(1000) {
        let y = prp.forward(x);
        if y >= domain || prp.inverse(y) != x {
            return false;
        }
    }
    true
}

#[wasm_bindgen]
pub fn verify_alf(domain: usize) -> bool {
    let key = [0x42u8; 16];
    let tweak = [0u8; 16];
    let prp = AlfPrp::new(&key, domain, &tweak, 0);
    for x in (0..domain).step_by((domain / 500).max(1)) {
        let y = prp.forward(x);
        if y >= domain || prp.inverse(y) != x {
            return false;
        }
    }
    true
}

#[wasm_bindgen]
pub fn verify_fastprp(domain: usize) -> bool {
    let key = [0x42u8; 16];
    let prp = FastPrpWrapper::new(&key, domain);
    for x in 0..domain.min(1000) {
        let y = prp.forward(x);
        if y >= domain || prp.inverse(y) != x {
            return false;
        }
    }
    true
}

/// Full end-to-end protocol test in WASM.
#[wasm_bindgen]
pub fn verify_protocol(n: usize) -> Vec<u8> {
    use harmonypir::params::Params;
    use harmonypir::protocol::Client;
    use harmonypir::server::Server;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    let w = 32;
    let t = (n as f64).sqrt().ceil() as usize;
    let t = t.max(4);

    let db: Vec<Vec<u8>> = (0..n)
        .map(|i| {
            let mut entry = vec![0u8; w];
            let bytes = (i as u64).to_le_bytes();
            entry[..8.min(w)].copy_from_slice(&bytes[..8.min(w)]);
            entry
        })
        .collect();

    let server = Server::new(db.clone());
    let params = match Params::new(n, w, t) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let prp = Box::new(HoangPrp::new(2 * n, params.r, &[0x42u8; 16]));
    let mut client = match Client::offline(params, prp, &server) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut rng = ChaCha20Rng::seed_from_u64(42);
    match client.query(0, &server, &mut rng) {
        Ok(entry) => entry,
        Err(_) => vec![],
    }
}

// --- Helpers ---

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

#[inline(never)]
fn black_box<T>(x: T) -> T {
    let ptr = &x as *const T;
    unsafe { core::ptr::read_volatile(ptr) }
}
