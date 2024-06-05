use crate::point::PointMatrix;
use crate::trajectory::Trajectory;
use geo::FrechetDistance;

fn euclidean(p1: &PointMatrix, p2: &PointMatrix) -> f64 {
    (p2 - p1).norm_squared()
}

// fn c(ca: &mut Vec<Vec<f64>>, a: &Trajectory, b: &Trajectory, i: usize, j: usize) -> f64 {
//     if ca[i][j] > -1.0 {
//         return ca[i][j];
//     }
//     if i == 0 && j == 0 {
//         ca[i][j] = euclidean(&a[0], &b[0]);
//     } else if i > 0 && j == 0 {
//         let rec = c(ca, a, b, i - 1, 0);
//         ca[i][j] = euclidean(&a[i], &b[0]).max(rec);
//     } else if i == 0 && j > 0 {
//         let rec = c(ca, a, b, 0, j - 1);
//         ca[i][j] = euclidean(&a[0], &b[j]).max(rec);
//     } else {
//         let rec_a = c(ca, a, b, i - 1, j - 1);
//         let rec_b = c(ca, a, b, i - 1, j);
//         let rec_c = c(ca, a, b, i, j - 1);
//         ca[i][j] = rec_a.min(rec_b).min(rec_c).max(euclidean(&a[i], &b[j]));
//     }
//     ca[i][j]
// }

// // https://github.com/Cecca/FRESH/blob/master/core/frechet.cpp#L53
// pub fn discrete_frechet_distance_r(t1: &Trajectory, t2: &Trajectory) -> f64 {
//     c(
//         &mut vec![vec![-1.0; t2.len()]; t1.len()],
//         t1,
//         t2,
//         t1.len() - 1,
//         t2.len() - 1,
//     )
// }

// // https://github.com/Cecca/FRESH/blob/master/core/frechet.h#L80C8-L80C33
// pub fn discrete_frechet_distance(t1: &Trajectory, t2: &Trajectory) -> f64 {
//     let t1_len = t1.len();
//     let mut prev_row = vec![0.; t1_len];
//     let mut current_row = vec![0.; t1_len];
//     current_row[0] = euclidean(&t1[0], &t2[0]);

//     for i in 1..t1.len() {
//         current_row[i] = euclidean(&t1[i], &t2[0]).max(current_row[i - 1]);
//     }

//     for j in 1..t2.len() {
//         current_row.swap_with_slice(&mut prev_row[..]);
//         current_row[0] = euclidean(&t1[0], &t2[j]).max(prev_row[0]);

//         for i in 1..t1_len {
//             current_row[i] = euclidean(&t1[i], &t2[j])
//                 .max(current_row[i - 1])
//                 .min(prev_row[i])
//                 .min(prev_row[i - 1]);
//         }
//     }

//     current_row[t1.len() - 1].sqrt()
// }

// const fn pair_to_idx(i: usize, j: usize, n: usize) -> usize {
//     i * n + j
// }

// const fn idx_to_pair(idx: usize, n: usize) -> (usize, usize) {
//     (idx / n, idx % n)
// }

fn discrete_frechet_distance_predicate_r(p1: &PointMatrix, p2: &PointMatrix, bound: f64) -> bool {
    euclidean(p1, p2) > bound
}

fn start_end_heuristic(t1: &Trajectory, t2: &Trajectory, bound: f64) -> bool {
    discrete_frechet_distance_predicate_r(&t1[0], &t2[0], bound)
        || discrete_frechet_distance_predicate_r(&t1[t1.len() - 1], &t2[t2.len() - 1], bound)
}

// fn reverse_start_end_heiristic(t1: &Trajectory, t2: &Trajectory, bound: f64) -> bool {
//     discrete_frechet_distance_predicate_r(&t1[0], &t2[t2.len() - 1], bound)
//         || discrete_frechet_distance_predicate_r(&t1[t1.len() - 1], &t2[0], bound)
// }

fn iter_indices_heuristic(
    mut indices: impl Iterator<Item = (usize, usize)>,
    t1: &Trajectory,
    t2: &Trajectory,
    bound: f64,
) -> bool {
    indices.any(|(i, j)| discrete_frechet_distance_predicate_r(&t1[i], &t2[j], bound))
}

