use crate::id::TrajectoryID;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use sysinfo::CpuRefreshKind;
use sysinfo::MemoryRefreshKind;
use sysinfo::Pid;
use sysinfo::ProcessRefreshKind;
use sysinfo::RefreshKind;
use sysinfo::System;

/// Measures the time it takes to execute a function.
///
/// ```
/// use master::util::timeit;
/// let result = timeit(|| {
///     let mut sum: u32 = 0;
///     for i in 0..1000000 {
///         sum += 1;
///     }
///     sum
/// });
/// assert_eq!(result, 1000000);
#[allow(dead_code)]
pub fn timeit<F: FnOnce() -> T, T>(f: F, description: &str) -> T {
    let start = SystemTime::now();
    let result = f();
    let duration = SystemTime::now().duration_since(start).unwrap();
    println!("[{:?}] {}", duration, description);
    result
}

/// Walks a directory and calls the callback for each file.
///
///
/// ```
/// use std::fs;
/// use std::fs::DirEntry;
/// use std::io;
/// use master::walk_dir;
///
/// let mut files = Vec::<String>::new();
/// let mut cb = |entry: &DirEntry| {
///    files.push(entry.path().to_str().unwrap().to_string());
/// };
/// walk_dir("data", &mut cb).unwrap();
/// assert!(files.len() > 2);
///
/// ```
#[allow(dead_code)]
pub fn walk_dir<P>(dir: P, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()>
where
    P: AsRef<Path>,
{
    if dir.as_ref().is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.as_path().is_dir() {
                walk_dir::<&Path>(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn dataset_path(arg_n: usize) -> Result<PathBuf> {
    Ok(PathBuf::from(
        args()
            .skip(arg_n)
            .next()
            .ok_or_else(|| anyhow::anyhow!("Usage: <dataset>"))?,
    ))
}

/// Runs the plot-trajectories.py script with the given query and candidate indices.
pub fn run_plot_curves_cmd<'a>(
    qid: TrajectoryID,
    qcandidates: impl Iterator<Item = TrajectoryID<'a>>,
) {
    Command::new("python")
        .arg("py-utils/plot-trajectories.py")
        .arg("porto.parquet")
        .arg("porto-query.parquet")
        .arg("-q")
        .arg(qid.value().to_string())
        .args(qcandidates.map(|v| v.value().to_string()))
        .arg("-o")
        .arg(format!("results/porto-{}.html", qid.value()))
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("failed to execute process");
}

/// Maps a list of query and candidate indices to a list of query and candidate trajectory IDs.
///
/// ```
/// use master::util::map_to_trajectory_ids;
/// use master::id::TrajectoryID;
/// let dataset = vec![TrajectoryID::new(0), TrajectoryID::new(1)];
/// let queryset = vec![TrajectoryID::new(2), TrajectoryID::new(3)];
/// let results = vec![(0, 1), (1, 0)];
/// let mapped = map_to_trajectory_ids(results, &dataset, &queryset).collect::<Vec<_>>();
/// assert_eq!(mapped, vec![[queryset[0], dataset[1]], [queryset[1], dataset[0]]]);
/// ```
pub fn map_to_trajectory_ids<'a>(
    results: impl IntoIterator<Item = (usize, usize)> + 'a,
    dataset: &'a [TrajectoryID<'a>],
    queryset: &'a [TrajectoryID<'a>],
) -> impl Iterator<Item = (TrajectoryID<'a>, TrajectoryID<'a>)> + 'a {
    results.into_iter().filter_map(|(query, candidate)| {
        match (queryset.get(query), dataset.get(candidate)) {
            (Some(q), Some(c)) => Some((q.clone(), c.clone())),
            _ => None,
        }
    })
}

pub fn count_candidates<'a>(
    results: impl IntoIterator<Item = (usize, usize)> + 'a,
) -> HashMap<usize, usize> {
    let mut counts = HashMap::new();
    for (query, _candidate) in results {
        *counts.entry(query).or_insert(0) += 1;
    }
    counts
}

lazy_static::lazy_static! {
    pub static ref SYS_INFO: Mutex<System> = Mutex::new(System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    ));
}

#[derive(Debug, Serialize)]
pub struct MemoryUsage {
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: f32,
    pub cpus: Vec<u64>,
    pub index_size: IndexSize,
}

impl MemoryUsage {
    pub fn sample(index_size: IndexSize) -> MemoryUsage {
        let mut sys = SYS_INFO.lock().unwrap();
        let pid = Pid::from_u32(std::process::id());
        sys.refresh_process_specifics(
            pid,
            ProcessRefreshKind::default()
                .with_cpu()
                .with_memory()
                .without_cwd()
                .without_disk_usage()
                .without_cmd()
                .without_environ()
                .without_exe()
                .without_root()
                .without_user(),
        );
        let process = sys.process(pid).unwrap();
        let memory = process.memory();
        let virtual_memory = process.virtual_memory();
        let cpu_usage = process.cpu_usage();
        let cpus: Vec<u64> = Vec::from_iter(sys.cpus().iter().map(|cpu| cpu.frequency()));

        MemoryUsage {
            memory,
            virtual_memory,
            cpu_usage,
            cpus,
            index_size,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MasterStats<'a, T, C> {
    name: &'a str,
    dataset: PathBuf,
    queryset: Option<PathBuf>,
    mem: HashMap<usize, MemoryUsage>,
    config: C,
    dataset_load: Option<Duration>,
    index_build_time: Option<Duration>,
    index_query_time: Option<Duration>,
    index_query_size: Option<usize>,
    index_stats: Option<T>,
    candidates: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Default)]
pub struct IndexSize {
    pub stack_size: usize,
    pub heap_size: usize,
}

pub trait GetIndexSize {
    fn index_size(&self) -> IndexSize; 
}

impl<'a, T, C: Clone + Serialize> MasterStats<'a, T, C> {
    pub fn new(
        name: &'a str,
        config: &C,
        dataset: impl AsRef<Path>,
        queryset: Option<impl AsRef<Path>>,
    ) -> Self {
        MasterStats {
            name,
            dataset: dataset.as_ref().to_path_buf(),
            queryset: queryset.map(|p| p.as_ref().to_path_buf()),
            config: config.clone(),
            mem: HashMap::new(),
            dataset_load: None,
            index_build_time: None,
            index_query_time: None,
            index_query_size: None,
            index_stats: None,
            candidates: None,
        }
    }

    pub fn sample_mem(&mut self, n: usize, index_size: IndexSize) {
        self.mem.insert(n, MemoryUsage::sample(index_size));
    }

    pub fn data_load_time(&mut self, time: Duration) {
        self.dataset_load = Some(time);
    }

    pub fn index_build_time(&mut self, time: Duration) {
        self.index_build_time = Some(time);
    }

    pub fn index_query_time(&mut self, time: Duration) {
        self.index_query_time = Some(time);
    }

    pub fn index_query_size(&mut self, size: usize) {
        self.index_query_size = Some(size);
    }

    pub fn index_stats(&mut self, states: T) {
        self.index_stats = Some(states);
    }

    pub fn index_stats_mut_unchecked(&mut self) -> &mut T {
        self.index_stats.as_mut().unwrap()
    }

    pub fn candidates<I>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = (TrajectoryID<'a>, TrajectoryID<'a>)>,
    {
        let mut results = HashMap::<String, Vec<String>>::new();
        candidates.into_iter().for_each(|(query, candidate)| {
            results
                .entry(query.value().to_string())
                .or_insert(Vec::new())
                .push(candidate.value().to_string());
        });
        self.candidates = Some(results);
    }
}
