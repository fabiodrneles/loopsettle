//! Hand-written sparse linear algebra: CSR matrices and conjugate gradient (CG).
//! Enough for graph Laplacians; no external linear-algebra crate (CLAUDE.md section 2).

/// Compressed Sparse Row matrix of `f64`.
#[derive(Clone, Debug)]
pub struct Csr {
    pub rows: usize,
    pub cols: usize,
    pub indptr: Vec<usize>,
    pub indices: Vec<u32>,
    pub values: Vec<f64>,
}

impl Csr {
    /// Build from `(row, col, value)` triplets (duplicates are summed).
    pub fn from_triplets(rows: usize, cols: usize, mut t: Vec<(u32, u32, f64)>) -> Self {
        t.sort_by_key(|&(r, c, _)| (r, c));
        let mut indptr = vec![0usize; rows + 1];
        let mut indices = Vec::with_capacity(t.len());
        let mut values: Vec<f64> = Vec::with_capacity(t.len());
        let mut last: Option<(u32, u32)> = None;
        for (r, c, v) in t {
            if last == Some((r, c)) {
                *values.last_mut().unwrap() += v;
                continue;
            }
            last = Some((r, c));
            indptr[r as usize + 1] += 1;
            indices.push(c);
            values.push(v);
        }
        for i in 0..rows {
            indptr[i + 1] += indptr[i];
        }
        Csr { rows, cols, indptr, indices, values }
    }

    /// `y = A x`
    pub fn mul(&self, x: &[f64], y: &mut [f64]) {
        for r in 0..self.rows {
            let mut s = 0.0;
            for k in self.indptr[r]..self.indptr[r + 1] {
                s += self.values[k] * x[self.indices[k] as usize];
            }
            y[r] = s;
        }
    }

    /// `y = A^T x`
    pub fn mul_t(&self, x: &[f64], y: &mut [f64]) {
        y.iter_mut().for_each(|v| *v = 0.0);
        for r in 0..self.rows {
            let xr = x[r];
            if xr != 0.0 {
                for k in self.indptr[r]..self.indptr[r + 1] {
                    y[self.indices[k] as usize] += self.values[k] * xr;
                }
            }
        }
    }

    pub fn nnz(&self) -> usize {
        self.values.len()
    }
}

pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
pub fn norm2(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
pub fn norm1(a: &[f64]) -> f64 {
    a.iter().map(|x| x.abs()).sum()
}

/// Result of a CG solve.
#[derive(Clone, Debug)]
pub struct CgReport {
    pub iterations: usize,
    pub rel_residual: f64,
    pub converged: bool,
}

/// Conjugate gradient for a symmetric positive **semi**-definite operator `apply(x, y): y = A x`.
///
/// Starting from `x = 0` with a right-hand side in the range of `A` (true for Laplacian
/// systems built as `B^T B x = B^T f`), CG converges to the minimum-norm solution even though
/// `A` is singular. The closure parameter (`impl FnMut`) lets callers pass a matrix-free
/// operator such as `B^T (B x)` without building `B^T B`.
pub fn cg(mut apply: impl FnMut(&[f64], &mut [f64]), b: &[f64], x: &mut [f64], tol: f64, max_iter: usize) -> CgReport {
    let n = b.len();
    x.iter_mut().for_each(|v| *v = 0.0);
    let bnorm = norm2(b);
    if bnorm == 0.0 {
        return CgReport { iterations: 0, rel_residual: 0.0, converged: true };
    }
    let mut r = b.to_vec();
    let mut p = r.clone();
    let mut ap = vec![0.0; n];
    let mut rr = dot(&r, &r);
    for it in 0..max_iter {
        apply(&p, &mut ap);
        let pap = dot(&p, &ap);
        if pap <= 0.0 {
            // Direction in the null space: residual cannot be reduced further.
            return CgReport { iterations: it, rel_residual: rr.sqrt() / bnorm, converged: rr.sqrt() / bnorm <= tol };
        }
        let alpha = rr / pap;
        for i in 0..n {
            x[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
        }
        let rr_new = dot(&r, &r);
        if rr_new.sqrt() / bnorm <= tol {
            return CgReport { iterations: it + 1, rel_residual: rr_new.sqrt() / bnorm, converged: true };
        }
        let beta = rr_new / rr;
        for i in 0..n {
            p[i] = r[i] + beta * p[i];
        }
        rr = rr_new;
    }
    CgReport { iterations: max_iter, rel_residual: rr.sqrt() / bnorm, converged: false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cg_solves_spd() {
        // [[4,1],[1,3]] x = [1,2]  ->  x = [1/11, 7/11]
        let a = Csr::from_triplets(2, 2, vec![(0, 0, 4.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)]);
        let mut x = vec![0.0; 2];
        let rep = cg(|v, y| a.mul(v, y), &[1.0, 2.0], &mut x, 1e-12, 50);
        assert!(rep.converged);
        assert!((x[0] - 1.0 / 11.0).abs() < 1e-10 && (x[1] - 7.0 / 11.0).abs() < 1e-10);
    }

    #[test]
    fn transpose_product() {
        let a = Csr::from_triplets(2, 3, vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]);
        let mut y = vec![0.0; 3];
        a.mul_t(&[1.0, 1.0], &mut y);
        assert_eq!(y, vec![1.0, 3.0, 2.0]);
    }
}