pub fn discrete_frechet_distance_predicate(t1: &Trajectory, t2: &Trajectory, bound: f64) -> bool {
    if start_end_heuristic(t1, t2, bound) {
        return false;
    }

    if iter_indices_heuristic(
        (1..t1.len() - 1)
            .map(|i| (i, 0))
            .chain((1..t2.len() - 1).map(|j| (t1.len() - 1, j))),
        t1,
        t2,
        bound,
    ) {
        return false;
    }
    if iter_indices_heuristic(
        (1..t2.len() - 1)
            .map(|i| (0, i))
            .chain((1..t1.len() - 1).map(|j| (j, t2.len() - 1))),
        t1,
        t2,
        bound,
    ) {
        return false;
    }

    t1.line_string().frechet_distance(&t2.line_string()) <= bound

    // let k = pair_to_idx(t1_len - 1, t2_len - 1, t2_len);
    // visited[k] = true;
    // stack.push(k);

    // while let Some(k) = stack.pop() {
    //     let (i, j) = idx_to_pair(k, t2_len);

    //     if i == 0 && j == 0 {
    //         return true;
    //         // return t1.line_string().frechet_distance(&t2.line_string()) <= bound;
    //     }

    //     if i > 0 && j > 0 {
    //         let k = pair_to_idx(i - 1, j - 1, t2_len);
    //         if !visited[k] && !discrete_frechet_distance_predicate_r(&t1[i - 1], &t2[j - 1], bound)
    //         {
    //             // let d = ;
    //             if euclidean(&t1[i], &t2[j]) <= bound {
    //                 visited[k] = true;
    //                 stack.push(k);
    //             }
    //         }
    //     }

    //     if i > 0 {
    //         let k = pair_to_idx(i - 1, j, t2_len);
    //         if !visited[k] && !discrete_frechet_distance_predicate_r(&t1[i - 1], &t2[j], bound) {
    //             // let d = ;
    //             if euclidean(&t1[i], &t2[j]) <= bound {
    //                 visited[k] = true;
    //                 stack.push(k);
    //             }
    //         }
    //     }

    //     if j > 0 {
    //         let k = pair_to_idx(i, j - 1, t2_len);
    //         if !visited[k] && !discrete_frechet_distance_predicate_r(&t1[i], &t2[j - 1], bound) {
    //             if euclidean(&t1[i], &t2[j]) <= bound {
    //                 visited[k] = true;
    //                 stack.push(k);
    //             }
    //         }
    //     }
    // }
    // false
}

// pub type DistanceHeuristic = fn(&Trajectory, &Trajectory, f64) -> bool;

// pub enum DistanceHeuristicResult {
//     True,
//     False,
//     Unknown,
// }

// pub fn frechet_decider(t1: &Trajectory, t2: &Trajectory, bound: f64) -> bool {
//     match inner_frechet_decider(t1, t2, bound, [].into_iter()) {
//         DistanceHeuristicResult::True => true,
//         DistanceHeuristicResult::False => false,
//         DistanceHeuristicResult::Unknown => {
//             t1.line_string().frechet_distance(&t2.line_string()) <= bound
//         }
//     }
// }

// // Bringmann et al. - 2019 - Walking the Dog Fast in Practice
// fn inner_frechet_decider(
//     t1: &Trajectory,
//     t2: &Trajectory,
//     bound: f64,
//     filters: impl Iterator<Item = DistanceHeuristic>,
// ) -> DistanceHeuristicResult {
//     if start_end_heuristic(t1, t2, bound) {
//         return DistanceHeuristicResult::False;
//     }

//     for filter in filters {
//         if filter(t1, t2, bound) {
//             return DistanceHeuristicResult::True;
//         }
//     }

//     DistanceHeuristicResult::Unknown
// }

pub trait FrechetDistanceFilter {
    fn frechet_distance_filter(
        self,
        dataset: &[Trajectory],
        queryset: &[Trajectory],
        range: f64,
    ) -> impl Iterator<Item = (usize, usize)>;
}

impl<T> FrechetDistanceFilter for T
where
    T: IntoIterator<Item = (usize, usize)>,
{
    fn frechet_distance_filter(
        self,
        dataset: &[Trajectory],
        queryset: &[Trajectory],
        range: f64,
    ) -> impl Iterator<Item = (usize, usize)> {
        self.into_iter().filter(move |&(query, candidate)| {
            discrete_frechet_distance_predicate(&queryset[query], &dataset[candidate], range)
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::io::trajectory_dataset;

    #[test]
    fn filters_work() {
        let dataset = trajectory_dataset("porto-data.parquet").unwrap();
        let dataset = dataset.trajectories();
        let (m, n) = (0usize..100, 100usize..200);

        let bound = 0.01;

        for (i, j) in m.zip(n) {
            let a = &dataset[i];
            let b = &dataset[j];
            let result = discrete_frechet_distance_predicate(a, b, bound);
            let result_r = a.line_string().frechet_distance(&b.line_string());
            println!("{} {}", result, result_r);
            assert_eq!(
                result_r <= bound,
                result,
                "Discrete Frechet distance predicate failed: i={}, j={}",
                i,
                j
            );
        }
    }
}
